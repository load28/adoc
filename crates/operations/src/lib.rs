#![forbid(unsafe_code)]

//! File, audit, and retention bounded context.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FileStatus {
    Uploading,
    Validating,
    Ready,
    Failed,
    Deleted,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileAsset {
    pub id: Uuid,
    pub original_name: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub checksum_sha256: String,
    pub status: FileStatus,
    pub failure_code: Option<String>,
    pub ready_at: Option<DateTime<Utc>>,
    pub revision: i64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileUpload {
    pub asset_id: Uuid,
    pub upload_url: String,
    pub upload_token: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ByteRange {
    pub start: u64,
    pub end_inclusive: u64,
}
impl ByteRange {
    pub fn parse(value: &str, size: u64) -> Option<Self> {
        let raw = value.strip_prefix("bytes=")?;
        if raw.contains(',') || size == 0 {
            return None;
        }
        let (left, right) = raw.split_once('-')?;
        let (start, end) = if left.is_empty() {
            let suffix = right.parse::<u64>().ok()?;
            if suffix == 0 {
                return None;
            }
            (size.saturating_sub(suffix), size - 1)
        } else {
            let start = left.parse::<u64>().ok()?;
            let end = if right.is_empty() {
                size - 1
            } else {
                right.parse::<u64>().ok()?
            };
            (start, end)
        };
        (start <= end && end < size).then_some(Self {
            start,
            end_inclusive: end,
        })
    }
    pub fn len(self) -> u64 {
        self.end_inclusive - self.start + 1
    }
    pub fn is_empty(self) -> bool {
        false
    }
}

pub fn sanitize_filename(value: &str) -> Option<String> {
    let cleaned = value
        .chars()
        .filter(|c| !c.is_control())
        .map(|c| if matches!(c, '/' | '\\') { '�' } else { c })
        .collect::<String>();
    let cleaned = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
    (cleaned.chars().count() >= 1
        && cleaned.chars().count() <= 500
        && cleaned != "."
        && cleaned != "..")
        .then_some(cleaned)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ranges_and_names_are_bounded() {
        assert_eq!(ByteRange::parse("bytes=2-4", 10).unwrap().len(), 3);
        assert_eq!(ByteRange::parse("bytes=-3", 10).unwrap().start, 7);
        assert!(ByteRange::parse("bytes=4-2", 10).is_none());
        assert_eq!(sanitize_filename("../a\n.pdf").unwrap(), "..�a.pdf");
    }
}
