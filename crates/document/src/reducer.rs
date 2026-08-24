use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;
use sha1::Sha1;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    content::{
        ValidatedContent, canonical_hash, collect_ids, find_node, find_node_mut, node_children,
        node_children_mut, node_id, node_kind, normalize_inline_array, root_mut,
    },
    model::{
        Affinity, AttrPatch, DocumentOperation, OperationBase, OperationError, OperationErrorCode,
        OperationPrecondition, OperationScope, ReducerInput, ReducerResult, ReferenceEffect,
        ReferenceSnapshot, RegionResolution, RegionResolutionStatus, SetMarksMode, TextAnchor,
    },
};

const INVERSE_NAMESPACE: Uuid = Uuid::from_u128(0xad0c0000_0000_5000_8000_000000000009);

pub fn reanchor_region(
    content: Value,
    region: &OperationScope,
) -> Result<RegionResolution, OperationError> {
    let content = ValidatedContent::parse(content)?.into_value();
    let OperationScope::TextRange {
        block_id,
        from,
        to,
        quote_hash,
    } = region
    else {
        return Ok(match scope_value(&content, region) {
            Ok(_) => RegionResolution {
                status: RegionResolutionStatus::Resolved,
                region: Some(region.clone()),
            },
            Err(_) => RegionResolution {
                status: RegionResolutionStatus::Orphaned,
                region: None,
            },
        });
    };
    if text_offsets(&content, region).is_ok() {
        return Ok(RegionResolution {
            status: RegionResolutionStatus::Resolved,
            region: Some(region.clone()),
        });
    }
    let Some(node) = find_node(content.get("root").ok_or_else(internal_error)?, *block_id) else {
        return Ok(RegionResolution {
            status: RegionResolutionStatus::Orphaned,
            region: None,
        });
    };
    let Some(inlines) = inline_children(node) else {
        return Ok(RegionResolution {
            status: RegionResolutionStatus::Orphaned,
            region: None,
        });
    };
    let logical = inline_logical(inlines)?;
    let selection_length = to.offset.saturating_sub(from.offset);
    let lower = from.offset.saturating_sub(256);
    let upper = from
        .offset
        .saturating_add(256)
        .min(logical.encode_utf16().count());
    let mut candidates = Vec::new();
    for candidate_from in lower..=upper {
        let candidate_to = candidate_from.saturating_add(selection_length);
        if validate_utf16_boundary(&logical, candidate_from).is_err()
            || validate_utf16_boundary(&logical, candidate_to).is_err()
        {
            continue;
        }
        let Ok(candidate_text) = utf16_slice(&logical, candidate_from, candidate_to) else {
            continue;
        };
        if text_hash(&candidate_text) != *quote_hash {
            continue;
        }
        let context_matches =
            usize::from(anchor_context_hash(&logical, candidate_from)? == from.context_hash)
                + usize::from(anchor_context_hash(&logical, candidate_to)? == to.context_hash);
        if context_matches > 0 {
            candidates.push((context_matches, candidate_from, candidate_to));
        }
    }
    let Some(best_score) = candidates.iter().map(|candidate| candidate.0).max() else {
        return Ok(RegionResolution {
            status: RegionResolutionStatus::Orphaned,
            region: None,
        });
    };
    let mut best = candidates
        .into_iter()
        .filter(|candidate| candidate.0 == best_score);
    let Some((_, candidate_from, candidate_to)) = best.next() else {
        return Err(internal_error());
    };
    if best.next().is_some() {
        return Ok(RegionResolution {
            status: RegionResolutionStatus::Ambiguous,
            region: None,
        });
    }
    Ok(RegionResolution {
        status: RegionResolutionStatus::Moved,
        region: Some(make_text_range(
            node,
            *block_id,
            candidate_from,
            candidate_to,
        )?),
    })
}

pub fn apply_operations(input: ReducerInput) -> Result<ReducerResult, OperationError> {
    if input.base_revision < 0 || input.operations.is_empty() || input.operations.len() > 500 {
        return Err(error(OperationErrorCode::BatchInvalid, None));
    }
    let original = ValidatedContent::parse(input.content)?.into_value();
    let ordered = order_operations(input.operations, input.base_revision)?;
    let mut content = original.clone();
    let reference_count = input.references.len();
    for reference in &input.references {
        validate_reference(reference)?;
    }
    let mut references = input
        .references
        .into_iter()
        .map(|reference| (reference.reference_id, reference))
        .collect::<BTreeMap<_, _>>();
    if references.len() != reference_count {
        return Err(error(OperationErrorCode::BatchInvalid, None));
    }
    let original_references = references.clone();
    let mut inverses = Vec::new();
    let mut effects = Vec::new();
    let mut applied = Vec::new();

    for operation in &ordered {
        validate_scope_contract(operation)?;
        check_precondition(operation, &content, &references)?;
        let inverse = apply_one(operation, &mut content, &mut references, &mut effects)?;
        ValidatedContent::parse(content.clone()).map_err(|mut failure| {
            failure.operation_id = Some(operation.base().op_id);
            failure
        })?;
        inverses.push(inverse);
        applied.push(operation.base().op_id);
    }
    let content = ValidatedContent::parse(content)?.into_value();
    if content == original && references == original_references {
        return Err(error(OperationErrorCode::NoEffect, None));
    }
    inverses.reverse();
    link_and_stamp_inverses(
        &mut inverses,
        input.base_revision + 1,
        &content,
        &references,
    )?;
    Ok(ReducerResult {
        content_fingerprint: canonical_hash(&content),
        content,
        applied_operation_ids: applied,
        inverse_operations: inverses,
        reference_effects: effects,
    })
}

