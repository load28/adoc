use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use url::Url;
use uuid::Uuid;

use crate::model::{OperationError, OperationErrorCode};

const MAX_NODES: usize = 50_000;
const MAX_DEPTH: usize = 32;
const MAX_TEXT_BYTES: usize = 10 * 1024 * 1024;

#[derive(Clone, Debug)]
pub struct ValidatedContent(Value);

impl ValidatedContent {
    pub fn parse(mut value: Value) -> Result<Self, OperationError> {
        normalize_content(&mut value)?;
        validate_content(&value)?;
        Ok(Self(value))
    }

    pub fn as_value(&self) -> &Value {
        &self.0
    }

    pub fn into_value(self) -> Value {
        self.0
    }
}

pub fn normalize_content(value: &mut Value) -> Result<(), OperationError> {
    let root = value
        .as_object_mut()
        .and_then(|object| object.get_mut("root"))
        .ok_or_else(schema_error)?;
    normalize_node(root)
}

fn normalize_node(node: &mut Value) -> Result<(), OperationError> {
    let kind = node_kind(node)?.to_owned();
    if matches!(kind.as_str(), "paragraph" | "heading") {
        normalize_inline_array(
            node.get_mut("children")
                .and_then(Value::as_array_mut)
                .ok_or_else(schema_error)?,
        )?;
    }
    if kind == "toggle" {
        normalize_inline_array(
            node.get_mut("summary")
                .and_then(Value::as_array_mut)
                .ok_or_else(schema_error)?,
        )?;
    }
    if let Some(children) = node_children_mut(node) {
        for child in children {
            normalize_node(child)?;
        }
    }
    Ok(())
}

pub(crate) fn normalize_inline_array(inlines: &mut Vec<Value>) -> Result<(), OperationError> {
    let mut normalized = Vec::<Value>::new();
    for mut inline in inlines.drain(..) {
        let object = inline.as_object_mut().ok_or_else(schema_error)?;
        match object.get("type").and_then(Value::as_str) {
            Some("text") => {
                if object.get("text").and_then(Value::as_str).is_none() {
                    return Err(schema_error());
                }
                normalize_marks(object)?;
                if object.get("text").and_then(Value::as_str) == Some("") {
                    continue;
                }
                let can_merge = normalized.last().is_some_and(|previous| {
                    previous.get("type").and_then(Value::as_str) == Some("text")
                        && previous.get("marks") == inline.get("marks")
                });
                if can_merge {
                    let text = inline
                        .get("text")
                        .and_then(Value::as_str)
                        .ok_or_else(schema_error)?
                        .to_owned();
                    let previous = normalized
                        .last_mut()
                        .and_then(Value::as_object_mut)
                        .and_then(|object| object.get_mut("text"))
                        .ok_or_else(schema_error)?;
                    let merged = format!("{}{}", previous.as_str().ok_or_else(schema_error)?, text);
                    *previous = Value::String(merged);
                } else {
                    normalized.push(inline);
                }
            }
            Some("hardBreak") => normalized.push(inline),
            _ => return Err(schema_error()),
        }
    }
    *inlines = normalized;
    Ok(())
}

fn normalize_marks(inline: &mut Map<String, Value>) -> Result<(), OperationError> {
    let Some(marks) = inline.get_mut("marks") else {
        return Ok(());
    };
    let marks = marks.as_array_mut().ok_or_else(schema_error)?;
    marks.sort_by(|left, right| {
        left.get("type")
            .and_then(Value::as_str)
            .cmp(&right.get("type").and_then(Value::as_str))
            .then_with(|| canonical_json(left).cmp(&canonical_json(right)))
    });
    if marks.is_empty() {
        inline.remove("marks");
    }
    Ok(())
}

fn validate_content(value: &Value) -> Result<(), OperationError> {
    let object = value.as_object().ok_or_else(schema_error)?;
    if object.get("schemaVersion").and_then(Value::as_u64) != Some(1) {
        return Err(schema_error());
    }
    let root = object.get("root").ok_or_else(schema_error)?;
    if node_kind(root)? != "doc" {
        return Err(schema_error());
    }
    let mut state = ValidationState::default();
    validate_node(root, None, 0, &mut state)?;
    if state.nodes > MAX_NODES || state.text_bytes > MAX_TEXT_BYTES {
        return Err(OperationError::new(OperationErrorCode::LimitExceeded, None));
    }
    Ok(())
}

