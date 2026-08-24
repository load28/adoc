#![forbid(unsafe_code)]

use std::process::ExitCode;

use adoc_configuration::{AppConfig, ConfigSource, ServiceKind};
use adoc_telemetry::{SafeEvent, TelemetryConfig};

fn main() -> ExitCode {
    run(std::env::args()
        .skip(1)
        .any(|argument| argument == "--check-config"))
}

fn run(check_config: bool) -> ExitCode {
    let source = match ConfigSource::from_process() {
        Ok(source) => source,
        Err(error) => return fail(error),
    };
    let config = match AppConfig::parse(&source, ServiceKind::Api) {
        Ok(config) => config,
        Err(error) => return fail(error),
    };
    if check_config {
        println!("{}", config.preflight_json());
        return ExitCode::SUCCESS;
    }
    let telemetry = TelemetryConfig::from(&config.common);
    if let Err(error) = adoc_telemetry::initialize(&telemetry) {
        return fail(error);
    }
    SafeEvent::new(&telemetry, "SERVICE_STARTED")
        .field("environment", format!("{:?}", config.common.environment))
        .emit();
    ExitCode::SUCCESS
}

fn fail(error: impl std::fmt::Display) -> ExitCode {
    eprintln!("configuration failed: {error}");
    ExitCode::FAILURE
}