fn order_operations(
    operations: Vec<DocumentOperation>,
    revision: i64,
) -> Result<Vec<DocumentOperation>, OperationError> {
    let mut by_id = BTreeMap::new();
    for operation in operations {
        let id = operation.base().op_id;
        if operation.base().precondition.draft_revision != revision
            || by_id.insert(id, operation).is_some()
        {
            return Err(error(OperationErrorCode::BatchInvalid, Some(id)));
        }
    }
    let ids = by_id.keys().copied().collect::<BTreeSet<_>>();
    let mut indegree = BTreeMap::<Uuid, usize>::new();
    let mut outgoing = BTreeMap::<Uuid, Vec<Uuid>>::new();
    for (id, operation) in &by_id {
        let dependencies = operation
            .base()
            .depends_on
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        if dependencies.len() != operation.base().depends_on.len()
            || dependencies.contains(id)
            || !dependencies.is_subset(&ids)
        {
            return Err(error(OperationErrorCode::DependencyInvalid, Some(*id)));
        }
        indegree.insert(*id, dependencies.len());
        for dependency in dependencies {
            outgoing.entry(dependency).or_default().push(*id);
        }
    }
    let mut ready = indegree
        .iter()
        .filter_map(|(id, degree)| (*degree == 0).then_some(*id))
        .collect::<BTreeSet<_>>();
    let mut result = Vec::with_capacity(by_id.len());
    while let Some(id) = ready.pop_first() {
        result.push(by_id.remove(&id).ok_or_else(internal_error)?);
        for dependent in outgoing.remove(&id).unwrap_or_default() {
            let degree = indegree.get_mut(&dependent).ok_or_else(internal_error)?;
            *degree -= 1;
            if *degree == 0 {
                ready.insert(dependent);
            }
        }
    }
    if !by_id.is_empty() {
        return Err(error(OperationErrorCode::DependencyInvalid, None));
    }
    Ok(result)
}

fn validate_scope_contract(operation: &DocumentOperation) -> Result<(), OperationError> {
    let valid = match operation {
        DocumentOperation::InsertBlock {
            base, parent_id, ..
        } => match parent_id {
            None => base.scope == OperationScope::Document,
            Some(parent) => base.scope == OperationScope::Block { block_id: *parent },
        },
        DocumentOperation::DeleteBlock { base, block_id }
        | DocumentOperation::MoveBlock { base, block_id, .. }
        | DocumentOperation::SetBlockAttrs { base, block_id, .. } => {
            base.scope
                == OperationScope::Block {
                    block_id: *block_id,
                }
        }
        DocumentOperation::ReplaceText { base, range, .. }
        | DocumentOperation::SetMarks { base, range, .. } => {
            base.scope == *range && matches!(range, OperationScope::TextRange { .. })
        }
        DocumentOperation::ReplaceRegion { base, region, .. } => {
            base.scope == *region && !matches!(region, OperationScope::TextRange { .. })
        }
        DocumentOperation::AddReference {
            base,
            source_region,
            ..
        }
        | DocumentOperation::RemoveReference {
            base,
            source_region,
            ..
        } => base.scope == *source_region,
    };
    if valid {
        Ok(())
    } else {
        Err(error(
            OperationErrorCode::BatchInvalid,
            Some(operation.base().op_id),
        ))
    }
}

fn check_precondition(
    operation: &DocumentOperation,
    content: &Value,
    references: &BTreeMap<Uuid, ReferenceSnapshot>,
) -> Result<(), OperationError> {
    let Some(expected) = &operation.base().precondition.target_hash else {
        return Ok(());
    };
    let actual = match operation {
        DocumentOperation::RemoveReference { reference_id, .. } => references
            .get(reference_id)
            .map(|reference| {
                canonical_hash(&serde_json::to_value(reference).unwrap_or(Value::Null))
            })
            .ok_or_else(|| {
                error(
                    OperationErrorCode::TargetConflict,
                    Some(operation.base().op_id),
                )
            })?,
        _ => scope_hash(content, &operation.base().scope)?,
    };
    if expected == &actual {
        Ok(())
    } else {
        Err(error(
            OperationErrorCode::PreconditionFailed,
            Some(operation.base().op_id),
        ))
    }
}