#[derive(Default)]
struct ValidationState {
    ids: BTreeSet<Uuid>,
    nodes: usize,
    text_bytes: usize,
}

fn validate_node(
    node: &Value,
    parent_kind: Option<&str>,
    depth: usize,
    state: &mut ValidationState,
) -> Result<(), OperationError> {
    if depth > MAX_DEPTH {
        return Err(OperationError::new(OperationErrorCode::LimitExceeded, None));
    }
    let object = node.as_object().ok_or_else(schema_error)?;
    let kind = node_kind(node)?;
    if kind != "doc" {
        let id = object
            .get("id")
            .and_then(Value::as_str)
            .and_then(|value| Uuid::parse_str(value).ok())
            .ok_or_else(schema_error)?;
        if !state.ids.insert(id) {
            return Err(OperationError::new(
                OperationErrorCode::ContentInvalid,
                None,
            ));
        }
        state.nodes += 1;
        if !child_allowed(parent_kind.ok_or_else(schema_error)?, kind) {
            return Err(OperationError::new(
                OperationErrorCode::ContentInvalid,
                None,
            ));
        }
    }
    validate_node_attributes(object, kind, parent_kind, state)?;
    if let Some(children) = node_children(node) {
        if required_non_empty(kind) && children.is_empty() {
            return Err(schema_error());
        }
        if kind == "table" {
            validate_table(children)?;
        }
        for child in children {
            validate_node(child, Some(kind), depth + 1, state)?;
        }
    }
    Ok(())
}

fn validate_node_attributes(
    object: &Map<String, Value>,
    kind: &str,
    parent_kind: Option<&str>,
    state: &mut ValidationState,
) -> Result<(), OperationError> {
    if matches!(kind, "paragraph" | "heading") {
        validate_inlines(object.get("children").ok_or_else(schema_error)?, state)?;
    }
    if kind == "toggle" {
        validate_inlines(object.get("summary").ok_or_else(schema_error)?, state)?;
    }
    if kind == "codeBlock" {
        state.text_bytes += object
            .get("text")
            .and_then(Value::as_str)
            .ok_or_else(schema_error)?
            .len();
    }
    if kind == "heading" && !matches!(object.get("level").and_then(Value::as_u64), Some(1..=6)) {
        return Err(schema_error());
    }
    if kind == "orderedList" {
        if object.get("start").and_then(Value::as_u64).unwrap_or(1) == 0 {
            return Err(schema_error());
        }
    } else if object.contains_key("start") {
        return Err(OperationError::new(
            OperationErrorCode::ContentInvalid,
            None,
        ));
    }
    if kind == "listItem" {
        if parent_kind == Some("taskList") {
            if !object.get("checked").is_some_and(Value::is_boolean) {
                return Err(OperationError::new(
                    OperationErrorCode::ContentInvalid,
                    None,
                ));
            }
        } else if object.contains_key("checked") {
            return Err(OperationError::new(
                OperationErrorCode::ContentInvalid,
                None,
            ));
        }
    }
    Ok(())
}

fn validate_inlines(value: &Value, state: &mut ValidationState) -> Result<(), OperationError> {
    let inlines = value.as_array().ok_or_else(schema_error)?;
    for inline in inlines {
        match inline.get("type").and_then(Value::as_str) {
            Some("text") => {
                let text = inline
                    .get("text")
                    .and_then(Value::as_str)
                    .ok_or_else(schema_error)?;
                state.text_bytes += text.len();
                validate_marks(inline.get("marks"))?;
            }
            Some("hardBreak") => state.text_bytes += 1,
            _ => return Err(schema_error()),
        }
    }
    Ok(())
}

