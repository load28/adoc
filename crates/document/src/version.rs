use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use thiserror::Error;
use uuid::Uuid;

use crate::model::OperationBase;
use crate::{DocumentOperation, OperationPrecondition, OperationScope, ValidatedContent};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishedVersion {
    pub id: Uuid,
    pub document_id: Uuid,
    pub number: i64,
    pub content: Value,
    pub schema_version: i32,
    pub content_fingerprint: String,
    pub based_on_version_id: Option<Uuid>,
    pub source_draft_revision: i64,
    pub publisher_id: Uuid,
    pub summary: String,
    pub published_at: DateTime<Utc>,
    pub review_snapshot: Value,
    pub discussion_ids: Vec<Uuid>,
}

impl PublishedVersion {
    pub fn validate_snapshot(&self) -> bool {
        self.number > 0
            && self.source_draft_revision >= 0
            && ValidatedContent::parse(self.content.clone())
                .map(|content| {
                    crate::canonical_hash(content.as_value()) == self.content_fingerprint
                })
                .unwrap_or(false)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionPage {
    pub items: Vec<PublishedVersion>,
    pub next_cursor: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentDiff {
    pub from_version_id: Uuid,
    pub to_version_id: Uuid,
    pub operations: Vec<DocumentOperation>,
}

#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum DiffError {
    #[error("versions do not belong to the same document")]
    DifferentDocument,
    #[error("version content cannot produce a structural diff")]
    InvalidContent,
}

pub fn structural_diff(
    from: &PublishedVersion,
    to: &PublishedVersion,
) -> Result<DocumentDiff, DiffError> {
    if from.document_id != to.document_id {
        return Err(DiffError::DifferentDocument);
    }
    let operations = if from.content_fingerprint == to.content_fingerprint {
        Vec::new()
    } else {
        let blocks = to
            .content
            .pointer("/root/children")
            .and_then(Value::as_array)
            .cloned()
            .ok_or(DiffError::InvalidContent)?;
        let digest = Sha256::digest(
            json!({"from":from.id,"to":to.id,"fingerprint":to.content_fingerprint})
                .to_string()
                .as_bytes(),
        );
        let mut bytes = [0_u8; 16];
        bytes.copy_from_slice(&digest[..16]);
        bytes[6] = (bytes[6] & 0x0f) | 0x50;
        bytes[8] = (bytes[8] & 0x3f) | 0x80;
        let base = OperationBase {
            op_id: Uuid::from_bytes(bytes),
            scope: OperationScope::Document,
            precondition: OperationPrecondition {
                draft_revision: 0,
                target_hash: Some(from.content_fingerprint.clone()),
            },
            depends_on: Vec::new(),
        };
        vec![DocumentOperation::ReplaceRegion {
            base,
            region: OperationScope::Document,
            blocks,
        }]
    };
    Ok(DocumentDiff {
        from_version_id: from.id,
        to_version_id: to.id,
        operations,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn version(id: Uuid, document_id: Uuid, text: &str) -> PublishedVersion {
        let block = Uuid::now_v7();
        let content = ValidatedContent::parse(json!({
            "schemaVersion": 1,
            "root": {"type":"doc","children":[{"id":block,"type":"paragraph","children":[{"type":"text","text":text}]}]}
        }))
        .unwrap()
        .into_value();
        PublishedVersion {
            id,
            document_id,
            number: 1,
            content_fingerprint: crate::canonical_hash(&content),
            content,
            schema_version: 1,
            based_on_version_id: None,
            source_draft_revision: 0,
            publisher_id: Uuid::nil(),
            summary: "summary".into(),
            published_at: Utc::now(),
            review_snapshot: json!({}),
            discussion_ids: Vec::new(),
        }
    }

    #[test]
    fn diff_is_deterministic_and_uses_the_operation_contract() {
        let document = Uuid::now_v7();
        let from = version(Uuid::now_v7(), document, "before");
        let to = version(Uuid::now_v7(), document, "after");
        assert_eq!(structural_diff(&from, &to), structural_diff(&from, &to));
        assert_eq!(structural_diff(&from, &to).unwrap().operations.len(), 1);
    }
}