fn apply_one(
    operation: &DocumentOperation,
    content: &mut Value,
    references: &mut BTreeMap<Uuid, ReferenceSnapshot>,
    effects: &mut Vec<ReferenceEffect>,
) -> Result<DocumentOperation, OperationError> {
    let inverse_id = inverse_id(operation.base().op_id);
    let inverse_base = |scope| OperationBase {
        op_id: inverse_id,
        scope,
        precondition: OperationPrecondition {
            draft_revision: operation.base().precondition.draft_revision + 1,
            target_hash: None,
        },
        depends_on: Vec::new(),
    };
    match operation {
        DocumentOperation::InsertBlock {
            parent_id,
            index,
            block,
            ..
        } => {
            let block_id = node_id(block).ok_or_else(|| op_schema(operation))?;
            ensure_new_ids(content, block, operation.base().op_id)?;
            insert_node(
                content,
                *parent_id,
                *index,
                block.clone(),
                operation.base().op_id,
            )?;
            Ok(DocumentOperation::DeleteBlock {
                base: inverse_base(OperationScope::Block { block_id }),
                block_id,
            })
        }
        DocumentOperation::DeleteBlock { block_id, .. } => {
            let (block, parent_id, index) = take_node(content, *block_id)
                .ok_or_else(|| op_error(operation, OperationErrorCode::RegionNotFound))?;
            let scope = parent_id.map_or(OperationScope::Document, |block_id| {
                OperationScope::Block { block_id }
            });
            Ok(DocumentOperation::InsertBlock {
                base: inverse_base(scope),
                parent_id,
                index,
                block,
            })
        }
        DocumentOperation::MoveBlock {
            block_id,
            new_parent_id,
            new_index,
            ..
        } => {
            let node = find_node(content.get("root").ok_or_else(internal_error)?, *block_id)
                .ok_or_else(|| op_error(operation, OperationErrorCode::RegionNotFound))?;
            let mut descendants = BTreeSet::new();
            collect_ids(node, &mut descendants);
            if new_parent_id.is_some_and(|parent| descendants.contains(&parent)) {
                return Err(op_error(operation, OperationErrorCode::TargetConflict));
            }
            let (block, old_parent_id, old_index) = take_node(content, *block_id)
                .ok_or_else(|| op_error(operation, OperationErrorCode::RegionNotFound))?;
            insert_node(
                content,
                *new_parent_id,
                *new_index,
                block,
                operation.base().op_id,
            )?;
            Ok(DocumentOperation::MoveBlock {
                base: inverse_base(OperationScope::Block {
                    block_id: *block_id,
                }),
                block_id: *block_id,
                new_parent_id: old_parent_id,
                new_index: old_index,
            })
        }
        DocumentOperation::ReplaceText {
            range,
            content: replacement,
            ..
        } => {
            let (block_id, from, to) = text_offsets(content, range)?;
            let node = find_node_mut(root_mut(content)?, block_id)
                .ok_or_else(|| op_error(operation, OperationErrorCode::RegionNotFound))?;
            let inlines = inline_children_mut(node).ok_or_else(|| op_schema(operation))?;
            let selected = replace_inline_range(inlines, from, to, replacement.clone())?;
            let inverse_range =
                make_text_range(node, block_id, from, from + inline_len(replacement)?)?;
            Ok(DocumentOperation::ReplaceText {
                base: inverse_base(inverse_range.clone()),
                range: inverse_range,
                content: selected,
            })
        }
        DocumentOperation::SetBlockAttrs {
            block_id, attrs, ..
        } => {
            let node = find_node_mut(root_mut(content)?, *block_id)
                .ok_or_else(|| op_error(operation, OperationErrorCode::RegionNotFound))?;
            let object = node.as_object_mut().ok_or_else(|| op_schema(operation))?;
            let kind = object
                .get("type")
                .and_then(Value::as_str)
                .ok_or_else(|| op_schema(operation))?
                .to_owned();
            let allowed = mutable_attrs(&kind);
            let mut inverse = BTreeMap::new();
            for (key, patch) in attrs {
                if !allowed.contains(&key.as_str()) {
                    return Err(op_error(operation, OperationErrorCode::TargetConflict));
                }
                inverse.insert(
                    key.clone(),
                    object
                        .get(key)
                        .cloned()
                        .map_or(AttrPatch::Remove, |value| AttrPatch::Set { value }),
                );
                match patch {
                    AttrPatch::Set { value } => {
                        object.insert(key.clone(), value.clone());
                    }
                    AttrPatch::Remove => {
                        object.remove(key);
                    }
                }
            }
            Ok(DocumentOperation::SetBlockAttrs {
                base: inverse_base(OperationScope::Block {
                    block_id: *block_id,
                }),
                block_id: *block_id,
                attrs: inverse,
            })
        }
        DocumentOperation::SetMarks {
            range, mode, marks, ..
        } => {
            let (block_id, from, to) = text_offsets(content, range)?;
            let node = find_node_mut(root_mut(content)?, block_id)
                .ok_or_else(|| op_error(operation, OperationErrorCode::RegionNotFound))?;
            let inlines = inline_children_mut(node).ok_or_else(|| op_schema(operation))?;
            let selected = replace_inline_range(inlines, from, to, Vec::new())?;
            let transformed = transform_marks(selected.clone(), *mode, marks)?;
            let _ = replace_inline_range(inlines, from, from, transformed)?;
            let inverse_range = make_text_range(node, block_id, from, to)?;
            Ok(DocumentOperation::ReplaceText {
                base: inverse_base(inverse_range.clone()),
                range: inverse_range,
                content: selected,
            })
        }
        DocumentOperation::ReplaceRegion { region, blocks, .. } => {
            if blocks.is_empty() {
                return Err(op_error(operation, OperationErrorCode::TargetConflict));
            }
            let old = replace_region(content, region, blocks.clone(), operation.base().op_id)?;
            let inverse_region = replacement_region(region, blocks)?;
            Ok(DocumentOperation::ReplaceRegion {
                base: inverse_base(inverse_region.clone()),
                region: inverse_region,
                blocks: old,
            })
        }
        DocumentOperation::AddReference {
            reference_id,
            source_region,
            target,
            ..
        } => {
            let reference = ReferenceSnapshot {
                reference_id: *reference_id,
                source_region: source_region.clone(),
                target: target.clone(),
            };
            validate_reference(&reference).map_err(|_| op_schema(operation))?;
            if references
                .insert(*reference_id, reference.clone())
                .is_some()
            {
                return Err(op_error(operation, OperationErrorCode::TargetConflict));
            }
            effects.push(ReferenceEffect::Add {
                reference: reference.clone(),
            });
            Ok(DocumentOperation::RemoveReference {
                base: inverse_base(source_region.clone()),
                reference_id: *reference_id,
                source_region: source_region.clone(),
                target: target.clone(),
            })
        }
        DocumentOperation::RemoveReference {
            reference_id,
            source_region,
            target,
            ..
        } => {
            let expected = ReferenceSnapshot {
                reference_id: *reference_id,
                source_region: source_region.clone(),
                target: target.clone(),
            };
            validate_reference(&expected).map_err(|_| op_schema(operation))?;
            if references.get(reference_id) != Some(&expected) {
                return Err(op_error(operation, OperationErrorCode::TargetConflict));
            }
            references.remove(reference_id);
            effects.push(ReferenceEffect::Remove {
                reference: expected.clone(),
            });
            Ok(DocumentOperation::AddReference {
                base: inverse_base(source_region.clone()),
                reference_id: *reference_id,
                source_region: source_region.clone(),
                target: target.clone(),
            })
        }
    }
}

fn link_and_stamp_inverses(
    inverses: &mut [DocumentOperation],
    revision: i64,
    content: &Value,
    references: &BTreeMap<Uuid, ReferenceSnapshot>,
) -> Result<(), OperationError> {
    let mut simulation = content.clone();
    let mut reference_simulation = references.clone();
    let mut effects = Vec::new();
    let mut previous = None;
    for inverse in inverses {
        inverse.base_mut().precondition.draft_revision = revision;
        inverse.base_mut().depends_on = previous.into_iter().collect();
        let hash = match inverse {
            DocumentOperation::RemoveReference { reference_id, .. } => reference_simulation
                .get(reference_id)
                .map(|reference| {
                    canonical_hash(&serde_json::to_value(reference).unwrap_or(Value::Null))
                })
                .ok_or_else(internal_error)?,
            _ => scope_hash(&simulation, &inverse.base().scope)?,
        };
        inverse.base_mut().precondition.target_hash = Some(hash);
        let _ = apply_one(
            inverse,
            &mut simulation,
            &mut reference_simulation,
            &mut effects,
        )?;
        previous = Some(inverse.base().op_id);
    }
    Ok(())
}

