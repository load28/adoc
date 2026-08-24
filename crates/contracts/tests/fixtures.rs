use std::{fs, path::PathBuf};

use adoc_contracts::{ContractName, validator};
use anyhow::Context;

#[test]
fn canonical_fixtures_have_expected_verdicts() -> anyhow::Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let cases = [
        ("ai-task", ContractName::AiTask),
        ("content", ContractName::Content),
        ("event", ContractName::Event),
        ("operation", ContractName::Operation),
    ];
    for (file, contract) in cases {
        let compiled = validator(contract)?;
        for expected in [true, false] {
            let kind = if expected { "valid" } else { "invalid" };
            let path = root.join(format!("docs/design/quality/fixtures/{file}.{kind}.json"));
            let value = serde_json::from_str(
                &fs::read_to_string(&path).with_context(|| path.display().to_string())?,
            )?;
            assert_eq!(compiled.is_valid(&value), expected, "{}", path.display());
        }
    }
    Ok(())
}
