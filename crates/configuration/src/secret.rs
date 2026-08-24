use std::{fmt, fs, path::Path};

use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;

use crate::{ConfigError, Environment};

pub struct SecretValue(SecretString);

impl SecretValue {
    pub(crate) fn new(
        value: String,
        key: &'static str,
        minimum: usize,
    ) -> Result<Self, ConfigError> {
        if value.trim().len() < minimum {
            return Err(ConfigError::invalid(
                key,
                format!("secret must contain at least {minimum} characters"),
            ));
        }
        Ok(Self(SecretString::from(value)))
    }

    pub fn expose(&self) -> &str {
        self.0.expose_secret()
    }
}

impl fmt::Debug for SecretValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SecretValue([REDACTED])")
    }
}

impl fmt::Display for SecretValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("[REDACTED]")
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecretMetadata {
    pub source_key: &'static str,
    pub key_ids: Vec<String>,
}

#[derive(Debug)]
pub struct LoadedSecret {
    pub value: SecretValue,
    pub metadata: SecretMetadata,
}

#[derive(Debug)]
pub struct RotatingSecret {
    current_id: String,
    current: SecretValue,
    previous: Option<(String, SecretValue)>,
    metadata: SecretMetadata,
}

impl RotatingSecret {
    pub fn current_id(&self) -> &str {
        &self.current_id
    }

    pub fn current(&self) -> &SecretValue {
        &self.current
    }

    pub fn previous(&self) -> Option<(&str, &SecretValue)> {
        self.previous
            .as_ref()
            .map(|(id, value)| (id.as_str(), value))
    }

    pub fn metadata(&self) -> &SecretMetadata {
        &self.metadata
    }
}

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

pub(crate) fn load_secret(
    path: &Path,
    source_key: &'static str,
    environment: Environment,
    minimum: usize,
) -> Result<LoadedSecret, ConfigError> {
    validate_permissions(path, source_key, environment)?;
    let value =
        fs::read_to_string(path).map_err(|error| ConfigError::secret_file(source_key, error))?;
    Ok(LoadedSecret {
        value: SecretValue::new(value.trim_end().to_owned(), source_key, minimum)?,
        metadata: SecretMetadata {
            source_key,
            key_ids: Vec::new(),
        },
    })
}

pub(crate) fn load_rotating_secret(
    path: &Path,
    source_key: &'static str,
    environment: Environment,
) -> Result<RotatingSecret, ConfigError> {
    validate_permissions(path, source_key, environment)?;
    let bytes = fs::read(path).map_err(|error| ConfigError::secret_file(source_key, error))?;
    let raw: RotatingSecretFile = serde_json::from_slice(&bytes).map_err(|_| {
        ConfigError::invalid(
            source_key,
            "secret file must match the rotating JSON contract",
        )
    })?;
    validate_key_id(source_key, &raw.current.id)?;
    let current = SecretValue::new(raw.current.value, source_key, 32)?;
    let previous = if let Some(previous) = raw.previous {
        validate_key_id(source_key, &previous.id)?;
        if previous.id == raw.current.id {
            return Err(ConfigError::invalid(
                source_key,
                "current and previous key IDs must differ",
            ));
        }
        Some((
            previous.id,
            SecretValue::new(previous.value, source_key, 32)?,
        ))
    } else {
        None
    };
    let mut key_ids = vec![raw.current.id.clone()];
    if let Some((id, _)) = &previous {
        key_ids.push(id.clone());
    }
    Ok(RotatingSecret {
        current_id: raw.current.id,
        current,
        previous,
        metadata: SecretMetadata {
            source_key,
            key_ids,
        },
    })
}

fn validate_key_id(key: &'static str, id: &str) -> Result<(), ConfigError> {
    let valid = (1..=64).contains(&id.len())
        && id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'));
    if valid {
        Ok(())
    } else {
        Err(ConfigError::invalid(
            key,
            "key ID must match [A-Za-z0-9._-]{1,64}",
        ))
    }
}

#[cfg(unix)]
fn validate_permissions(
    path: &Path,
    key: &'static str,
    environment: Environment,
) -> Result<(), ConfigError> {
    use std::os::unix::fs::PermissionsExt;

    let metadata = fs::metadata(path).map_err(|error| ConfigError::secret_file(key, error))?;
    if !metadata.is_file() {
        return Err(ConfigError::invalid(
            key,
            "secret source must be a regular file",
        ));
    }
    if environment == Environment::Production && metadata.permissions().mode() & 0o077 != 0 {
        return Err(ConfigError::invalid(
            key,
            "production secret file must not be group/world accessible",
        ));
    }
    Ok(())
}

#[cfg(not(unix))]
fn validate_permissions(
    path: &Path,
    key: &'static str,
    _environment: Environment,
) -> Result<(), ConfigError> {
    if fs::metadata(path)
        .map_err(|error| ConfigError::secret_file(key, error))?
        .is_file()
    {
        Ok(())
    } else {
        Err(ConfigError::invalid(
            key,
            "secret source must be a regular file",
        ))
    }
}