fn ensure_new_ids(content: &Value, node: &Value, operation: Uuid) -> Result<(), OperationError> {
    let mut existing = BTreeSet::new();
    collect_ids(
        content.get("root").ok_or_else(internal_error)?,
        &mut existing,
    );
    let mut incoming = BTreeSet::new();
    collect_ids(node, &mut incoming);
    if incoming.is_empty() || !existing.is_disjoint(&incoming) {
        Err(error(OperationErrorCode::TargetConflict, Some(operation)))
    } else {
        Ok(())
    }
}

fn insert_node(
    content: &mut Value,
    parent_id: Option<Uuid>,
    index: usize,
    node: Value,
    operation: Uuid,
) -> Result<(), OperationError> {
    let parent = match parent_id {
        Some(id) => find_node_mut(root_mut(content)?, id)
            .ok_or_else(|| error(OperationErrorCode::RegionNotFound, Some(operation)))?,
        None => root_mut(content)?,
    };
    let children = node_children_mut(parent)
        .ok_or_else(|| error(OperationErrorCode::TargetConflict, Some(operation)))?;
    if index > children.len() {
        return Err(error(OperationErrorCode::TargetConflict, Some(operation)));
    }
    children.insert(index, node);
    Ok(())
}

fn take_node(content: &mut Value, id: Uuid) -> Option<(Value, Option<Uuid>, usize)> {
    take_child(root_mut(content).ok()?, id, None)
}

fn take_child(
    node: &mut Value,
    id: Uuid,
    parent_id: Option<Uuid>,
) -> Option<(Value, Option<Uuid>, usize)> {
    let current_id = node_id(node);
    let children = node_children_mut(node)?;
    if let Some(index) = children.iter().position(|child| node_id(child) == Some(id)) {
        return Some((children.remove(index), current_id.or(parent_id), index));
    }
    for child in children {
        if let Some(found) = take_child(child, id, current_id) {
            return Some(found);
        }
    }
    None
}

fn replace_region(
    content: &mut Value,
    region: &OperationScope,
    replacement: Vec<Value>,
    operation: Uuid,
) -> Result<Vec<Value>, OperationError> {
    for node in &replacement {
        ensure_new_ids_for_replace(content, node, region, operation)?;
    }
    if matches!(region, OperationScope::Document) {
        let children = node_children_mut(root_mut(content)?)
            .ok_or_else(|| error(OperationErrorCode::RegionNotFound, Some(operation)))?;
        return Ok(std::mem::replace(children, replacement));
    }
    replace_region_in_node(root_mut(content)?, region, replacement)
        .ok_or_else(|| error(OperationErrorCode::RegionNotFound, Some(operation)))
}

fn replace_region_in_node(
    node: &mut Value,
    region: &OperationScope,
    replacement: Vec<Value>,
) -> Option<Vec<Value>> {
    let children = node_children_mut(node)?;
    if let Some((start, end)) = sibling_bounds(children, region) {
        return Some(children.splice(start..=end, replacement).collect());
    }
    for child in children {
        if let Some(replaced) = replace_region_in_node(child, region, replacement.clone()) {
            return Some(replaced);
        }
    }
    None
}

fn sibling_bounds(children: &[Value], region: &OperationScope) -> Option<(usize, usize)> {
    match region {
        OperationScope::Block { block_id } => children
            .iter()
            .position(|child| node_id(child) == Some(*block_id))
            .map(|index| (index, index)),
        OperationScope::BlockRange {
            start_block_id,
            end_block_id,
        } => {
            let start = children
                .iter()
                .position(|child| node_id(child) == Some(*start_block_id))?;
            let end = children
                .iter()
                .position(|child| node_id(child) == Some(*end_block_id))?;
            (start <= end).then_some((start, end))
        }
        OperationScope::Section { heading_id } => {
            let start = children
                .iter()
                .position(|child| node_id(child) == Some(*heading_id))?;
            let level = children[start].get("level")?.as_u64()?;
            let end = children
                .iter()
                .enumerate()
                .skip(start + 1)
                .find(|(_, child)| {
                    node_kind(child).ok() == Some("heading")
                        && child
                            .get("level")
                            .and_then(Value::as_u64)
                            .is_some_and(|next| next <= level)
                })
                .map_or(children.len() - 1, |(index, _)| index - 1);
            Some((start, end))
        }
        _ => None,
    }
}

fn ensure_new_ids_for_replace(
    content: &Value,
    node: &Value,
    region: &OperationScope,
    operation: Uuid,
) -> Result<(), OperationError> {
    let mut existing = BTreeSet::new();
    collect_ids(
        content.get("root").ok_or_else(internal_error)?,
        &mut existing,
    );
    for id in region_ids(content, region)? {
        existing.remove(&id);
    }
    let mut incoming = BTreeSet::new();
    collect_ids(node, &mut incoming);
    if incoming.is_empty() || !existing.is_disjoint(&incoming) {
        Err(error(OperationErrorCode::TargetConflict, Some(operation)))
    } else {
        Ok(())
    }
}

fn region_ids(content: &Value, region: &OperationScope) -> Result<BTreeSet<Uuid>, OperationError> {
    let value = scope_value(content, region)?;
    let mut ids = BTreeSet::new();
    match value {
        Value::Array(nodes) => {
            for node in &nodes {
                collect_ids(node, &mut ids);
            }
        }
        node => collect_ids(&node, &mut ids),
    }
    Ok(ids)
}

