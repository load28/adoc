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
    pub permission_key: String,
    pub snapshot_hash: String,
    pub authority: String,
    pub updated_at: DateTime<Utc>,
    pub outbox_sequence: i64,
    pub deleted: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SearchPermissionKey {
    pub document_id: Uuid,
    pub source_kind: SearchSourceKind,
    pub scope_token: String,
    pub ancestry_fingerprint: String,
    pub composite_key: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SearchHit {
    pub stable_id: String,
    pub source_kind: SearchSourceKind,
    pub document_id: Uuid,
    pub source_revision: i64,
    pub version_number: Option<i64>,
    pub region_id: Uuid,
    pub title: String,
    pub body: String,
    pub terms: Vec<String>,
    pub snapshot_hash: String,
    pub updated_at: DateTime<Utc>,
    pub outbox_sequence: i64,
    pub provider_score: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchDisplaySnapshot {
    pub title: String,
    pub excerpt: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchSource {
    pub kind: SearchSourceKind,
    pub stable_id: String,
    pub document_id: Uuid,
    pub region_id: Uuid,
    pub version: Option<i64>,
    pub draft_revision: Option<i64>,
    pub authority: String,
    pub snapshot_hash: String,
    pub display_snapshot: SearchDisplaySnapshot,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResultItem {
    pub source: SearchSource,
    pub score: f64,
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

#[must_use]
pub fn permission_composite_key(scope_token: &str, ancestry_fingerprint: &str) -> String {
    digest(format!(
        "permission-key:v1:{scope_token}:{ancestry_fingerprint}"
    ))
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

pub fn normalize_search_query(value: &str) -> Option<String> {
    let normalized = value.nfc().collect::<String>();
    let normalized = normalized.split_whitespace().collect::<Vec<_>>().join(" ");
    if !(1..=500).contains(&normalized.chars().count())
        || normalized.chars().any(forbidden_search_character)
    {
        return None;
    }
    Some(normalized)
}

#[must_use]
pub fn valid_query_vector(value: &[f32], dimension: usize) -> bool {
    dimension > 0 && value.len() == dimension && value.iter().all(|item| item.is_finite())
}

pub fn fuse_search_hits(
    lexical: Vec<SearchHit>,
    vector: Vec<SearchHit>,
    normalized_query: &str,
    now: DateTime<Utc>,
) -> Option<Vec<SearchResultItem>> {
    let mut fused = std::collections::BTreeMap::<String, (SearchHit, f64)>::new();
    add_modality(&mut fused, lexical)?;
    add_modality(&mut fused, vector)?;
    let exact_key = search_key(normalized_query);
    let mut deduped = std::collections::BTreeMap::<(Uuid, Uuid, String), SearchResultItem>::new();
    for (_, (hit, rrf)) in fused {
        let authority = match hit.source_kind {
            SearchSourceKind::Published if hit.version_number.is_some_and(|value| value > 0) => {
                "OFFICIAL"
            }
            SearchSourceKind::Draft if hit.version_number.is_none() && hit.source_revision >= 0 => {
                "WORKING"
            }
            _ => return None,
        };
        if hit.outbox_sequence <= 0 || hit.snapshot_hash.len() != 64 || hit.stable_id.len() != 64 {
            return None;
        }
        let exact = hit.terms.iter().any(|term| search_key(term) == exact_key);
        let age_seconds = now
            .signed_duration_since(hit.updated_at)
            .num_seconds()
            .max(0) as f64;
        let age_days = age_seconds / 86_400.0;
        let freshness = 0.002 * 0.5_f64.powf(age_days / 180.0);
        let score = rrf
            + if exact { 0.005 } else { 0.0 }
            + match hit.source_kind {
                SearchSourceKind::Published => 0.003,
                SearchSourceKind::Draft => 0.001,
            }
            + freshness;
        let key = (hit.document_id, hit.region_id, hit.snapshot_hash.clone());
        let item = SearchResultItem {
            source: SearchSource {
                kind: hit.source_kind,
                stable_id: hit.stable_id,
                document_id: hit.document_id,
                region_id: hit.region_id,
                version: hit.version_number,
                draft_revision: (hit.source_kind == SearchSourceKind::Draft)
                    .then_some(hit.source_revision),
                authority: authority.to_owned(),
                snapshot_hash: hit.snapshot_hash,
                display_snapshot: SearchDisplaySnapshot {
                    title: hit.title,
                    excerpt: search_excerpt(&hit.body, 500),
                    updated_at: hit.updated_at,
                },
            },
            score,
        };
        match deduped.get(&key) {
            Some(current)
                if current.score > item.score
                    || (current.score == item.score
                        && current.source.stable_id <= item.source.stable_id) => {}
            _ => {
                deduped.insert(key, item);
            }
        }
    }
    let mut result = deduped.into_values().collect::<Vec<_>>();
    result.sort_by(|left, right| {
        right
            .score
            .total_cmp(&left.score)
            .then_with(|| left.source.stable_id.cmp(&right.source.stable_id))
    });
    result.truncate(30);
    Some(result)
}

#[must_use]
pub fn search_excerpt(value: &str, limit: usize) -> String {
    let mut chars = value.chars();
    let prefix = chars.by_ref().take(limit).collect::<String>();
    if chars.next().is_some() {
        format!("{prefix}…")
    } else {
        prefix
    }
}

fn add_modality(
    fused: &mut std::collections::BTreeMap<String, (SearchHit, f64)>,
    mut hits: Vec<SearchHit>,
) -> Option<()> {
    if hits.iter().any(|hit| !hit.provider_score.is_finite()) {
        return None;
    }
    hits.sort_by(|left, right| {
        right
            .provider_score
            .total_cmp(&left.provider_score)
            .then_with(|| left.stable_id.cmp(&right.stable_id))
    });
    hits.dedup_by(|left, right| left.stable_id == right.stable_id);
    for (index, hit) in hits.into_iter().take(100).enumerate() {
        let increment = 1.0 / (60.0 + (index + 1) as f64);
        match fused.get_mut(&hit.stable_id) {
            Some((existing, score)) => {
                if !same_search_source(existing, &hit) {
                    return None;
                }
                *score += increment;
            }
            None => {
                fused.insert(hit.stable_id.clone(), (hit, increment));
            }
        }
    }
    Some(())
}

fn same_search_source(left: &SearchHit, right: &SearchHit) -> bool {
    left.source_kind == right.source_kind
        && left.document_id == right.document_id
        && left.source_revision == right.source_revision
        && left.version_number == right.version_number
        && left.region_id == right.region_id
        && left.snapshot_hash == right.snapshot_hash
        && left.outbox_sequence == right.outbox_sequence
}

fn search_key(value: &str) -> String {
    value
        .nfc()
        .flat_map(char::to_lowercase)
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn forbidden_search_character(value: char) -> bool {
    value.is_control()
        || matches!(
            value,
            '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}' | '\u{200e}' | '\u{200f}'
        )
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

    #[test]
    fn search_query_vector_and_permission_key_are_canonical() {
        assert_eq!(
            normalize_search_query("  CAFÉ\n정책  ").unwrap(),
            "CAFÉ 정책"
        );
        assert!(normalize_search_query("unsafe\u{202e}query").is_none());
        assert!(normalize_search_query("\n\t").is_none());
        assert!(valid_query_vector(&[0.0, 1.0], 2));
        assert!(!valid_query_vector(&[f32::NAN, 1.0], 2));
        let scope = "a".repeat(64);
        let fingerprint = "b".repeat(64);
        assert_eq!(
            permission_composite_key(&scope, &fingerprint),
            permission_composite_key(&scope, &fingerprint)
        );
    }

    #[test]
    fn rrf_fuses_modalities_and_returns_deterministic_source_snapshot() {
        let now = Utc::now();
        let first = search_hit("a", Uuid::from_u128(1), 4.0, now);
        let mut second = search_hit("b", Uuid::from_u128(2), 3.0, now);
        second.terms.clear();
        let result = fuse_search_hits(
            vec![first.clone(), second.clone()],
            vec![first, second],
            "정책",
            now,
        )
        .unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].source.region_id, Uuid::from_u128(1));
        assert_eq!(result[0].source.authority, "OFFICIAL");
        assert_eq!(result[0].source.version, Some(3));
        assert_eq!(result[0].source.display_snapshot.excerpt, "본문");
        assert!(result[0].score > result[1].score);
        assert_eq!(search_excerpt(&"x".repeat(501), 500).chars().count(), 501);
    }

    fn search_hit(
        identity: &str,
        region_id: Uuid,
        provider_score: f64,
        now: DateTime<Utc>,
    ) -> SearchHit {
        SearchHit {
            stable_id: identity.repeat(64),
            source_kind: SearchSourceKind::Published,
            document_id: Uuid::from_u128(9),
            source_revision: 3,
            version_number: Some(3),
            region_id,
            title: "정책".to_owned(),
            body: "본문".to_owned(),
            terms: vec!["정책".to_owned()],
            snapshot_hash: "c".repeat(64),
            updated_at: now,
            outbox_sequence: 4,
            provider_score,
        }
    }
}
