#![forbid(unsafe_code)]

//! Knowledge bounded context.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use unicode_normalization::UnicodeNormalization;
use uuid::Uuid;

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
}