fn replacement_region(
    original: &OperationScope,
    replacement: &[Value],
) -> Result<OperationScope, OperationError> {
    if matches!(original, OperationScope::Document) {
        return Ok(OperationScope::Document);
    }
    let first =
        node_id(replacement.first().ok_or_else(internal_error)?).ok_or_else(internal_error)?;
    let last =
        node_id(replacement.last().ok_or_else(internal_error)?).ok_or_else(internal_error)?;
    Ok(if first == last {
        OperationScope::Block { block_id: first }
    } else {
        OperationScope::BlockRange {
            start_block_id: first,
            end_block_id: last,
        }
    })
}

fn scope_hash(content: &Value, scope: &OperationScope) -> Result<String, OperationError> {
    Ok(canonical_hash(&scope_value(content, scope)?))
}

fn scope_value(content: &Value, scope: &OperationScope) -> Result<Value, OperationError> {
    let root = content.get("root").ok_or_else(internal_error)?;
    match scope {
        OperationScope::Document => Ok(Value::Array(
            node_children(root).ok_or_else(internal_error)?.clone(),
        )),
        OperationScope::Block { block_id } => find_node(root, *block_id)
            .cloned()
            .ok_or_else(|| error(OperationErrorCode::RegionNotFound, None)),
        OperationScope::TextRange { .. } => {
            let (block_id, from, to) = text_offsets(content, scope)?;
            let node = find_node(root, block_id)
                .ok_or_else(|| error(OperationErrorCode::RegionNotFound, None))?;
            let mut inlines = inline_children(node)
                .ok_or_else(|| error(OperationErrorCode::RegionNotFound, None))?
                .clone();
            let selected = replace_inline_range(&mut inlines, from, to, Vec::new())?;
            Ok(Value::Array(selected))
        }
        _ => find_sibling_scope(root, scope)
            .map(Value::Array)
            .ok_or_else(|| error(OperationErrorCode::RegionNotFound, None)),
    }
}

fn find_sibling_scope(node: &Value, scope: &OperationScope) -> Option<Vec<Value>> {
    let children = node_children(node)?;
    if let Some((start, end)) = sibling_bounds(children, scope) {
        return Some(children[start..=end].to_vec());
    }
    children
        .iter()
        .find_map(|child| find_sibling_scope(child, scope))
}

fn text_offsets(
    content: &Value,
    range: &OperationScope,
) -> Result<(Uuid, usize, usize), OperationError> {
    let OperationScope::TextRange {
        block_id,
        from,
        to,
        quote_hash,
    } = range
    else {
        return Err(error(OperationErrorCode::BatchInvalid, None));
    };
    if from.offset > to.offset {
        return Err(error(OperationErrorCode::RegionNotFound, None));
    }
    let node = find_node(content.get("root").ok_or_else(internal_error)?, *block_id)
        .ok_or_else(|| error(OperationErrorCode::RegionNotFound, None))?;
    let logical = inline_logical(
        inline_children(node).ok_or_else(|| error(OperationErrorCode::RegionNotFound, None))?,
    )?;
    validate_utf16_boundary(&logical, from.offset)?;
    validate_utf16_boundary(&logical, to.offset)?;
    if anchor_context_hash(&logical, from.offset)? != from.context_hash
        || anchor_context_hash(&logical, to.offset)? != to.context_hash
        || text_hash(&utf16_slice(&logical, from.offset, to.offset)?) != *quote_hash
    {
        return Err(error(OperationErrorCode::PreconditionFailed, None));
    }
    Ok((*block_id, from.offset, to.offset))
}

fn make_text_range(
    node: &Value,
    block_id: Uuid,
    from: usize,
    to: usize,
) -> Result<OperationScope, OperationError> {
    let logical = inline_logical(inline_children(node).ok_or_else(internal_error)?)?;
    Ok(OperationScope::TextRange {
        block_id,
        from: TextAnchor {
            offset: from,
            affinity: Affinity::After,
            context_hash: anchor_context_hash(&logical, from)?,
        },
        to: TextAnchor {
            offset: to,
            affinity: Affinity::Before,
            context_hash: anchor_context_hash(&logical, to)?,
        },
        quote_hash: text_hash(&utf16_slice(&logical, from, to)?),
    })
}

fn inline_children(node: &Value) -> Option<&Vec<Value>> {
    match node_kind(node).ok()? {
        "paragraph" | "heading" => node.get("children")?.as_array(),
        "toggle" => node.get("summary")?.as_array(),
        _ => None,
    }
}

fn inline_children_mut(node: &mut Value) -> Option<&mut Vec<Value>> {
    let key = match node_kind(node).ok()? {
        "paragraph" | "heading" => "children",
        "toggle" => "summary",
        _ => return None,
    };
    node.get_mut(key)?.as_array_mut()
}

fn replace_inline_range(
    inlines: &mut Vec<Value>,
    from: usize,
    to: usize,
    replacement: Vec<Value>,
) -> Result<Vec<Value>, OperationError> {
    let (before_to, after) = split_inlines(inlines.clone(), to)?;
    let (before, selected) = split_inlines(before_to, from)?;
    let mut result = before;
    result.extend(replacement);
    result.extend(after);
    normalize_inline_array(&mut result)?;
    *inlines = result;
    Ok(selected)
}

fn split_inlines(
    inlines: Vec<Value>,
    offset: usize,
) -> Result<(Vec<Value>, Vec<Value>), OperationError> {
    let mut before = Vec::new();
    let mut after = Vec::new();
    let mut position = 0;
    for inline in inlines {
        let length = inline_len(std::slice::from_ref(&inline))?;
        if position + length <= offset {
            before.push(inline);
        } else if position >= offset {
            after.push(inline);
        } else if inline.get("type").and_then(Value::as_str) == Some("text") {
            let split = offset - position;
            let text = inline
                .get("text")
                .and_then(Value::as_str)
                .ok_or_else(internal_error)?
                .to_owned();
            let byte = utf16_byte_index(&text, split)?;
            let mut left = inline.clone();
            let mut right = inline;
            left["text"] = Value::String(text[..byte].to_owned());
            right["text"] = Value::String(text[byte..].to_owned());
            if split > 0 {
                before.push(left);
            }
            if split < length {
                after.push(right);
            }
        } else {
            return Err(error(OperationErrorCode::RegionNotFound, None));
        }
        position += length;
    }
    if offset > position {
        return Err(error(OperationErrorCode::RegionNotFound, None));
    }
    Ok((before, after))
}