fn validate_marks(value: Option<&Value>) -> Result<(), OperationError> {
    let Some(value) = value else { return Ok(()) };
    let marks = value.as_array().ok_or_else(schema_error)?;
    let mut kinds = BTreeSet::new();
    let mut previous = None;
    for mark in marks {
        let kind = mark
            .get("type")
            .and_then(Value::as_str)
            .ok_or_else(schema_error)?;
        if !kinds.insert(kind) || previous.is_some_and(|value| value > kind) {
            return Err(OperationError::new(
                OperationErrorCode::ContentInvalid,
                None,
            ));
        }
        previous = Some(kind);
        if kind == "link" {
            let href = mark
                .get("href")
                .and_then(Value::as_str)
                .ok_or_else(schema_error)?;
            let url = Url::parse(href).map_err(|_| schema_error())?;
            if !matches!(url.scheme(), "http" | "https" | "mailto")
                || !url.username().is_empty()
                || url.password().is_some()
                || href.chars().any(char::is_control)
            {
                return Err(OperationError::new(
                    OperationErrorCode::ContentInvalid,
                    None,
                ));
            }
        }
    }
    if kinds.contains("subscript") && kinds.contains("superscript") {
        return Err(OperationError::new(
            OperationErrorCode::ContentInvalid,
            None,
        ));
    }
    Ok(())
}

fn validate_table(rows: &[Value]) -> Result<(), OperationError> {
    let first = rows.first().ok_or_else(schema_error)?;
    let width = node_children(first)
        .ok_or_else(schema_error)?
        .iter()
        .map(|cell| cell.get("colspan").and_then(Value::as_u64).unwrap_or(1) as usize)
        .sum::<usize>();
    if width == 0 || width > 100 {
        return Err(schema_error());
    }
    let mut occupied = vec![vec![false; width]; rows.len()];
    for (row_index, row) in rows.iter().enumerate() {
        let cells = node_children(row).ok_or_else(schema_error)?;
        let mut column = 0;
        for cell in cells {
            while column < width && occupied[row_index][column] {
                column += 1;
            }
            let colspan = cell.get("colspan").and_then(Value::as_u64).unwrap_or(1) as usize;
            let rowspan = cell.get("rowspan").and_then(Value::as_u64).unwrap_or(1) as usize;
            if colspan == 0
                || rowspan == 0
                || column + colspan > width
                || row_index + rowspan > rows.len()
            {
                return Err(OperationError::new(
                    OperationErrorCode::ContentInvalid,
                    None,
                ));
            }
            for occupied_row in occupied.iter_mut().skip(row_index).take(rowspan) {
                for slot in occupied_row.iter_mut().skip(column).take(colspan) {
                    if *slot {
                        return Err(OperationError::new(
                            OperationErrorCode::ContentInvalid,
                            None,
                        ));
                    }
                    *slot = true;
                }
            }
            column += colspan;
        }
        if occupied[row_index].iter().any(|slot| !slot) {
            return Err(OperationError::new(
                OperationErrorCode::ContentInvalid,
                None,
            ));
        }
    }
    Ok(())
}

fn required_non_empty(kind: &str) -> bool {
    matches!(
        kind,
        "quote"
            | "callout"
            | "bulletList"
            | "orderedList"
            | "taskList"
            | "listItem"
            | "table"
            | "tableRow"
            | "tableCell"
            | "tableHeader"
    )
}

fn child_allowed(parent: &str, child: &str) -> bool {
    match parent {
        "doc" | "toggle" => is_block(child),
        "quote" | "callout" | "listItem" => matches!(
            child,
            "paragraph" | "bulletList" | "orderedList" | "taskList"
        ),
        "bulletList" | "orderedList" | "taskList" => child == "listItem",
        "table" => child == "tableRow",
        "tableRow" => matches!(child, "tableCell" | "tableHeader"),
        "tableCell" | "tableHeader" => matches!(
            child,
            "paragraph" | "bulletList" | "orderedList" | "taskList" | "codeBlock"
        ),
        _ => false,
    }
}

fn is_block(kind: &str) -> bool {
    matches!(
        kind,
        "paragraph"
            | "heading"
            | "quote"
            | "callout"
            | "bulletList"
            | "orderedList"
            | "taskList"
            | "codeBlock"
            | "table"
            | "toggle"
            | "divider"
            | "image"
            | "file"
    )
}

pub(crate) fn node_kind(node: &Value) -> Result<&str, OperationError> {
    node.get("type")
        .and_then(Value::as_str)
        .ok_or_else(schema_error)
}

pub(crate) fn node_id(node: &Value) -> Option<Uuid> {
    node.get("id")
        .and_then(Value::as_str)
        .and_then(|value| Uuid::parse_str(value).ok())
}

