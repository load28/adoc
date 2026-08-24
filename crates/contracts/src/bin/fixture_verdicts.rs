use std::{collections::BTreeMap, fs, path::PathBuf};

use adoc_contracts::{ContractName, validator};

fn main() -> anyhow::Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let contracts = [
        ("ai-task", ContractName::AiTask),
        ("content", ContractName::Content),
        ("event", ContractName::Event),
        ("operation", ContractName::Operation),
    ];
    let mut verdicts = BTreeMap::new();
    for (file, contract) in contracts {
        let compiled = validator(contract)?;
        for kind in ["valid", "invalid"] {
            let path = root.join(format!("docs/design/quality/fixtures/{file}.{kind}.json"));
            let value = serde_json::from_str(&fs::read_to_string(path)?)?;
            verdicts.insert(format!("{file}.{kind}"), compiled.is_valid(&value));
        }
    }
    println!("{}", serde_json::to_string(&verdicts)?);
    Ok(())
}