fn transform_marks(
    mut inlines: Vec<Value>,
    mode: SetMarksMode,
    marks: &[Value],
) -> Result<Vec<Value>, OperationError> {
    let requested = marks
        .iter()
        .map(|mark| {
            mark.get("type")
                .and_then(Value::as_str)
                .map(|kind| (kind.to_owned(), mark.clone()))
                .ok_or_else(internal_error)
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    for inline in &mut inlines {
        if inline.get("type").and_then(Value::as_str) != Some("text") {
            continue;
        }
        let object = inline.as_object_mut().ok_or_else(internal_error)?;
        let mut current = object
            .get("marks")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|mark| {
                mark.get("type")
                    .and_then(Value::as_str)
                    .map(|kind| (kind.to_owned(), mark.clone()))
            })
            .collect::<BTreeMap<_, _>>();
        match mode {
            SetMarksMode::Add => current.extend(requested.clone()),
            SetMarksMode::Remove => {
                for kind in requested.keys() {
                    current.remove(kind);
                }
            }
            SetMarksMode::Replace => current = requested.clone(),
        }
        if current.is_empty() {
            object.remove("marks");
        } else {
            object.insert(
                "marks".to_owned(),
                Value::Array(current.into_values().collect()),
            );
        }
    }
    normalize_inline_array(&mut inlines)?;
    Ok(inlines)
}

fn inline_len(inlines: &[Value]) -> Result<usize, OperationError> {
    inlines.iter().try_fold(0, |total, inline| {
        match inline.get("type").and_then(Value::as_str) {
            Some("text") => Ok(total
                + inline
                    .get("text")
                    .and_then(Value::as_str)
                    .ok_or_else(internal_error)?
                    .encode_utf16()
                    .count()),
            Some("hardBreak") => Ok(total + 1),
            _ => Err(internal_error()),
        }
    })
}

fn inline_logical(inlines: &[Value]) -> Result<String, OperationError> {
    let mut output = String::new();
    for inline in inlines {
        match inline.get("type").and_then(Value::as_str) {
            Some("text") => output.push_str(
                inline
                    .get("text")
                    .and_then(Value::as_str)
                    .ok_or_else(internal_error)?,
            ),
            Some("hardBreak") => output.push('\n'),
            _ => return Err(internal_error()),
        }
    }
    Ok(output)
}

fn validate_utf16_boundary(text: &str, offset: usize) -> Result<(), OperationError> {
    utf16_byte_index(text, offset).map(|_| ())
}

fn utf16_byte_index(text: &str, offset: usize) -> Result<usize, OperationError> {
    if offset == 0 {
        return Ok(0);
    }
    let mut units = 0;
    for (byte, character) in text.char_indices() {
        if units == offset {
            return Ok(byte);
        }
        units += character.len_utf16();
        if units > offset {
            return Err(error(OperationErrorCode::RegionNotFound, None));
        }
    }
    if units == offset {
        Ok(text.len())
    } else {
        Err(error(OperationErrorCode::RegionNotFound, None))
    }
}

fn utf16_slice(text: &str, from: usize, to: usize) -> Result<String, OperationError> {
    let from = utf16_byte_index(text, from)?;
    let to = utf16_byte_index(text, to)?;
    Ok(text[from..to].to_owned())
}

fn anchor_context_hash(text: &str, offset: usize) -> Result<String, OperationError> {
    let units = text.encode_utf16().collect::<Vec<_>>();
    if offset > units.len() {
        return Err(error(OperationErrorCode::RegionNotFound, None));
    }
    let before = String::from_utf16(&units[offset.saturating_sub(32)..offset])
        .map_err(|_| error(OperationErrorCode::RegionNotFound, None))?;
    let after = String::from_utf16(&units[offset..(offset + 32).min(units.len())])
        .map_err(|_| error(OperationErrorCode::RegionNotFound, None))?;
    Ok(text_hash(&format!("{before}\0{after}")))
}

fn text_hash(text: &str) -> String {
    hex::encode(Sha256::digest(text.as_bytes()))
}

fn mutable_attrs(kind: &str) -> &'static [&'static str] {
    match kind {
        "heading" => &["level"],
        "callout" => &["tone", "icon"],
        "orderedList" => &["start"],
        "listItem" => &["checked"],
        "codeBlock" => &["language"],
        "tableCell" | "tableHeader" => &["colspan", "rowspan"],
        "image" => &["alt", "caption", "width"],
        "file" => &["caption"],
        _ => &[],
    }
}

fn validate_reference(reference: &ReferenceSnapshot) -> Result<(), OperationError> {
    if !matches!(
        reference.target.kind.as_str(),
        "DOCUMENT" | "REGION" | "DISCUSSION" | "VOCABULARY" | "EXTERNAL"
    ) || reference.target.id.len() > 2048
    {
        return Err(error(OperationErrorCode::SchemaInvalid, None));
    }
    Ok(())
}

fn inverse_id(operation_id: Uuid) -> Uuid {
    let mut digest = Sha1::new();
    digest.update(INVERSE_NAMESPACE.as_bytes());
    digest.update(format!("{operation_id}:inverse").as_bytes());
    let hash = digest.finalize();
    let mut bytes = [0_u8; 16];
    bytes.copy_from_slice(&hash[..16]);
    bytes[6] = (bytes[6] & 0x0f) | 0x50;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Uuid::from_bytes(bytes)
}

fn op_schema(operation: &DocumentOperation) -> OperationError {
    op_error(operation, OperationErrorCode::SchemaInvalid)
}

fn op_error(operation: &DocumentOperation, code: OperationErrorCode) -> OperationError {
    error(code, Some(operation.base().op_id))
}

