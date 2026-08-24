use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, bail};
use typify::{TypeSpace, TypeSpaceSettings};

fn main() -> anyhow::Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    let generated = root.join("packages/contracts/src/generated");
    let rust_generated = root.join("crates/contracts/src/generated.rs");
    let temporary = tempfile::tempdir()?;
    let candidate = temporary.path().join("generated");

    let status = Command::new("bun")
        .args(["run", "packages/contracts/scripts/generate.mjs"])
        .arg(&candidate)
        .current_dir(&root)
        .status()
        .context("run TypeScript contract generator")?;
    if !status.success() {
        bail!("TypeScript contract generator failed");
    }

    let mut schema_value: serde_json::Value =
        serde_json::from_slice(&fs::read(candidate.join("contract-bundle.schema.json"))?)?;
    remove_runtime_only_keywords(&mut schema_value);
    let schema: schemars::schema::RootSchema = serde_json::from_value(schema_value)?;
    let settings = TypeSpaceSettings::default();
    let mut type_space = TypeSpace::new(&settings);
    type_space.add_root_schema(schema)?;
    let syntax = syn::parse2::<syn::File>(type_space.to_stream())?;
    let rust_source = format!(
        "// Generated from canonical contracts. Do not edit.\n#![allow(clippy::derivable_impls, clippy::large_enum_variant)]\n{}",
        prettyplease::unparse(&syntax)
    );
    let rust_candidate = temporary.path().join("generated.rs");
    fs::write(&rust_candidate, rust_source)?;
    let status = Command::new("rustfmt")
        .args(["--edition", "2024"])
        .arg(&rust_candidate)
        .status()
        .context("format generated Rust contracts")?;
    if !status.success() {
        bail!("rustfmt failed for generated contracts");
    }

    if env::args().any(|argument| argument == "--check") {
        compare_trees(&generated, &candidate)?;
        compare_files(&rust_generated, &rust_candidate)?;
    } else {
        fs::create_dir_all(&generated)?;
        for entry in fs::read_dir(&candidate)? {
            let entry = entry?;
            fs::copy(entry.path(), generated.join(entry.file_name()))?;
        }
        fs::copy(rust_candidate, rust_generated)?;
    }
    Ok(())
}

fn remove_runtime_only_keywords(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Array(values) => {
            for value in values {
                remove_runtime_only_keywords(value);
            }
        }
        serde_json::Value::Object(object) => {
            for keyword in ["if", "then", "else", "unevaluatedProperties"] {
                object.remove(keyword);
            }
            for value in object.values_mut() {
                remove_runtime_only_keywords(value);
            }
        }
        _ => {}
    }
}

fn compare_trees(expected: &Path, actual: &Path) -> anyhow::Result<()> {
    for entry in fs::read_dir(actual)? {
        let entry = entry?;
        compare_files(&expected.join(entry.file_name()), &entry.path())?;
    }
    Ok(())
}

fn compare_files(expected: &Path, actual: &Path) -> anyhow::Result<()> {
    if fs::read(expected).ok() != fs::read(actual).ok() {
        bail!("generated contract is stale: {}", expected.display());
    }
    Ok(())
}
