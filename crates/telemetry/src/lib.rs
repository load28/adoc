#![forbid(unsafe_code)]

//! Structured telemetry wiring without domain policy or secret access.

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, Mutex},
};

use adoc_configuration::CommonConfig;
use serde_json::{Map, Value};
use thiserror::Error;
use tracing_subscriber::EnvFilter;

const REDACTED: &str = "[REDACTED]";
const FORBIDDEN_FIELD_PARTS: &[&str] = &[
    "authorization",
    "content",
    "cookie",
    "credential",
    "file_name",
    "password",
    "prompt",
    "query",
    "secret",
    "signed_url",
    "title",
    "token",
];
const KNOWN_METRICS: &[MetricDescriptor] = &[
    MetricDescriptor::new(
        "http_requests_total",
        &["method", "status", "code", "service"],
    ),
    MetricDescriptor::new("http_request_duration_ms", &["method", "status", "service"]),
    MetricDescriptor::new("db_pool_in_use", &["service"]),
    MetricDescriptor::new("db_transaction_retries_total", &["result", "service"]),
    MetricDescriptor::new("queue_depth", &["queue", "service"]),
    MetricDescriptor::new("queue_oldest_age_seconds", &["queue", "service"]),
    MetricDescriptor::new("outbox_lag_seconds", &["service"]),
    MetricDescriptor::new("search_index_watermark", &["source", "service"]),
    MetricDescriptor::new("search_requests_total", &["result", "service"]),
    MetricDescriptor::new("sse_connections", &["service"]),
    MetricDescriptor::new("sse_resets_total", &["reason", "service"]),
    MetricDescriptor::new("lease_conflicts_total", &["result", "service"]),
    MetricDescriptor::new("ai_usage_tokens_total", &["provider", "result", "service"]),
    MetricDescriptor::new("ai_quota_rejections_total", &["scope", "service"]),
    MetricDescriptor::new(
        "ai_first_progress_duration_ms",
        &["driver", "result", "service"],
    ),
    MetricDescriptor::new("file_validation_total", &["result", "service"]),
    MetricDescriptor::new("file_gc_oldest_age_seconds", &["service"]),
    MetricDescriptor::new("backup_age_seconds", &["service"]),
    MetricDescriptor::new("purge_oldest_age_seconds", &["kind", "service"]),
    MetricDescriptor::new(
        "permission_invariant_failures_total",
        &["boundary", "service"],
    ),
    MetricDescriptor::new(
        "provider_credential_failures_total",
        &["provider", "service"],
    ),
];

#[derive(Clone, Debug)]
pub struct TelemetryConfig {
    pub service: &'static str,
    pub version: String,
    pub log_level: &'static str,
    pub otel_endpoint_configured: bool,
}

impl From<&CommonConfig> for TelemetryConfig {
    fn from(config: &CommonConfig) -> Self {
        Self {
            service: config.service.as_str(),
            version: config.release_sha.clone(),
            log_level: config.log_level.as_str(),
            otel_endpoint_configured: config.otel_endpoint.is_some(),
        }
    }
}

#[derive(Debug, Error)]
pub enum TelemetryError {
    #[error("telemetry subscriber has already been initialized")]
    AlreadyInitialized,
    #[error("unknown metric: {0}")]
    UnknownMetric(String),
    #[error("metric {metric} does not permit label {label}")]
    UnknownLabel { metric: String, label: String },
    #[error("metric label value is unsafe for cardinality")]
    UnsafeCardinality,
    #[error("telemetry registry lock is unavailable")]
    RegistryUnavailable,
}

