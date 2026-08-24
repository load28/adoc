use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use uuid::Uuid;

const RANK_LENGTH: usize = 32;
const RADIX: u16 = 62;
const ALPHABET: &[u8; 62] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentTitle(String);

impl DocumentTitle {
    pub fn parse(value: &str) -> Result<Self, TreeValidationError> {
        let normalized = value.trim();
        if normalized.is_empty()
            || normalized.chars().count() > 500
            || normalized.chars().any(|character| {
                character.is_control()
                    || matches!(
                        character,
                        '\u{202A}'
                            | '\u{202B}'
                            | '\u{202C}'
                            | '\u{202D}'
                            | '\u{202E}'
                            | '\u{2066}'
                            | '\u{2067}'
                            | '\u{2068}'
                            | '\u{2069}'
                    )
            })
        {
            return Err(TreeValidationError::InvalidTitle);
        }
        Ok(Self(normalized.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DocumentStatus {
    Active,
    Trashed,
    Purging,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    pub id: Uuid,
    pub title: String,
    pub parent_id: Option<Uuid>,
    pub status: DocumentStatus,
    pub current_version_id: Option<Uuid>,
    pub revision: i64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Draft {
    pub id: Uuid,
    pub document_id: Uuid,
    pub base_version_id: Option<Uuid>,
    pub revision: i64,
    pub schema_version: i32,
    pub content_fingerprint: String,
    pub content: Value,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct TreeRank([u8; RANK_LENGTH]);

impl TreeRank {
    pub fn parse(value: &str) -> Result<Self, TreeValidationError> {
        if value.len() != RANK_LENGTH {
            return Err(TreeValidationError::InvalidRank);
        }
        let mut digits = [0_u8; RANK_LENGTH];
        for (index, byte) in value.bytes().enumerate() {
            digits[index] = ALPHABET
                .iter()
                .position(|candidate| *candidate == byte)
                .and_then(|position| u8::try_from(position).ok())
                .ok_or(TreeValidationError::InvalidRank)?;
        }
        let rank = Self(digits);
        if rank == Self::lower_sentinel() || rank == Self::upper_sentinel() {
            return Err(TreeValidationError::InvalidRank);
        }
        Ok(rank)
    }

    pub fn between(after: Option<&Self>, before: Option<&Self>) -> Option<Self> {
        let lower = after.cloned().unwrap_or_else(Self::lower_sentinel);
        let upper = before.cloned().unwrap_or_else(Self::upper_sentinel);
        if lower >= upper {
            return None;
        }
        let midpoint = midpoint(&lower.0, &upper.0);
        let candidate = Self(midpoint);
        (candidate > lower && candidate < upper).then_some(candidate)
    }

    pub fn rebalance(count: usize) -> Result<Vec<Self>, TreeValidationError> {
        if count == 0 {
            return Ok(Vec::new());
        }
        let divisor = count
            .checked_add(1)
            .ok_or(TreeValidationError::RankSpaceExhausted)?;
        let step = divide_digits([61_u8; RANK_LENGTH], divisor);
        if step.iter().all(|digit| *digit == 0) {
            return Err(TreeValidationError::RankSpaceExhausted);
        }
        (1..=count)
            .map(|factor| {
                let digits = multiply_digits(step, factor)?;
                let rank = Self(digits);
                if rank == Self::upper_sentinel() {
                    return Err(TreeValidationError::RankSpaceExhausted);
                }
                Ok(rank)
            })
            .collect()
    }

    fn lower_sentinel() -> Self {
        Self([0; RANK_LENGTH])
    }

    fn upper_sentinel() -> Self {
        Self([61; RANK_LENGTH])
    }
}

impl std::fmt::Display for TreeRank {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = self
            .0
            .iter()
            .map(|digit| char::from(ALPHABET[usize::from(*digit)]))
            .collect::<String>();
        formatter.write_str(&value)
    }
}

fn midpoint(lower: &[u8; RANK_LENGTH], upper: &[u8; RANK_LENGTH]) -> [u8; RANK_LENGTH] {
    let mut sum = [0_u16; RANK_LENGTH + 1];
    let mut carry = 0_u16;
    for index in (0..RANK_LENGTH).rev() {
        let value = u16::from(lower[index]) + u16::from(upper[index]) + carry;
        sum[index + 1] = value % RADIX;
        carry = value / RADIX;
    }
    sum[0] = carry;
    let mut result = [0_u8; RANK_LENGTH];
    let mut remainder = 0_u16;
    for (index, digit) in sum.into_iter().enumerate() {
        let value = remainder * RADIX + digit;
        if index > 0 {
            result[index - 1] = u8::try_from(value / 2).unwrap_or(0);
        }
        remainder = value % 2;
    }
    result
}

fn divide_digits(mut digits: [u8; RANK_LENGTH], divisor: usize) -> [u8; RANK_LENGTH] {
    let mut remainder = 0_usize;
    for digit in &mut digits {
        let value = remainder * usize::from(RADIX) + usize::from(*digit);
        *digit = u8::try_from(value / divisor).unwrap_or(0);
        remainder = value % divisor;
    }
    digits
}

fn multiply_digits(
    digits: [u8; RANK_LENGTH],
    factor: usize,
) -> Result<[u8; RANK_LENGTH], TreeValidationError> {
    let mut output = [0_u8; RANK_LENGTH];
    let mut carry = 0_usize;
    for index in (0..RANK_LENGTH).rev() {
        let value = usize::from(digits[index]) * factor + carry;
        output[index] = u8::try_from(value % usize::from(RADIX))
            .map_err(|_| TreeValidationError::RankSpaceExhausted)?;
        carry = value / usize::from(RADIX);
    }
    if carry != 0 {
        return Err(TreeValidationError::RankSpaceExhausted);
    }
    Ok(output)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EditLease {
    pub document_id: Uuid,
    pub holder_user_id: Uuid,
    pub client_instance_id: Uuid,
    pub expires_at: DateTime<Utc>,
    pub revision: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LeaseDecision {
    Acquire,
    ForceTakeover,
}

pub fn validate_lease_acquire(
    current: Option<&EditLease>,
    actor: Uuid,
    client: Uuid,
    now: DateTime<Utc>,
    force: bool,
    can_manage: bool,
    reason: Option<&str>,
) -> Result<LeaseDecision, TreeValidationError> {
    let available = current.is_none_or(|lease| lease.expires_at <= now);
    if available {
        return Ok(LeaseDecision::Acquire);
    }
    let current = current.ok_or(TreeValidationError::LeaseHeld)?;
    if current.holder_user_id == actor && current.client_instance_id == client {
        return Err(TreeValidationError::LeaseHeld);
    }
    if force && can_manage && reason.is_some_and(|value| !value.trim().is_empty()) {
        return Ok(LeaseDecision::ForceTakeover);
    }
    Err(TreeValidationError::LeaseHeld)
}

pub fn validate_lease_holder(
    lease: &EditLease,
    actor: Uuid,
    client: Uuid,
    expected_revision: i64,
    now: DateTime<Utc>,
) -> Result<(), TreeValidationError> {
    if lease.expires_at <= now {
        return Err(TreeValidationError::LeaseExpired);
    }
    if lease.holder_user_id != actor
        || lease.client_instance_id != client
        || lease.revision != expected_revision
    {
        return Err(TreeValidationError::LeaseInvalid);
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum TreeValidationError {
    #[error("invalid document title")]
    InvalidTitle,
    #[error("invalid tree rank")]
    InvalidRank,
    #[error("tree rank space exhausted")]
    RankSpaceExhausted,
    #[error("edit lease is held")]
    LeaseHeld,
    #[error("edit lease is invalid")]
    LeaseInvalid,
    #[error("edit lease expired")]
    LeaseExpired,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn rank_midpoint_and_rebalance_preserve_strict_lexical_order() {
        let middle = TreeRank::between(None, None).unwrap();
        assert_eq!(middle.to_string().len(), 32);
        let ranks = TreeRank::rebalance(500).unwrap();
        assert_eq!(ranks.len(), 500);
        assert!(ranks.windows(2).all(|window| window[0] < window[1]));
        assert!(TreeRank::between(Some(&ranks[10]), Some(&ranks[11])).is_some());
    }

    #[test]
    fn title_rejects_control_and_direction_override() {
        assert_eq!(DocumentTitle::parse("  title  ").unwrap().as_str(), "title");
        assert!(DocumentTitle::parse("bad\nname").is_err());
        assert!(DocumentTitle::parse("bad\u{202e}name").is_err());
    }

    #[test]
    fn lease_is_bound_to_user_client_revision_and_server_time() {
        let now = Utc::now();
        let actor = Uuid::from_u128(1);
        let client = Uuid::from_u128(2);
        let lease = EditLease {
            document_id: Uuid::from_u128(3),
            holder_user_id: actor,
            client_instance_id: client,
            expires_at: now + Duration::seconds(90),
            revision: 4,
        };
        assert!(validate_lease_holder(&lease, actor, client, 4, now).is_ok());
        assert_eq!(
            validate_lease_acquire(Some(&lease), actor, client, now, false, false, None),
            Err(TreeValidationError::LeaseHeld)
        );
        assert_eq!(
            validate_lease_acquire(
                Some(&lease),
                Uuid::from_u128(9),
                Uuid::from_u128(8),
                now,
                true,
                true,
                Some("takeover")
            ),
            Ok(LeaseDecision::ForceTakeover)
        );
    }
}
