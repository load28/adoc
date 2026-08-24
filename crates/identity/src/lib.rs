#![forbid(unsafe_code)]

//! Pure identity, preference and session lifetime rules.

use std::{fmt, str::FromStr};

use chrono::{DateTime, Duration, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

pub const GOOGLE_ISSUER: &str = "https://accounts.google.com";
pub const SESSION_IDLE_HOURS: i64 = 12;
pub const SESSION_ABSOLUTE_DAYS: i64 = 30;
pub const LOGIN_FLOW_MINUTES: i64 = 10;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Locale {
    Ko,
    En,
}

impl Locale {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ko => "ko",
            Self::En => "en",
        }
    }
}

impl FromStr for Locale {
    type Err = IdentityValidationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "ko" => Ok(Self::Ko),
            "en" => Ok(Self::En),
            _ => Err(IdentityValidationError::Locale),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Theme {
    Light,
    Dark,
    System,
}

impl Theme {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Light => "LIGHT",
            Self::Dark => "DARK",
            Self::System => "SYSTEM",
        }
    }
}

impl FromStr for Theme {
    type Err = IdentityValidationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "LIGHT" => Ok(Self::Light),
            "DARK" => Ok(Self::Dark),
            "SYSTEM" => Ok(Self::System),
            _ => Err(IdentityValidationError::Theme),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedEmail(String);

impl VerifiedEmail {
    pub fn parse(value: &str) -> Result<Self, IdentityValidationError> {
        let value = value.trim();
        if value.is_empty() || value.len() > 320 || value.chars().any(char::is_control) {
            return Err(IdentityValidationError::Email);
        }
        let (local, domain) = value
            .rsplit_once('@')
            .ok_or(IdentityValidationError::Email)?;
        let valid_domain = !domain.is_empty()
            && domain.is_ascii()
            && domain.split('.').all(|label| {
                !label.is_empty()
                    && !label.starts_with('-')
                    && !label.ends_with('-')
                    && label
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
            });
        if local.is_empty() || local.contains('@') || !valid_domain {
            return Err(IdentityValidationError::Email);
        }
        Ok(Self(format!("{local}@{}", domain.to_ascii_lowercase())))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisplayName(String);

impl DisplayName {
    pub fn parse(value: &str) -> Result<Self, IdentityValidationError> {
        let value = value.trim();
        if value.is_empty() || value.chars().count() > 200 || value.chars().any(char::is_control) {
            return Err(IdentityValidationError::DisplayName);
        }
        Ok(Self(value.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReturnPath(String);

impl ReturnPath {
    pub fn parse(value: Option<&str>) -> Result<Self, IdentityValidationError> {
        let value = value.unwrap_or("/");
        let valid = !value.is_empty()
            && value.len() <= 2048
            && value.starts_with('/')
            && !value.starts_with("//")
            && !value.contains('\\')
            && !value.chars().any(char::is_control);
        if valid {
            Ok(Self(value.to_owned()))
        } else {
            Err(IdentityValidationError::ReturnPath)
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedExternalIdentity {
    pub issuer: String,
    pub subject: String,
    pub email: VerifiedEmail,
    pub display_name: DisplayName,
}

impl VerifiedExternalIdentity {
    pub fn google(
        issuer: &str,
        subject: &str,
        email: &str,
        display_name: &str,
    ) -> Result<Self, IdentityValidationError> {
        if issuer != GOOGLE_ISSUER {
            return Err(IdentityValidationError::Issuer);
        }
        if subject.is_empty() || subject.len() > 255 || subject.chars().any(char::is_control) {
            return Err(IdentityValidationError::Subject);
        }
        Ok(Self {
            issuer: issuer.to_owned(),
            subject: subject.to_owned(),
            email: VerifiedEmail::parse(email)?,
            display_name: DisplayName::parse(display_name)?,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PreferenceInput {
    pub locale: Locale,
    pub timezone: String,
    pub theme: Theme,
}

impl PreferenceInput {
    pub fn validate(self) -> Result<Self, IdentityValidationError> {
        if self.timezone.len() > 100 || Tz::from_str(&self.timezone).is_err() {
            return Err(IdentityValidationError::Timezone);
        }
        Ok(self)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct UserPreferences {
    pub locale: Locale,
    pub timezone: String,
    pub theme: Theme,
    pub revision: i64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct UserSummary {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub locale: Locale,
    pub timezone: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SessionLifetime {
    pub created_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub idle_expires_at: DateTime<Utc>,
    pub absolute_expires_at: DateTime<Utc>,
}

impl SessionLifetime {
    pub fn new(now: DateTime<Utc>, idle: Duration) -> Self {
        Self {
            created_at: now,
            last_seen_at: now,
            idle_expires_at: now + idle,
            absolute_expires_at: now + Duration::days(SESSION_ABSOLUTE_DAYS),
        }
    }

    pub fn is_active(self, now: DateTime<Utc>) -> bool {
        now < self.idle_expires_at && now < self.absolute_expires_at
    }

    pub fn refreshed(self, now: DateTime<Utc>, idle: Duration) -> Self {
        let idle_expires_at = std::cmp::min(now + idle, self.absolute_expires_at);
        Self {
            last_seen_at: now,
            idle_expires_at,
            ..self
        }
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum IdentityValidationError {
    #[error("invalid identity issuer")]
    Issuer,
    #[error("invalid identity subject")]
    Subject,
    #[error("invalid verified email")]
    Email,
    #[error("invalid display name")]
    DisplayName,
    #[error("invalid locale")]
    Locale,
    #[error("invalid timezone")]
    Timezone,
    #[error("invalid theme")]
    Theme,
    #[error("invalid return path")]
    ReturnPath,
}

impl fmt::Display for Locale {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    #[test]
    fn return_path_rejects_cross_origin_and_ambiguous_forms() {
        for value in ["https://evil.example", "//evil.example", "/\\evil", "/x\n"] {
            assert!(
                ReturnPath::parse(Some(value)).is_err(),
                "accepted {value:?}"
            );
        }
        assert_eq!(ReturnPath::parse(None).unwrap().as_str(), "/");
        assert_eq!(
            ReturnPath::parse(Some("/w/alpha?mode=draft"))
                .unwrap()
                .as_str(),
            "/w/alpha?mode=draft"
        );
    }

    #[test]
    fn verified_identity_uses_subject_not_email_as_identity() {
        let identity = VerifiedExternalIdentity::google(
            GOOGLE_ISSUER,
            "subject-1",
            "Person@Example.COM",
            " Person ",
        )
        .unwrap();
        assert_eq!(identity.email.as_str(), "Person@example.com");
        assert_eq!(identity.display_name.as_str(), "Person");
    }

    #[test]
    fn preference_requires_real_iana_timezone() {
        assert!(
            PreferenceInput {
                locale: Locale::Ko,
                timezone: "Asia/Seoul".into(),
                theme: Theme::System,
            }
            .validate()
            .is_ok()
        );
        assert!(
            PreferenceInput {
                locale: Locale::En,
                timezone: "Mars/Olympus".into(),
                theme: Theme::Dark,
            }
            .validate()
            .is_err()
        );
    }

    #[test]
    fn session_idle_refresh_never_exceeds_absolute_expiry() {
        let now = Utc.with_ymd_and_hms(2026, 1, 15, 9, 0, 0).unwrap();
        let lifetime = SessionLifetime::new(now, Duration::hours(SESSION_IDLE_HOURS));
        let near_absolute = lifetime.absolute_expires_at - Duration::hours(1);
        let refreshed = lifetime.refreshed(near_absolute, Duration::hours(SESSION_IDLE_HOURS));
        assert_eq!(refreshed.idle_expires_at, lifetime.absolute_expires_at);
        assert!(!refreshed.is_active(lifetime.absolute_expires_at));
    }
}
