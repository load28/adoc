#![forbid(unsafe_code)]

//! Generated transport types and runtime validation for canonical contracts.

mod generated;

pub use generated::*;

use jsonschema::{Draft, Validator};
use serde_json::Value;

const BUNDLE: &str =
    include_str!("../../../packages/contracts/src/generated/contract-bundle.schema.json");

#[derive(Clone, Copy, Debug)]
pub enum ContractName {
    AiTask,
    Content,
    Event,
    Operation,
}

impl ContractName {
    fn definition(self) -> &'static str {
        match self {
            Self::AiTask => "AiContracts__task",
            Self::Content => "DocumentContent",
            Self::Event => "EventPayloads",
            Self::Operation => "DocumentOperation",
        }
    }
}

pub fn validator(name: ContractName) -> Result<Validator, jsonschema::ValidationError<'static>> {
    let bundle: Value = serde_json::from_str(BUNDLE).expect("generated bundle must be JSON");
    let schema = serde_json::json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$ref": format!("{}#/$defs/{}", bundle["$id"].as_str().unwrap(), name.definition()),
        "$defs": bundle["$defs"].clone(),
        "$id": bundle["$id"].clone(),
    });
    jsonschema::options()
        .with_draft(Draft::Draft202012)
        .build(&schema)
}