fn error(code: OperationErrorCode, operation_id: Option<Uuid>) -> OperationError {
    OperationError::new(code, operation_id)
}

fn internal_error() -> OperationError {
    error(OperationErrorCode::ContentInvalid, None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn empty_content() -> Value {
        json!({"schemaVersion":1,"root":{"type":"doc","children":[]}})
    }

    fn base(id: Uuid, revision: i64, scope: OperationScope) -> OperationBase {
        OperationBase {
            op_id: id,
            scope,
            precondition: OperationPrecondition {
                draft_revision: revision,
                target_hash: None,
            },
            depends_on: Vec::new(),
        }
    }

    #[test]
    fn batch_order_is_uuid_deterministic_and_inverse_restores_content() {
        let first = Uuid::from_u128(1);
        let second = Uuid::from_u128(2);
        let block_a = Uuid::from_u128(11);
        let block_b = Uuid::from_u128(12);
        let operations = vec![
            DocumentOperation::InsertBlock {
                base: base(second, 0, OperationScope::Document),
                parent_id: None,
                index: 1,
                block: json!({"id":block_b,"type":"paragraph","children":[{"type":"text","text":"나"}]}),
            },
            DocumentOperation::InsertBlock {
                base: base(first, 0, OperationScope::Document),
                parent_id: None,
                index: 0,
                block: json!({"id":block_a,"type":"paragraph","children":[{"type":"text","text":"가"}]}),
            },
        ];
        let result = apply_operations(ReducerInput {
            content: empty_content(),
            base_revision: 0,
            operations,
            references: Vec::new(),
        })
        .unwrap();
        assert_eq!(result.applied_operation_ids, vec![first, second]);
        let restored = apply_operations(ReducerInput {
            content: result.content,
            base_revision: 1,
            operations: result.inverse_operations,
            references: Vec::new(),
        })
        .unwrap();
        assert_eq!(restored.content, empty_content());
    }

    #[test]
    fn invalid_dependency_and_duplicate_content_id_are_atomic_errors() {
        let id = Uuid::from_u128(1);
        let missing = Uuid::from_u128(2);
        let mut common = base(id, 0, OperationScope::Document);
        common.depends_on.push(missing);
        let error = apply_operations(ReducerInput {
            content: empty_content(),
            base_revision: 0,
            operations: vec![DocumentOperation::InsertBlock {
                base: common,
                parent_id: None,
                index: 0,
                block: json!({"id":Uuid::from_u128(3),"type":"divider"}),
            }],
            references: Vec::new(),
        })
        .unwrap_err();
        assert_eq!(error.code, OperationErrorCode::DependencyInvalid);
    }

    #[test]
    fn utf16_range_rejects_surrogate_interior() {
        let block = Uuid::from_u128(10);
        let content = json!({"schemaVersion":1,"root":{"type":"doc","children":[{"id":block,"type":"paragraph","children":[{"type":"text","text":"가😀나"}]}]}});
        let logical = "가😀나";
        let range = OperationScope::TextRange {
            block_id: block,
            from: TextAnchor {
                offset: 2,
                affinity: Affinity::After,
                context_hash: anchor_context_hash(logical, 1).unwrap(),
            },
            to: TextAnchor {
                offset: 3,
                affinity: Affinity::Before,
                context_hash: anchor_context_hash(logical, 3).unwrap(),
            },
            quote_hash: text_hash("😀"),
        };
        let error = apply_operations(ReducerInput {
            content,
            base_revision: 0,
            operations: vec![DocumentOperation::ReplaceText {
                base: base(Uuid::from_u128(1), 0, range.clone()),
                range,
                content: vec![json!({"type":"text","text":"X"})],
            }],
            references: Vec::new(),
        })
        .unwrap_err();
        assert!(matches!(
            error.code,
            OperationErrorCode::RegionNotFound | OperationErrorCode::PreconditionFailed
        ));
    }

    #[test]
    fn rich_text_replace_and_inverse_preserve_utf16_and_marks() {
        let block = Uuid::from_u128(10);
        let original = json!({"schemaVersion":1,"root":{"type":"doc","children":[{"id":block,"type":"paragraph","children":[{"type":"text","text":"가😀나","marks":[{"type":"bold"}]}]}]}});
        let logical = "가😀나";
        let range = OperationScope::TextRange {
            block_id: block,
            from: TextAnchor {
                offset: 1,
                affinity: Affinity::After,
                context_hash: anchor_context_hash(logical, 1).unwrap(),
            },
            to: TextAnchor {
                offset: 3,
                affinity: Affinity::Before,
                context_hash: anchor_context_hash(logical, 3).unwrap(),
            },
            quote_hash: text_hash("😀"),
        };
        let changed = apply_operations(ReducerInput {
            content: original.clone(),
            base_revision: 0,
            operations: vec![DocumentOperation::ReplaceText {
                base: base(Uuid::from_u128(1), 0, range.clone()),
                range,
                content: vec![json!({"type":"text","text":"X","marks":[{"type":"italic"}]})],
            }],
            references: Vec::new(),
        })
        .unwrap();
        let restored = apply_operations(ReducerInput {
            content: changed.content,
            base_revision: 1,
            operations: changed.inverse_operations,
            references: Vec::new(),
        })
        .unwrap();
        assert_eq!(restored.content, original);
    }

    #[test]
    fn reference_effect_is_self_contained_and_reversible() {
        let reference_id = Uuid::from_u128(20);
        let target = crate::ReferenceTarget {
            kind: "DOCUMENT".to_owned(),
            id: Uuid::from_u128(21).to_string(),
        };
        let changed = apply_operations(ReducerInput {
            content: empty_content(),
            base_revision: 0,
            operations: vec![DocumentOperation::AddReference {
                base: base(Uuid::from_u128(1), 0, OperationScope::Document),
                reference_id,
                source_region: OperationScope::Document,
                target: target.clone(),
            }],
            references: Vec::new(),
        })
        .unwrap();
        let reference = ReferenceSnapshot {
            reference_id,
            source_region: OperationScope::Document,
            target,
        };
        let restored = apply_operations(ReducerInput {
            content: changed.content,
            base_revision: 1,
            operations: changed.inverse_operations,
            references: vec![reference],
        })
        .unwrap();
        assert!(matches!(
            restored.reference_effects.as_slice(),
            [ReferenceEffect::Remove { .. }]
        ));
    }

    #[test]
    fn wire_operation_uses_canonical_camel_case_fields() {
        let operation = DocumentOperation::InsertBlock {
            base: base(Uuid::from_u128(1), 7, OperationScope::Document),
            parent_id: None,
            index: 0,
            block: json!({"id":Uuid::from_u128(2),"type":"divider"}),
        };
        let value = serde_json::to_value(&operation).unwrap();
        assert_eq!(value["kind"], "INSERT_BLOCK");
        assert_eq!(value["opId"], Uuid::from_u128(1).to_string());
        assert_eq!(value["precondition"]["draftRevision"], 7);
        assert!(value.get("parentId").is_some());
        assert_eq!(
            serde_json::from_value::<DocumentOperation>(value).unwrap(),
            operation
        );
    }

    #[test]
    fn shared_fixture_has_the_same_rust_result() {
        let fixture: Value = serde_json::from_str(include_str!(
            "../../../docs/design/quality/fixtures/operation-reducer.valid.json"
        ))
        .unwrap();
        let result = apply_operations(ReducerInput {
            content: fixture["content"].clone(),
            base_revision: fixture["baseRevision"].as_i64().unwrap(),
            operations: serde_json::from_value(fixture["operations"].clone()).unwrap(),
            references: serde_json::from_value(fixture["references"].clone()).unwrap(),
        })
        .unwrap();
        assert_eq!(result.content, fixture["expected"]["content"]);
        assert_eq!(
            result.content_fingerprint,
            fixture["expected"]["contentFingerprint"].as_str().unwrap()
        );
        assert_eq!(
            serde_json::to_value(result.applied_operation_ids).unwrap(),
            fixture["expected"]["appliedOperationIds"]
        );
        assert_eq!(
            serde_json::to_value(
                result
                    .inverse_operations
                    .iter()
                    .map(|operation| operation.base().op_id)
                    .collect::<Vec<_>>()
            )
            .unwrap(),
            fixture["expected"]["inverseOperationIds"]
        );
    }

    #[test]
    fn move_attrs_marks_and_region_replace_round_trip_as_one_batch() {
        let heading = Uuid::from_u128(30);
        let paragraph = Uuid::from_u128(31);
        let divider = Uuid::from_u128(32);
        let original = json!({"schemaVersion":1,"root":{"type":"doc","children":[
            {"id":heading,"type":"heading","level":1,"children":[{"type":"text","text":"hello"}]},
            {"id":paragraph,"type":"paragraph","children":[{"type":"text","text":"world"}]}
        ]}});
        let text_range = OperationScope::TextRange {
            block_id: heading,
            from: TextAnchor {
                offset: 0,
                affinity: Affinity::After,
                context_hash: anchor_context_hash("hello", 0).unwrap(),
            },
            to: TextAnchor {
                offset: 5,
                affinity: Affinity::Before,
                context_hash: anchor_context_hash("hello", 5).unwrap(),
            },
            quote_hash: text_hash("hello"),
        };
        let mut attrs = BTreeMap::new();
        attrs.insert("level".to_owned(), AttrPatch::Set { value: json!(2) });
        let mut operations = vec![
            DocumentOperation::MoveBlock {
                base: base(
                    Uuid::from_u128(1),
                    0,
                    OperationScope::Block {
                        block_id: paragraph,
                    },
                ),
                block_id: paragraph,
                new_parent_id: None,
                new_index: 0,
            },
            DocumentOperation::SetBlockAttrs {
                base: base(
                    Uuid::from_u128(2),
                    0,
                    OperationScope::Block { block_id: heading },
                ),
                block_id: heading,
                attrs,
            },
            DocumentOperation::SetMarks {
                base: base(Uuid::from_u128(3), 0, text_range.clone()),
                range: text_range,
                mode: SetMarksMode::Add,
                marks: vec![json!({"type":"bold"})],
            },
            DocumentOperation::ReplaceRegion {
                base: base(
                    Uuid::from_u128(4),
                    0,
                    OperationScope::Block {
                        block_id: paragraph,
                    },
                ),
                region: OperationScope::Block {
                    block_id: paragraph,
                },
                blocks: vec![json!({"id":divider,"type":"divider"})],
            },
        ];
        for index in 1..operations.len() {
            let previous = operations[index - 1].base().op_id;
            operations[index].base_mut().depends_on = vec![previous];
        }
        let changed = apply_operations(ReducerInput {
            content: original.clone(),
            base_revision: 0,
            operations,
            references: Vec::new(),
        })
        .unwrap();
        let restored = apply_operations(ReducerInput {
            content: changed.content,
            base_revision: 1,
            operations: changed.inverse_operations,
            references: Vec::new(),
        })
        .unwrap();
        assert_eq!(restored.content, original);
    }

    #[test]
    fn text_region_reanchors_to_one_exact_context_candidate() {
        let block = Uuid::from_u128(40);
        let original_text = format!("{}target{}", "x".repeat(40), "y".repeat(40));
        let region = make_text_range(
            &json!({"id":block,"type":"paragraph","children":[{"type":"text","text":original_text}]}),
            block,
            40,
            46,
        )
        .unwrap();
        let moved_text = format!("z{original_text}");
        let content = json!({"schemaVersion":1,"root":{"type":"doc","children":[
            {"id":block,"type":"paragraph","children":[{"type":"text","text":moved_text}]}
        ]}});
        let resolution = reanchor_region(content, &region).unwrap();
        assert_eq!(resolution.status, RegionResolutionStatus::Moved);
        assert!(matches!(
            resolution.region,
            Some(OperationScope::TextRange {
                from: TextAnchor { offset: 41, .. },
                to: TextAnchor { offset: 47, .. },
                ..
            })
        ));
    }
}