pub(crate) fn node_children(node: &Value) -> Option<&Vec<Value>> {
    let key = child_key(node.get("type")?.as_str()?)?;
    node.get(key)?.as_array()
}

pub(crate) fn node_children_mut(node: &mut Value) -> Option<&mut Vec<Value>> {
    let kind = node.get("type")?.as_str()?.to_owned();
    let key = child_key(&kind)?;
    node.get_mut(key)?.as_array_mut()
}

fn child_key(kind: &str) -> Option<&'static str> {
    match kind {
        "doc" | "quote" | "callout" | "listItem" | "tableCell" | "tableHeader" | "toggle" => {
            Some("children")
        }
        "bulletList" | "orderedList" | "taskList" => Some("items"),
        "table" => Some("rows"),
        "tableRow" => Some("cells"),
        _ => None,
    }
}

pub(crate) fn root_mut(content: &mut Value) -> Result<&mut Value, OperationError> {
    content.get_mut("root").ok_or_else(schema_error)
}

pub(crate) fn find_node(node: &Value, id: Uuid) -> Option<&Value> {
    if node_id(node) == Some(id) {
        return Some(node);
    }
    node_children(node)?
        .iter()
        .find_map(|child| find_node(child, id))
}

pub(crate) fn find_node_mut(node: &mut Value, id: Uuid) -> Option<&mut Value> {
    if node_id(node) == Some(id) {
        return Some(node);
    }
    node_children_mut(node)?
        .iter_mut()
        .find_map(|child| find_node_mut(child, id))
}

pub(crate) fn collect_ids(node: &Value, output: &mut BTreeSet<Uuid>) {
    if let Some(id) = node_id(node) {
        output.insert(id);
    }
    if let Some(children) = node_children(node) {
        for child in children {
            collect_ids(child, output);
        }
    }
}

pub(crate) fn canonical_json(value: &Value) -> String {
    match value {
        Value::Null => "null".to_owned(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::String(value) => serde_json::to_string(value).unwrap_or_else(|_| "\"\"".to_owned()),
        Value::Array(values) => format!(
            "[{}]",
            values
                .iter()
                .map(canonical_json)
                .collect::<Vec<_>>()
                .join(",")
        ),
        Value::Object(object) => {
            let sorted = object.iter().collect::<BTreeMap<_, _>>();
            format!(
                "{{{}}}",
                sorted
                    .into_iter()
                    .map(|(key, value)| format!(
                        "{}:{}",
                        serde_json::to_string(key).unwrap_or_else(|_| "\"\"".to_owned()),
                        canonical_json(value)
                    ))
                    .collect::<Vec<_>>()
                    .join(",")
            )
        }
    }
}

pub fn canonical_hash(value: &Value) -> String {
    hex::encode(Sha256::digest(canonical_json(value).as_bytes()))
}

pub(crate) fn schema_error() -> OperationError {
    OperationError::new(OperationErrorCode::SchemaInvalid, None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use uuid::Uuid;

    #[test]
    fn semantic_validator_rejects_duplicate_ids_unsafe_links_and_invalid_table_grid() {
        let id = Uuid::from_u128(1);
        let invalid = [
            json!({"schemaVersion":1,"root":{"type":"doc","children":[
                {"id":id,"type":"divider"},{"id":id,"type":"divider"}
            ]}}),
            json!({"schemaVersion":1,"root":{"type":"doc","children":[
                {"id":id,"type":"paragraph","children":[{"type":"text","text":"x","marks":[{"type":"link","href":"javascript:alert(1)"}]}]}
            ]}}),
            json!({"schemaVersion":1,"root":{"type":"doc","children":[
                {"id":id,"type":"table","rows":[
                    {"id":Uuid::from_u128(2),"type":"tableRow","cells":[{"id":Uuid::from_u128(3),"type":"tableCell","colspan":2,"children":[{"id":Uuid::from_u128(4),"type":"paragraph","children":[]}]}]},
                    {"id":Uuid::from_u128(5),"type":"tableRow","cells":[{"id":Uuid::from_u128(6),"type":"tableCell","children":[{"id":Uuid::from_u128(7),"type":"paragraph","children":[]}]}]}
                ]}
            ]}}),
        ];
        for content in invalid {
            assert!(ValidatedContent::parse(content).is_err());
        }
    }
}
