#![forbid(unsafe_code)]

use std::{env, fs, sync::Arc};

use adoc_application::identity::{KeyRing, SigningKey};
use adoc_document::{ValidatedContent, canonical_hash};
use anyhow::{Context, bail};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::postgres::PgPoolOptions;
use uuid::{Uuid, uuid};

const OWNER_ID: Uuid = uuid!("70000000-0000-7000-8000-000000000001");
const MEMBER_ID: Uuid = uuid!("70000000-0000-7000-8000-000000000002");
const WORKSPACE_ID: Uuid = uuid!("70000000-0000-7000-8000-000000000010");
const DOCUMENT_ID: Uuid = uuid!("70000000-0000-7000-8000-000000000020");
const VERSION_ID: Uuid = uuid!("70000000-0000-7000-8000-000000000021");
const DRAFT_ID: Uuid = uuid!("70000000-0000-7000-8000-000000000022");
const PRIVATE_DOCUMENT_ID: Uuid = uuid!("70000000-0000-7000-8000-000000000023");
const TRASH_DOCUMENT_ID: Uuid = uuid!("70000000-0000-7000-8000-000000000024");
const PUBLIC_LINK_ID: Uuid = uuid!("70000000-0000-7000-8000-000000000030");
const OWNER_TOKEN: &str = "browser-owner-session-token-v1";
const MEMBER_TOKEN: &str = "browser-member-session-token-v1";
const PUBLIC_TOKEN: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RotatingSecretFile {
    current: SecretEntry,
    previous: Option<SecretEntry>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SecretEntry {
    id: String,
    value: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FixtureOutput {
    schema_version: u8,
    run_id: String,
    workspace_id: Uuid,
    workspace_slug: String,
    document_id: Uuid,
    private_document_id: Uuid,
    trash_document_id: Uuid,
    owner_user_id: Uuid,
    owner_session_token: &'static str,
    member_user_id: Uuid,
    member_session_token: &'static str,
    public_token: &'static str,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let run_id = env::var("ADOC_BROWSER_RUN_ID").unwrap_or_else(|_| "local".to_owned());
    if run_id.is_empty()
        || run_id.len() > 24
        || !run_id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        bail!("ADOC_BROWSER_RUN_ID must match [a-z0-9-]{{1,24}}");
    }
    let database_url = secret("ADOC_TEST_DATABASE_URL", "ADOC_TEST_DATABASE_URL_FILE")?;
    let session_secret_path =
        env::var("ADOC_SESSION_HMAC_KEY_FILE").context("ADOC_SESSION_HMAC_KEY_FILE is required")?;
    let secret: RotatingSecretFile = serde_json::from_slice(
        &fs::read(&session_secret_path).context("read session HMAC key file")?,
    )
    .context("parse session HMAC key file")?;
    let key_ring = KeyRing::new(
        SigningKey {
            id: secret.current.id.clone(),
            value: Arc::from(secret.current.value.as_bytes()),
        },
        secret.previous.map(|entry| SigningKey {
            id: entry.id,
            value: Arc::from(entry.value.as_bytes()),
        }),
    )
    .map_err(|_| anyhow::anyhow!("invalid session HMAC key ring"))?;
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&database_url)
        .await
        .context("connect browser fixture database")?;
    let slug = format!("browser-{run_id}");
    let now = Utc::now();
    let visual_epoch = DateTime::parse_from_rfc3339("2026-08-25T00:00:00Z")
        .expect("fixed browser epoch must be valid")
        .with_timezone(&Utc);
    let content = ValidatedContent::parse(json!({
        "schemaVersion": 1,
        "root": {"type":"doc","children":[
            {"id":"70000000-0000-7000-8000-000000000101","type":"heading","level":1,"children":[{"type":"text","text":"인증 설계"}]},
            {"id":"70000000-0000-7000-8000-000000000102","type":"paragraph","children":[{"type":"text","text":"발행된 지식을 기준으로 설명합니다."}]}
        ]}
    }))
    .map_err(|_| anyhow::anyhow!("fixture content must satisfy CONTRACT-01"))?
    .into_value();
    let fingerprint = canonical_hash(&content);
    let mut tx = pool.begin().await?;
    sqlx::query("INSERT INTO users(id,identity_issuer,google_subject,email,display_name,locale,timezone,theme) VALUES($1,'https://accounts.google.com',$2,$3,$4,'ko','Asia/Seoul','LIGHT'),($5,'https://accounts.google.com',$6,$7,$8,'en','UTC','DARK')")
        .bind(OWNER_ID).bind(format!("browser-owner-{run_id}")).bind("owner@browser.example.test").bind("브라우저 소유자")
        .bind(MEMBER_ID).bind(format!("browser-member-{run_id}")).bind("member@browser.example.test").bind("Browser Member")
        .execute(&mut *tx).await?;
    for (user_id, raw_token) in [(OWNER_ID, OWNER_TOKEN), (MEMBER_ID, MEMBER_TOKEN)] {
        let candidate = key_ring.hash_current(raw_token);
        sqlx::query("INSERT INTO sessions(id_hash,hash_key_id,user_id,created_at,last_seen_at,idle_expires_at,absolute_expires_at) VALUES($1,$2,$3,$4,$4,$5,$6)")
            .bind(candidate.hash.0.as_slice()).bind(candidate.key_id).bind(user_id).bind(now)
            .bind(now + Duration::hours(12)).bind(now + Duration::days(30)).execute(&mut *tx).await?;
    }
    sqlx::query("INSERT INTO workspaces(id,slug,name,created_by) VALUES($1,$2,'Alpha Browser',$3)")
        .bind(WORKSPACE_ID)
        .bind(&slug)
        .bind(OWNER_ID)
        .execute(&mut *tx)
        .await?;
    sqlx::query("INSERT INTO memberships(workspace_id,user_id,role,status) VALUES($1,$2,'OWNER','ACTIVE'),($1,$3,'MEMBER','ACTIVE')")
        .bind(WORKSPACE_ID).bind(OWNER_ID).bind(MEMBER_ID).execute(&mut *tx).await?;
    for (id, rank, title, status) in [
        (
            DOCUMENT_ID,
            "00000000000000000000000000000001",
            "Authentication",
            "ACTIVE",
        ),
        (
            PRIVATE_DOCUMENT_ID,
            "00000000000000000000000000000002",
            "Private Architecture",
            "ACTIVE",
        ),
        (
            TRASH_DOCUMENT_ID,
            "00000000000000000000000000000003",
            "Trashed Decision",
            "TRASHED",
        ),
    ] {
        sqlx::query("INSERT INTO documents(id,workspace_id,rank,title,status,trashed_at,purge_after,created_by) VALUES($1,$2,$3,$4,$5::document_status,CASE WHEN $5='TRASHED' THEN $6 ELSE NULL END,CASE WHEN $5='TRASHED' THEN $7 ELSE NULL END,$8)")
            .bind(id).bind(WORKSPACE_ID).bind(rank).bind(title).bind(status).bind(now - Duration::days(1))
            .bind(now + Duration::days(29)).bind(OWNER_ID).execute(&mut *tx).await?;
    }
    for (grant_id, document_id, subject_id, access, manage) in [
        (
            uuid!("70000000-0000-7000-8000-000000000040"),
            DOCUMENT_ID,
            OWNER_ID,
            "EDITOR",
            true,
        ),
        (
            uuid!("70000000-0000-7000-8000-000000000041"),
            DOCUMENT_ID,
            MEMBER_ID,
            "EDITOR",
            false,
        ),
        (
            uuid!("70000000-0000-7000-8000-000000000042"),
            PRIVATE_DOCUMENT_ID,
            OWNER_ID,
            "EDITOR",
            true,
        ),
        (
            uuid!("70000000-0000-7000-8000-000000000043"),
            TRASH_DOCUMENT_ID,
            OWNER_ID,
            "EDITOR",
            true,
        ),
    ] {
        sqlx::query("INSERT INTO permission_grants(id,workspace_id,document_id,subject_kind,subject_id,access,can_manage,granted_by) VALUES($1,$2,$3,'USER',$4,$5::document_access,$6,$7)")
            .bind(grant_id).bind(WORKSPACE_ID).bind(document_id).bind(subject_id).bind(access).bind(manage).bind(OWNER_ID)
            .execute(&mut *tx).await?;
    }
    sqlx::query("INSERT INTO published_versions(id,workspace_id,document_id,number,content_json,schema_version,content_fingerprint,based_on_version_id,source_draft_revision,publisher_id,summary,published_at) VALUES($1,$2,$3,1,$4,1,$5,NULL,7,$6,'Initial browser baseline',$7)")
        .bind(VERSION_ID).bind(WORKSPACE_ID).bind(DOCUMENT_ID).bind(&content).bind(&fingerprint).bind(OWNER_ID).bind(visual_epoch - Duration::hours(1))
        .execute(&mut *tx).await?;
    sqlx::query("INSERT INTO version_context(version_id,review_snapshot_json,discussion_ids,source_revision) VALUES($1,'{}','{}',7)")
        .bind(VERSION_ID).execute(&mut *tx).await?;
    sqlx::query("UPDATE documents SET current_version_id=$1 WHERE id=$2")
        .bind(VERSION_ID)
        .bind(DOCUMENT_ID)
        .execute(&mut *tx)
        .await?;
    sqlx::query("INSERT INTO drafts(id,workspace_id,document_id,base_version_id,content_json,schema_version,revision,updated_by) VALUES($1,$2,$3,$4,$5,1,7,$6)")
        .bind(DRAFT_ID).bind(WORKSPACE_ID).bind(DOCUMENT_ID).bind(VERSION_ID).bind(&content).bind(OWNER_ID)
        .execute(&mut *tx).await?;
    let public_hash: [u8; 32] = Sha256::digest(PUBLIC_TOKEN.as_bytes()).into();
    sqlx::query("INSERT INTO public_links(id,workspace_id,document_id,token_hash,created_by) VALUES($1,$2,$3,$4,$5)")
        .bind(PUBLIC_LINK_ID).bind(WORKSPACE_ID).bind(DOCUMENT_ID).bind(public_hash.as_slice()).bind(OWNER_ID)
        .execute(&mut *tx).await?;
    tx.commit().await?;
    pool.close().await;
    println!(
        "{}",
        serde_json::to_string(&FixtureOutput {
            schema_version: 1,
            run_id,
            workspace_id: WORKSPACE_ID,
            workspace_slug: slug,
            document_id: DOCUMENT_ID,
            private_document_id: PRIVATE_DOCUMENT_ID,
            trash_document_id: TRASH_DOCUMENT_ID,
            owner_user_id: OWNER_ID,
            owner_session_token: OWNER_TOKEN,
            member_user_id: MEMBER_ID,
            member_session_token: MEMBER_TOKEN,
            public_token: PUBLIC_TOKEN,
        })?
    );
    Ok(())
}

fn secret(value_key: &str, file_key: &str) -> anyhow::Result<String> {
    if let Ok(value) = env::var(value_key) {
        return Ok(value);
    }
    let path =
        env::var(file_key).with_context(|| format!("{value_key} or {file_key} is required"))?;
    Ok(fs::read_to_string(path)?.trim().to_owned())
}
