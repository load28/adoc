#![forbid(unsafe_code)]

//! Typed, fail-fast process configuration shared by Adoc binaries.

mod parser;
mod secret;

pub use parser::{
    AiConfig, AiDriver, AppConfig, AuthConfig, CommonConfig, ConfigError, ConfigSource,
    DependencyConfig, Environment, LogLevel, ObjectStorageDriver, ServiceKind, StorageConfig,
    WorkerConfig,
};
pub use secret::{LoadedSecret, RotatingSecret, SecretMetadata, SecretValue};
