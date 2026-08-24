//! Thin native shell for the adoc desktop app.
//!
//! All domain logic lives in the TypeScript packages (@adoc/core, @adoc/git,
//! @adoc/indexer) running in the webview. This crate only provides the two
//! ports the domain needs — a filesystem and an allowlisted process runner —
//! plus streaming for local agent CLIs (DESIGN.md §12).

use serde::Serialize;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tauri::Emitter;

/// Only these binaries may ever be spawned from the webview (DESIGN.md §12, §14).
const ALLOWED_COMMANDS: &[&str] = &["git", "claude", "codex"];

#[derive(Serialize)]
pub struct ProcResult {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

fn assert_allowed(command: &str) -> Result<(), String> {
    if ALLOWED_COMMANDS.contains(&command) {
        Ok(())
    } else {
        Err(format!("command not allowed: {command}"))
    }
}

// ---------------------------------------------------------------------------
// FileSystem port
// ---------------------------------------------------------------------------

#[tauri::command]
fn fs_read_file(path: String) -> Result<String, String> {
    std::fs::read_to_string(&path).map_err(|e| format!("{path}: {e}"))
}

#[tauri::command]
fn fs_write_file(path: String, content: String) -> Result<(), String> {
    if let Some(parent) = Path::new(&path).parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(&path, content).map_err(|e| format!("{path}: {e}"))
}

#[tauri::command]
fn fs_exists(path: String) -> bool {
    Path::new(&path).exists()
}

#[tauri::command]
fn fs_mkdirp(path: String) -> Result<(), String> {
    std::fs::create_dir_all(&path).map_err(|e| format!("{path}: {e}"))
}

#[tauri::command]
fn fs_remove(path: String) -> Result<(), String> {
    let p = Path::new(&path);
    if p.is_dir() {
        std::fs::remove_dir_all(p).map_err(|e| e.to_string())
    } else if p.exists() {
        std::fs::remove_file(p).map_err(|e| e.to_string())
    } else {
        Ok(())
    }
}

fn walk(dir: &Path, prefix: &str, out: &mut Vec<String>) -> Result<(), String> {
    let entries = std::fs::read_dir(dir).map_err(|e| e.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name == ".git" || name == "node_modules" {
            continue;
        }
        let rel = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{prefix}/{name}")
        };
        let path = entry.path();
        if path.is_dir() {
            walk(&path, &rel, out)?;
        } else {
            out.push(rel);
        }
    }
    Ok(())
}

/// Recursive file listing with paths relative to `dir` (uses `/` separators).
#[tauri::command]
fn fs_list_files(dir: String) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    walk(&PathBuf::from(&dir), "", &mut out)?;
    Ok(out)
}

// ---------------------------------------------------------------------------
// Process runner port
// ---------------------------------------------------------------------------

fn spawn_process(
    command: &str,
    args: &[String],
    cwd: &str,
    stdin: Option<&str>,
    mut on_chunk: impl FnMut(String),
) -> Result<ProcResult, String> {
    assert_allowed(command)?;
    let mut child = Command::new(command)
        .args(args)
        .current_dir(cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to spawn {command}: {e}"))?;

    if let Some(input) = stdin {
        if let Some(mut handle) = child.stdin.take() {
            // Ignore broken-pipe: the child may exit before reading everything.
            let _ = handle.write_all(input.as_bytes());
        }
    } else {
        drop(child.stdin.take());
    }

    // Drain stderr on a thread so neither pipe can deadlock when full.
    let stderr_pipe = child.stderr.take();
    let stderr_thread = std::thread::spawn(move || {
        let mut buf = String::new();
        if let Some(pipe) = stderr_pipe {
            let mut reader = BufReader::new(pipe);
            let mut line = String::new();
            while let Ok(n) = reader.read_line(&mut line) {
                if n == 0 {
                    break;
                }
                buf.push_str(&line);
                line.clear();
            }
        }
        buf
    });

    let mut stdout_buf = String::new();
    if let Some(pipe) = child.stdout.take() {
        let mut reader = BufReader::new(pipe);
        let mut line = String::new();
        while let Ok(n) = reader.read_line(&mut line) {
            if n == 0 {
                break;
            }
            stdout_buf.push_str(&line);
            on_chunk(std::mem::take(&mut line));
        }
    }

    let status = child.wait().map_err(|e| e.to_string())?;
    let stderr_buf = stderr_thread.join().unwrap_or_default();
    Ok(ProcResult {
        code: status.code().unwrap_or(-1),
        stdout: stdout_buf,
        stderr: stderr_buf,
    })
}

/// Run an allowlisted command to completion.
#[tauri::command]
async fn proc_run(
    command: String,
    args: Vec<String>,
    cwd: String,
    stdin: Option<String>,
) -> Result<ProcResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        spawn_process(&command, &args, &cwd, stdin.as_deref(), |_| {})
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Run an allowlisted command, streaming stdout lines to the webview as
/// `proc-chunk:{stream_id}` events (agent CLIs stream their answer).
#[tauri::command]
async fn proc_stream(
    window: tauri::Window,
    stream_id: String,
    command: String,
    args: Vec<String>,
    cwd: String,
    stdin: Option<String>,
) -> Result<ProcResult, String> {
    let event = format!("proc-chunk:{stream_id}");
    tauri::async_runtime::spawn_blocking(move || {
        spawn_process(&command, &args, &cwd, stdin.as_deref(), |chunk| {
            let _ = window.emit(&event, chunk);
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

// ---------------------------------------------------------------------------

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            fs_read_file,
            fs_write_file,
            fs_exists,
            fs_mkdirp,
            fs_remove,
            fs_list_files,
            proc_run,
            proc_stream
        ])
        .run(tauri::generate_context!())
        .expect("error while running adoc");
}
