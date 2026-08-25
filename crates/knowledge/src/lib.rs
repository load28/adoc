#![forbid(unsafe_code)]

//! Knowledge bounded context.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use unicode_normalization::UnicodeNormalization;
use uuid::Uuid;

pub const SEARCH_PROJECTION_SCHEMA: i32 = 1;
pub const TOMBSTONE_REGION_ID: Uuid = Uuid::nil();

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SearchSourceKind {
    Published,
    Draft,
}

impl SearchSourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Published => "PUBLISHED",
            Self::Draft => "DRAFT",
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct PermissionPathNode {
    pub document_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub permission_revision: i64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SearchProjection {
    pub projection_schema: i32,
    pub workspace_id: Uuid,
    pub document_id: Uuid,
    pub document_status: String,
    pub source_kind: String,
    pub source_revision: i64,
    pub version_number: Option<i64>,
    pub region_id: Uuid,
    pub region_kind: String,
    pub ancestor_ids: Vec<Uuid>,
    pub title: String,
    pub body: String,
    pub terms: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding: Option<Vec<f32>>,
    pub permission_scope: String,
    pub permission_fingerprint: String,
    pub snapshot_hash: String,
    pub authority: String,
    pub updated_at: DateTime<Utc>,
    pub outbox_sequence: i64,
    pub deleted: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExtractedRegion {
    pub id: Uuid,
    pub kind: String,
    pub body: String,
}

#[must_use]
pub fn projection_id(
    workspace_id: Uuid,
    source_kind: SearchSourceKind,
    document_id: Uuid,
    region_id: Uuid,
) -> String {
    digest(format!(
        "{workspace_id}:{}:{document_id}:{region_id}",
        source_kind.as_str()
    ))
}

#[must_use]
pub fn permission_scope_token(workspace_id: Uuid, document_id: Uuid) -> String {
    digest(format!("scope:v1:{workspace_id}:{document_id}"))
}

pub fn permission_fingerprint(path: &[PermissionPathNode]) -> Option<String> {
    if path.is_empty()
        || path.iter().any(|node| node.permission_revision < 0)
        || path
            .windows(2)
            .any(|nodes| nodes[1].parent_id != Some(nodes[0].document_id))
    {
        return None;
    }
    serde_json::to_vec(path).ok().map(digest)
}

pub fn extract_search_regions(content: &Value) -> Option<Vec<ExtractedRegion>> {
    if content.get("schemaVersion").and_then(Value::as_i64) != Some(1) {
        return None;
    }
    let children = content
        .get("root")?
        .as_object()?
        .get("children")?
        .as_array()?;
    children
        .iter()
        .map(|block| {
            let id = block
                .get("id")
                .and_then(Value::as_str)
                .and_then(|value| Uuid::parse_str(value).ok())?;
            let kind = block.get("type").and_then(Value::as_str)?.to_owned();
            let mut fragments = Vec::new();
            collect_text(block, &mut fragments)?;
            Some(ExtractedRegion {
                id,
                kind,
                body: fragments
                    .join(" ")
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" "),
            })
        })
        .collect()
}

#[must_use]
pub fn snapshot_hash(content: &Value) -> String {
    digest(serde_json::to_vec(content).unwrap_or_default())
}

fn collect_text(value: &Value, output: &mut Vec<String>) -> Option<()> {
    match value {
        Value::Array(values) => {
            for value in values {
                collect_text(value, output)?;
            }
        }
        Value::Object(object) => {
            if object.get("type").and_then(Value::as_str) == Some("text") {
                output.push(object.get("text")?.as_str()?.to_owned());
            } else if object.get("type").and_then(Value::as_str) == Some("hardBreak") {
                output.push("\n".to_owned());
            } else if object.get("type").and_then(Value::as_str) == Some("codeBlock") {
                output.push(object.get("text")?.as_str()?.to_owned());
            }
            for key in ["summary", "children"] {
                if let Some(child) = object.get(key) {
                    collect_text(child, output)?;
                }
            }
        }
        _ => {}
    }
    Some(())
}

fn digest(value: impl AsRef<[u8]>) -> String {
    hex::encode(Sha256::digest(value.as_ref()))
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Reference {
    pub id: Uuid,
    pub source_document_id: Uuid,
    pub source_region: Value,
    pub target: Value,
    pub snapshot: ReferenceDisplaySnapshot,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceDisplaySnapshot {
    pub title: String,
    pub snapshot_hash: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferencePage {
    pub items: Vec<Reference>,
    pub next_cursor: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VocabularyStatus {
    Active,
    Deprecated,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VocabularyTermKind {
    Canonical,
    Synonym,
    Prohibited,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VocabularyTerm {
    pub term: String,
    pub kind: VocabularyTermKind,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VocabularyConcept {
    pub id: Uuid,
    pub canonical_term: String,
    pub definition: String,
    pub terms: Vec<VocabularyTerm>,
    pub status: VocabularyStatus,
    pub replacement_concept_id: Option<Uuid>,
    pub revision: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VocabularyPage {
    pub items: Vec<VocabularyConcept>,
    pub next_cursor: Option<String>,
}

pub fn normalize_term(value: &str) -> Option<(String, String)> {
    let display = value.nfc().collect::<String>();
    let display = display.split_whitespace().collect::<Vec<_>>().join(" ");
    if !(1..=200).contains(&display.chars().count()) {
        return None;
    }
    let normalized = display
        .chars()
        .flat_map(char::to_lowercase)
        .collect::<String>();
    Some((display, normalized))
}

pub fn normalize_terms(
    canonical: &str,
    terms: Vec<VocabularyTerm>,
) -> Option<(String, Vec<(VocabularyTerm, String)>)> {
    let (canonical, canonical_key) = normalize_term(canonical)?;
    let mut values = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    let mut canonical_count = 0;
    for item in terms {
        let (term, key) = normalize_term(&item.term)?;
        if !seen.insert(key.clone()) {
            return None;
        }
        if item.kind == VocabularyTermKind::Canonical {
            canonical_count += 1;
            if key != canonical_key {
                return None;
            }
        }
        values.push((
            VocabularyTerm {
                term,
                kind: item.kind,
            },
            key,
        ));
    }
    (canonical_count == 1 && !values.is_empty() && values.len() <= 100)
        .then_some((canonical, values))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn terms_use_nfc_case_and_whitespace_key() {
        let left = normalize_term("  CAFÉ\n정책 ").unwrap();
        let right = normalize_term("cafe\u{301} 정책").unwrap();
        assert_eq!(left.1, right.1);
        assert_eq!(left.0, "CAFÉ 정책");
    }

    #[test]
    fn search_regions_preserve_top_level_identity_and_nested_text() {
        let first = Uuid::from_u128(1);
        let nested = Uuid::from_u128(2);
        let content = serde_json::json!({"schemaVersion":1,"root":{"type":"doc","children":[
            {"id":first,"type":"paragraph","children":[{"type":"text","text":"hello"},{"type":"hardBreak"},{"type":"text","text":"world"}]},
            {"id":nested,"type":"toggle","summary":[{"type":"text","text":"details"}],"children":[{"id":Uuid::from_u128(3),"type":"codeBlock","text":"let x = 1;"}]}
        ]}});
        let regions = extract_search_regions(&content).unwrap();
        assert_eq!(regions.len(), 2);
        assert_eq!(regions[0].id, first);
        assert_eq!(regions[0].body, "hello world");
        assert_eq!(regions[1].body, "details let x = 1;");
    }

    #[test]
    fn permission_identity_is_deterministic_and_parent_sensitive() {
        let workspace = Uuid::from_u128(1);
        let root = Uuid::from_u128(2);
        let child = Uuid::from_u128(3);
        assert_eq!(
            permission_scope_token(workspace, child),
            permission_scope_token(workspace, child)
        );
        let path = vec![
            PermissionPathNode {
                document_id: root,
                parent_id: None,
                permission_revision: 1,
            },
            PermissionPathNode {
                document_id: child,
                parent_id: Some(root),
                permission_revision: 2,
            },
        ];
        let fingerprint = permission_fingerprint(&path).unwrap();
        let mut changed = path.clone();
        changed[0].permission_revision += 1;
        assert_ne!(fingerprint, permission_fingerprint(&changed).unwrap());
        changed[1].parent_id = None;
        assert!(permission_fingerprint(&changed).is_none());
    }
}