pub fn initialize(config: &TelemetryConfig) -> Result<(), TelemetryError> {
    tracing_subscriber::fmt()
        .json()
        .with_current_span(true)
        .with_span_list(true)
        .with_env_filter(EnvFilter::new(config.log_level))
        .try_init()
        .map_err(|_| TelemetryError::AlreadyInitialized)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraceContext {
    request_id: String,
    correlation_id: String,
    causation_id: Option<String>,
    provider_request_id: Option<String>,
}

impl TraceContext {
    pub fn new(
        request_id: impl Into<String>,
        correlation_id: impl Into<String>,
    ) -> Result<Self, TelemetryError> {
        let request_id = request_id.into();
        let correlation_id = correlation_id.into();
        if !safe_trace_id(&request_id) || !safe_trace_id(&correlation_id) {
            return Err(TelemetryError::UnsafeCardinality);
        }
        Ok(Self {
            request_id,
            correlation_id,
            causation_id: None,
            provider_request_id: None,
        })
    }

    pub fn causation_id(mut self, value: impl Into<String>) -> Result<Self, TelemetryError> {
        let value = value.into();
        if !safe_trace_id(&value) {
            return Err(TelemetryError::UnsafeCardinality);
        }
        self.causation_id = Some(value);
        Ok(self)
    }

    pub fn provider_request_id(mut self, value: impl Into<String>) -> Result<Self, TelemetryError> {
        let value = value.into();
        if !safe_trace_id(&value) {
            return Err(TelemetryError::UnsafeCardinality);
        }
        self.provider_request_id = Some(value);
        Ok(self)
    }

    pub fn span(&self) -> tracing::Span {
        tracing::info_span!(
            "request",
            request_id = self.request_id,
            correlation_id = self.correlation_id,
            causation_id = self.causation_id,
            provider_request_id = self.provider_request_id,
        )
    }
}

fn safe_trace_id(value: &str) -> bool {
    (8..=128).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'))
}

#[derive(Clone, Debug)]
pub struct SafeEvent {
    service: &'static str,
    version: String,
    code: String,
    duration_ms: Option<u64>,
    fields: Map<String, Value>,
}

impl SafeEvent {
    pub fn new(config: &TelemetryConfig, code: impl Into<String>) -> Self {
        Self {
            service: config.service,
            version: config.version.clone(),
            code: code.into(),
            duration_ms: None,
            fields: Map::new(),
        }
    }

    pub fn duration_ms(mut self, duration_ms: u64) -> Self {
        self.duration_ms = Some(duration_ms);
        self
    }

    pub fn field(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        let key = key.into();
        let value = if is_forbidden_field(&key) {
            Value::String(REDACTED.to_owned())
        } else {
            sanitize_value(value.into())
        };
        self.fields.insert(key, value);
        self
    }

    pub fn json(&self) -> Value {
        serde_json::json!({
            "service": self.service,
            "version": self.version,
            "code": self.code,
            "durationMs": self.duration_ms,
            "fields": self.fields,
        })
    }

    pub fn emit(self) {
        let fields = serde_json::Value::Object(self.fields);
        tracing::info!(
            service = self.service,
            version = self.version,
            code = self.code,
            duration_ms = self.duration_ms,
            fields = %fields,
        );
    }
}

fn is_forbidden_field(key: &str) -> bool {
    let normalized = key.to_ascii_lowercase().replace(['-', '.'], "_");
    FORBIDDEN_FIELD_PARTS
        .iter()
        .any(|part| normalized.contains(part))
}

fn sanitize_value(value: Value) -> Value {
    match value {
        Value::Object(object) => Value::Object(
            object
                .into_iter()
                .map(|(key, value)| {
                    let value = if is_forbidden_field(&key) {
                        Value::String(REDACTED.to_owned())
                    } else {
                        sanitize_value(value)
                    };
                    (key, value)
                })
                .collect(),
        ),
        Value::Array(values) => Value::Array(values.into_iter().map(sanitize_value).collect()),
        value => value,
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MetricDescriptor {
    pub name: &'static str,
    pub allowed_labels: &'static [&'static str],
}

impl MetricDescriptor {
    pub const fn new(name: &'static str, allowed_labels: &'static [&'static str]) -> Self {
        Self {
            name,
            allowed_labels,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct MetricRegistry {
    values: Arc<Mutex<BTreeMap<MetricKey, u64>>>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct MetricKey {
    name: String,
    labels: Vec<(String, String)>,
}

impl MetricRegistry {
    pub fn increment(
        &self,
        name: &str,
        labels: &[(&str, &str)],
        value: u64,
    ) -> Result<(), TelemetryError> {
        let descriptor = KNOWN_METRICS
            .iter()
            .find(|descriptor| descriptor.name == name)
            .ok_or_else(|| TelemetryError::UnknownMetric(name.to_owned()))?;
        let allowed: BTreeSet<_> = descriptor.allowed_labels.iter().copied().collect();
        let mut normalized = Vec::with_capacity(labels.len());
        for (label, value) in labels {
            if !allowed.contains(label) {
                return Err(TelemetryError::UnknownLabel {
                    metric: name.to_owned(),
                    label: (*label).to_owned(),
                });
            }
            if !safe_label_value(label, value) {
                return Err(TelemetryError::UnsafeCardinality);
            }
            normalized.push(((*label).to_owned(), (*value).to_owned()));
        }
        normalized.sort();
        let mut values = self
            .values
            .lock()
            .map_err(|_| TelemetryError::RegistryUnavailable)?;
        *values
            .entry(MetricKey {
                name: name.to_owned(),
                labels: normalized,
            })
            .or_default() += value;
        Ok(())
    }

    pub fn snapshot(&self) -> Result<BTreeMap<String, u64>, TelemetryError> {
        let values = self
            .values
            .lock()
            .map_err(|_| TelemetryError::RegistryUnavailable)?;
        Ok(values
            .iter()
            .map(|(key, value)| {
                let labels = key
                    .labels
                    .iter()
                    .map(|(name, value)| format!("{name}={value}"))
                    .collect::<Vec<_>>()
                    .join(",");
                (
                    if labels.is_empty() {
                        key.name.clone()
                    } else {
                        format!("{}{{{labels}}}", key.name)
                    },
                    *value,
                )
            })
            .collect())
    }
}

fn safe_label_value(label: &str, value: &str) -> bool {
    if value.is_empty() || value.len() > 64 || label.ends_with("_id") {
        return false;
    }
    if label == "workspace_bucket" {
        return value.len() == 2 && value.bytes().all(|byte| byte.is_ascii_hexdigit());
    }
    !looks_like_uuid(value)
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.' | b'/'))
}

fn looks_like_uuid(value: &str) -> bool {
    value.len() == 36
        && value.bytes().enumerate().all(|(index, byte)| {
            if [8, 13, 18, 23].contains(&index) {
                byte == b'-'
            } else {
                byte.is_ascii_hexdigit()
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> TelemetryConfig {
        TelemetryConfig {
            service: "api",
            version: "abc123".into(),
            log_level: "info",
            otel_endpoint_configured: false,
        }
    }

    #[test]
    fn redacts_forbidden_fields_recursively() {
        let value = SafeEvent::new(&config(), "REQUEST")
            .field("document_title", "secret title")
            .field(
                "safe",
                serde_json::json!({"prompt": "do not log", "result": "ok"}),
            )
            .json();
        let rendered = value.to_string();
        assert!(!rendered.contains("secret title"));
        assert!(!rendered.contains("do not log"));
        assert!(rendered.contains(REDACTED));
        assert!(rendered.contains("ok"));
    }

    #[test]
    fn metric_registry_rejects_unknown_and_high_cardinality_labels() {
        let registry = MetricRegistry::default();
        assert!(registry.increment("missing", &[], 1).is_err());
        assert!(
            registry
                .increment("http_requests_total", &[("user_id", "u1")], 1)
                .is_err()
        );
        assert!(
            registry
                .increment(
                    "http_requests_total",
                    &[("service", "00000000-0000-7000-8000-000000000001")],
                    1,
                )
                .is_err()
        );
        registry
            .increment(
                "http_requests_total",
                &[("service", "api"), ("status", "200")],
                1,
            )
            .unwrap();
        assert_eq!(registry.snapshot().unwrap().values().sum::<u64>(), 1);
    }

    #[test]
    fn trace_context_carries_causal_ids_and_rejects_unsafe_values() {
        let context = TraceContext::new("request-0001", "correlation-0001")
            .unwrap()
            .causation_id("event-0001")
            .unwrap()
            .provider_request_id("provider:0001")
            .unwrap();
        assert_eq!(context.causation_id.as_deref(), Some("event-0001"));
        let _span = context.span();
        assert!(TraceContext::new("short", "correlation-0001").is_err());
        assert!(TraceContext::new("request-0001", "contains secret value").is_err());
    }
}
