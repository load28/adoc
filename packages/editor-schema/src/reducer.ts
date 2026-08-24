import type {
  DocumentContent,
  DocumentOperation,
  DocumentOperation_ReferenceTarget,
} from "@adoc/contracts";

type Json = null | boolean | number | string | Json[] | { [key: string]: Json };
type Node = { id?: string; type: string; [key: string]: Json | undefined };
type Scope = DocumentOperation["scope"];
type Reference = {
  referenceId: string;
  sourceRegion: Scope;
  target: DocumentOperation_ReferenceTarget;
};

export type OperationErrorCode =
  | "SCHEMA_INVALID"
  | "CONTENT_INVALID"
  | "BATCH_INVALID"
  | "DEPENDENCY_INVALID"
  | "REGION_NOT_FOUND"
  | "REGION_AMBIGUOUS"
  | "PRECONDITION_FAILED"
  | "TARGET_CONFLICT"
  | "NO_EFFECT"
  | "LIMIT_EXCEEDED";

export class OperationError extends Error {
  constructor(
    readonly code: OperationErrorCode,
    readonly operationId?: string,
  ) {
    super(`operation failed: ${code}`);
  }
}

export interface ReducerInput {
  content: DocumentContent;
  baseRevision: number;
  operations: DocumentOperation[];
  references: Reference[];
}

export interface ReducerResult {
  content: DocumentContent;
  contentFingerprint: string;
  appliedOperationIds: string[];
  inverseOperations: DocumentOperation[];
  referenceEffects: Array<{ kind: "ADD" | "REMOVE"; reference: Reference }>;
}

export interface RegionResolution {
  status: "RESOLVED" | "MOVED" | "AMBIGUOUS" | "ORPHANED";
  region?: Scope;
}

export async function createTextRegion(
  contentInput: DocumentContent,
  blockId: string,
  from: number,
  to: number,
): Promise<Scope> {
  const content = normalize(structuredClone(contentInput));
  validateContent(content);
  const node = findNode(root(content), blockId);
  const inlines = node && inlineChildren(node);
  if (!node || !inlines || from > to) throw new OperationError("REGION_NOT_FOUND");
  const logical = inlineLogical(inlines);
  assertUtf16Boundary(logical, from);
  assertUtf16Boundary(logical, to);
  return makeTextRange(node, blockId, from, to);
}

export async function reanchorRegion(
  contentInput: DocumentContent,
  region: Scope,
): Promise<RegionResolution> {
  const content = normalize(structuredClone(contentInput));
  validateContent(content);
  if (region.kind !== "TEXT_RANGE") {
    try {
      await scopeValue(content, region);
      return { status: "RESOLVED", region: structuredClone(region) };
    } catch (error) {
      if (error instanceof OperationError && error.code === "REGION_NOT_FOUND")
        return { status: "ORPHANED" };
      throw error;
    }
  }
  try {
    await textOffsets(content, region);
    return { status: "RESOLVED", region: structuredClone(region) };
  } catch (error) {
    if (
      !(error instanceof OperationError) ||
      !["REGION_NOT_FOUND", "PRECONDITION_FAILED"].includes(error.code)
    )
      throw error;
  }
  const node = findNode(root(content), region.blockId);
  const inlines = node && inlineChildren(node);
  if (!node || !inlines) return { status: "ORPHANED" };
  const logical = inlineLogical(inlines);
  const length = Math.max(0, region.to.offset - region.from.offset);
  const candidates: Array<{ score: number; from: number; to: number }> = [];
  for (
    let from = Math.max(0, region.from.offset - 256);
    from <= Math.min(logical.length, region.from.offset + 256);
    from += 1
  ) {
    const to = from + length;
    try {
      assertUtf16Boundary(logical, from);
      assertUtf16Boundary(logical, to);
    } catch {
      continue;
    }
    if ((await hashText(logical.slice(from, to))) !== region.quoteHash) continue;
    const score =
      Number((await contextHash(logical, from)) === region.from.contextHash) +
      Number((await contextHash(logical, to)) === region.to.contextHash);
    if (score > 0) candidates.push({ score, from, to });
  }
  const bestScore = Math.max(0, ...candidates.map((candidate) => candidate.score));
  const best = candidates.filter((candidate) => candidate.score === bestScore);
  if (best.length === 0) return { status: "ORPHANED" };
  if (best.length > 1) return { status: "AMBIGUOUS" };
  const candidate = best[0];
  if (!candidate) return { status: "ORPHANED" };
  return {
    status: "MOVED",
    region: await makeTextRange(node, region.blockId, candidate.from, candidate.to),
  };
}

export async function applyOperations(input: ReducerInput): Promise<ReducerResult> {
  if (input.baseRevision < 0 || input.operations.length < 1 || input.operations.length > 500) {
    throw new OperationError("BATCH_INVALID");
  }
  const original = normalize(structuredClone(input.content)) as DocumentContent;
  validateContent(original);
  for (const reference of input.references) validateReference(reference);
  const operations = topologicalOrder(input.operations, input.baseRevision);
  const content = structuredClone(original);
  const references = new Map(
    input.references.map((reference) => [reference.referenceId, structuredClone(reference)]),
  );
  if (references.size !== input.references.length) throw new OperationError("BATCH_INVALID");
  const originalReferences = canonical([...references.values()]);
  const inverses: DocumentOperation[] = [];
  const effects: ReducerResult["referenceEffects"] = [];
  const applied: string[] = [];
  for (const operation of operations) {
    validateScope(operation);
    await checkPrecondition(operation, content, references);
    inverses.push(await applyOne(operation, content, references, effects));
    validateContent(normalize(content));
    applied.push(operation.opId);
  }
  normalize(content);
  if (
    canonical(content) === canonical(original) &&
    canonical([...references.values()]) === originalReferences
  ) {
    throw new OperationError("NO_EFFECT");
  }
  inverses.reverse();
  await stampInverses(inverses, input.baseRevision + 1, content, references);
  return {
    content,
    contentFingerprint: await hashJson(content),
    appliedOperationIds: applied,
    inverseOperations: inverses,
    referenceEffects: effects,
  };
}

function topologicalOrder(operations: DocumentOperation[], revision: number): DocumentOperation[] {
  const byId = new Map<string, DocumentOperation>();
  for (const operation of operations) {
    if (operation.precondition.draftRevision !== revision || byId.has(operation.opId)) {
      throw new OperationError("BATCH_INVALID", operation.opId);
    }
    byId.set(operation.opId, operation);
  }
  const degree = new Map<string, number>();
  const outgoing = new Map<string, string[]>();
  for (const operation of operations) {
    const dependencies = new Set(operation.dependsOn ?? []);
    if (
      dependencies.size !== (operation.dependsOn ?? []).length ||
      dependencies.has(operation.opId) ||
      [...dependencies].some((id) => !byId.has(id))
    ) {
      throw new OperationError("DEPENDENCY_INVALID", operation.opId);
    }
    degree.set(operation.opId, dependencies.size);
    for (const dependency of dependencies) {
      const next = outgoing.get(dependency) ?? [];
      next.push(operation.opId);
      outgoing.set(dependency, next);
    }
  }
  const ready = [...degree]
    .filter(([, value]) => value === 0)
    .map(([id]) => id)
    .sort();
  const result: DocumentOperation[] = [];
  while (ready.length > 0) {
    const id = ready.shift();
    if (!id) break;
    const operation = byId.get(id);
    if (!operation) throw new OperationError("CONTENT_INVALID");
    byId.delete(id);
    result.push(operation);
    for (const dependent of outgoing.get(id) ?? []) {
      const next = (degree.get(dependent) ?? 0) - 1;
      degree.set(dependent, next);
      if (next === 0) {
        ready.push(dependent);
        ready.sort();
      }
    }
  }
  if (byId.size > 0) throw new OperationError("DEPENDENCY_INVALID");
  return result;
}

function validateScope(operation: DocumentOperation): void {
  let target: Scope;
  switch (operation.kind) {
    case "INSERT_BLOCK":
      target =
        operation.parentId === null
          ? { kind: "DOCUMENT" }
          : { kind: "BLOCK", blockId: operation.parentId };
      break;
    case "DELETE_BLOCK":
    case "MOVE_BLOCK":
    case "SET_BLOCK_ATTRS":
      target = { kind: "BLOCK", blockId: operation.blockId };
      break;
    case "REPLACE_TEXT":
    case "SET_MARKS":
      target = operation.range;
      if (target.kind !== "TEXT_RANGE") throw new OperationError("BATCH_INVALID", operation.opId);
      break;
    case "REPLACE_REGION":
      target = operation.region;
      if (target.kind === "TEXT_RANGE") throw new OperationError("BATCH_INVALID", operation.opId);
      break;
    case "ADD_REFERENCE":
    case "REMOVE_REFERENCE":
      target = operation.sourceRegion;
      break;
  }
  if (canonical(target as unknown as Json) !== canonical(operation.scope as unknown as Json)) {
    throw new OperationError("BATCH_INVALID", operation.opId);
  }
}

async function checkPrecondition(
  operation: DocumentOperation,
  content: DocumentContent,
  references: Map<string, Reference>,
): Promise<void> {
  const expected = operation.precondition.targetHash;
  if (!expected) return;
  const actual =
    operation.kind === "REMOVE_REFERENCE"
      ? await hashJson(references.get(operation.referenceId) ?? null)
      : await hashJson(await scopeValue(content, operation.scope));
  if (actual !== expected) throw new OperationError("PRECONDITION_FAILED", operation.opId);
}

async function applyOne(
  operation: DocumentOperation,
  content: DocumentContent,
  references: Map<string, Reference>,
  effects: ReducerResult["referenceEffects"],
): Promise<DocumentOperation> {
  const inverse = (scope: Scope) => ({
    opId: "",
    scope,
    precondition: { draftRevision: operation.precondition.draftRevision + 1, targetHash: null },
    dependsOn: [] as string[],
  });
  const inverseId = await uuidV5(`${operation.opId}:inverse`);
  switch (operation.kind) {
    case "INSERT_BLOCK": {
      const block = operation.block as unknown as Node;
      if (!block.id) throw new OperationError("SCHEMA_INVALID", operation.opId);
      ensureNewIds(content, block, operation.opId);
      insertNode(
        content,
        operation.parentId,
        operation.index,
        structuredClone(block),
        operation.opId,
      );
      return {
        ...inverse({ kind: "BLOCK", blockId: block.id }),
        opId: inverseId,
        kind: "DELETE_BLOCK",
        blockId: block.id,
      } as DocumentOperation;
    }
    case "DELETE_BLOCK": {
      const removed = takeNode(content, operation.blockId);
      if (!removed) throw new OperationError("REGION_NOT_FOUND", operation.opId);
      const scope: Scope = removed.parentId
        ? { kind: "BLOCK", blockId: removed.parentId }
        : { kind: "DOCUMENT" };
      return {
        ...inverse(scope),
        opId: inverseId,
        kind: "INSERT_BLOCK",
        parentId: removed.parentId,
        index: removed.index,
        block: removed.node,
      } as DocumentOperation;
    }
    case "MOVE_BLOCK": {
      const target = findNode(root(content), operation.blockId);
      if (!target) throw new OperationError("REGION_NOT_FOUND", operation.opId);
      if (operation.newParentId && collectIds(target).has(operation.newParentId))
        throw new OperationError("TARGET_CONFLICT", operation.opId);
      const removed = takeNode(content, operation.blockId);
      if (!removed) throw new OperationError("REGION_NOT_FOUND", operation.opId);
      insertNode(content, operation.newParentId, operation.newIndex, removed.node, operation.opId);
      return {
        ...inverse({ kind: "BLOCK", blockId: operation.blockId }),
        opId: inverseId,
        kind: "MOVE_BLOCK",
        blockId: operation.blockId,
        newParentId: removed.parentId,
        newIndex: removed.index,
      } as DocumentOperation;
    }
    case "REPLACE_TEXT": {
      const offsets = await textOffsets(content, operation.range);
      const node = findNode(root(content), offsets.blockId);
      const inlines = node && inlineChildren(node);
      if (!node || !inlines) throw new OperationError("REGION_NOT_FOUND", operation.opId);
      const selected = replaceInlineRange(
        inlines,
        offsets.from,
        offsets.to,
        structuredClone(operation.content) as Json[],
      );
      const range = await makeTextRange(
        node,
        offsets.blockId,
        offsets.from,
        offsets.from + inlineLength(operation.content as Json[]),
      );
      return {
        ...inverse(range),
        opId: inverseId,
        kind: "REPLACE_TEXT",
        range,
        content: selected,
      } as DocumentOperation;
    }
    case "SET_BLOCK_ATTRS": {
      const node = findNode(root(content), operation.blockId);
      if (!node) throw new OperationError("REGION_NOT_FOUND", operation.opId);
      const allowed = mutableAttrs(node.type);
      const previous: Record<string, Json> = {};
      for (const [key, patch] of Object.entries(operation.attrs)) {
        if (!allowed.has(key)) throw new OperationError("TARGET_CONFLICT", operation.opId);
        previous[key] =
          key in node
            ? { action: "SET", value: structuredClone(node[key] as Json) }
            : { action: "REMOVE" };
        if (patch.action === "REMOVE") delete node[key];
        else node[key] = structuredClone(patch.value) as Json;
      }
      return {
        ...inverse({ kind: "BLOCK", blockId: operation.blockId }),
        opId: inverseId,
        kind: "SET_BLOCK_ATTRS",
        blockId: operation.blockId,
        attrs: previous,
      } as DocumentOperation;
    }
    case "SET_MARKS": {
      const offsets = await textOffsets(content, operation.range);
      const node = findNode(root(content), offsets.blockId);
      const inlines = node && inlineChildren(node);
      if (!node || !inlines) throw new OperationError("REGION_NOT_FOUND", operation.opId);
      const selected = replaceInlineRange(inlines, offsets.from, offsets.to, []);
      replaceInlineRange(
        inlines,
        offsets.from,
        offsets.from,
        transformMarks(selected, operation.mode, operation.marks as Json[]),
      );
      const range = await makeTextRange(node, offsets.blockId, offsets.from, offsets.to);
      return {
        ...inverse(range),
        opId: inverseId,
        kind: "REPLACE_TEXT",
        range,
        content: selected,
      } as DocumentOperation;
    }
    case "REPLACE_REGION": {
      if (operation.blocks.length === 0)
        throw new OperationError("TARGET_CONFLICT", operation.opId);
      const old = replaceRegion(
        content,
        operation.region,
        structuredClone(operation.blocks) as unknown as Node[],
        operation.opId,
      );
      const region = replacementRegion(operation.region, operation.blocks as unknown as Node[]);
      return {
        ...inverse(region),
        opId: inverseId,
        kind: "REPLACE_REGION",
        region,
        blocks: old,
      } as DocumentOperation;
    }
    case "ADD_REFERENCE": {
      validateReference({
        referenceId: operation.referenceId,
        sourceRegion: operation.sourceRegion,
        target: operation.target,
      });
      if (references.has(operation.referenceId))
        throw new OperationError("TARGET_CONFLICT", operation.opId);
      const reference = {
        referenceId: operation.referenceId,
        sourceRegion: structuredClone(operation.sourceRegion),
        target: structuredClone(operation.target),
      };
      references.set(reference.referenceId, reference);
      effects.push({ kind: "ADD", reference });
      return {
        ...inverse(operation.sourceRegion),
        opId: inverseId,
        kind: "REMOVE_REFERENCE",
        ...reference,
      } as DocumentOperation;
    }
    case "REMOVE_REFERENCE": {
      validateReference({
        referenceId: operation.referenceId,
        sourceRegion: operation.sourceRegion,
        target: operation.target,
      });
      const reference = {
        referenceId: operation.referenceId,
        sourceRegion: structuredClone(operation.sourceRegion),
        target: structuredClone(operation.target),
      };
      if (
        canonical(references.get(reference.referenceId) as unknown as Json) !==
        canonical(reference as unknown as Json)
      )
        throw new OperationError("TARGET_CONFLICT", operation.opId);
      references.delete(reference.referenceId);
      effects.push({ kind: "REMOVE", reference });
      return {
        ...inverse(operation.sourceRegion),
        opId: inverseId,
        kind: "ADD_REFERENCE",
        ...reference,
      } as DocumentOperation;
    }
  }
}

async function stampInverses(
  inverses: DocumentOperation[],
  revision: number,
  content: DocumentContent,
  references: Map<string, Reference>,
): Promise<void> {
  const simulation = structuredClone(content);
  const referenceSimulation = new Map(
    [...references].map(([id, value]) => [id, structuredClone(value)]),
  );
  let previous: string | undefined;
  for (const operation of inverses) {
    operation.precondition.draftRevision = revision;
    operation.dependsOn = previous ? [previous] : [];
    operation.precondition.targetHash =
      operation.kind === "REMOVE_REFERENCE"
        ? await hashJson(referenceSimulation.get(operation.referenceId) ?? null)
        : await hashJson(await scopeValue(simulation, operation.scope));
    await applyOne(operation, simulation, referenceSimulation, []);
    previous = operation.opId;
  }
}

function root(content: DocumentContent): Node {
  return content.root as unknown as Node;
}

function childArray(node: Node): Node[] | undefined {
  const key =
    node.type === "table"
      ? "rows"
      : node.type === "tableRow"
        ? "cells"
        : ["bulletList", "orderedList", "taskList"].includes(node.type)
          ? "items"
          : ["doc", "quote", "callout", "listItem", "tableCell", "tableHeader", "toggle"].includes(
                node.type,
              )
            ? "children"
            : undefined;
  return key ? (node[key] as unknown as Node[]) : undefined;
}

function findNode(node: Node, id: string): Node | undefined {
  if (node.id === id) return node;
  return childArray(node)
    ?.map((child) => findNode(child, id))
    .find(Boolean);
}

function collectIds(node: Node, output = new Set<string>()): Set<string> {
  if (node.id) {
    if (output.has(node.id)) throw new OperationError("CONTENT_INVALID");
    output.add(node.id);
  }
  for (const child of childArray(node) ?? []) collectIds(child, output);
  return output;
}

function ensureNewIds(content: DocumentContent, node: Node, operationId: string): void {
  const existing = collectIds(root(content));
  const incoming = collectIds(node);
  if (incoming.size === 0 || [...incoming].some((id) => existing.has(id)))
    throw new OperationError("TARGET_CONFLICT", operationId);
}

function insertNode(
  content: DocumentContent,
  parentId: string | null,
  index: number,
  node: Node,
  operationId: string,
): void {
  const parent = parentId ? findNode(root(content), parentId) : root(content);
  const children = parent && childArray(parent);
  if (!children || index < 0 || index > children.length)
    throw new OperationError("TARGET_CONFLICT", operationId);
  children.splice(index, 0, node);
}

function takeNode(
  content: DocumentContent,
  id: string,
): { node: Node; parentId: string | null; index: number } | undefined {
  const visit = (
    parent: Node,
  ): { node: Node; parentId: string | null; index: number } | undefined => {
    const children = childArray(parent);
    if (!children) return undefined;
    const index = children.findIndex((child) => child.id === id);
    if (index >= 0)
      return { node: children.splice(index, 1)[0] as Node, parentId: parent.id ?? null, index };
    return children.map(visit).find(Boolean);
  };
  return visit(root(content));
}

function siblingBounds(children: Node[], scope: Scope): [number, number] | undefined {
  if (scope.kind === "BLOCK") {
    const index = children.findIndex((child) => child.id === scope.blockId);
    return index >= 0 ? [index, index] : undefined;
  }
  if (scope.kind === "BLOCK_RANGE") {
    const start = children.findIndex((child) => child.id === scope.startBlockId);
    const end = children.findIndex((child) => child.id === scope.endBlockId);
    return start >= 0 && start <= end ? [start, end] : undefined;
  }
  if (scope.kind === "SECTION") {
    const start = children.findIndex(
      (child) => child.id === scope.headingId && child.type === "heading",
    );
    if (start < 0) return undefined;
    const level = Number(children[start]?.level);
    const next = children.findIndex(
      (child, index) => index > start && child.type === "heading" && Number(child.level) <= level,
    );
    return [start, next < 0 ? children.length - 1 : next - 1];
  }
  return undefined;
}

function findSibling(
  node: Node,
  scope: Scope,
): { children: Node[]; start: number; end: number } | undefined {
  const children = childArray(node);
  if (!children) return undefined;
  const bounds = siblingBounds(children, scope);
  if (bounds) return { children, start: bounds[0], end: bounds[1] };
  return children.map((child) => findSibling(child, scope)).find(Boolean);
}

function replaceRegion(
  content: DocumentContent,
  scope: Scope,
  replacement: Node[],
  operationId: string,
): Node[] {
  const current =
    scope.kind === "DOCUMENT"
      ? {
          children: childArray(root(content)) as Node[],
          start: 0,
          end: (childArray(root(content))?.length ?? 0) - 1,
        }
      : findSibling(root(content), scope);
  if (!current) throw new OperationError("REGION_NOT_FOUND", operationId);
  const removedIds = new Set(
    current.children.slice(current.start, current.end + 1).flatMap((node) => [...collectIds(node)]),
  );
  const existing = collectIds(root(content));
  for (const id of removedIds) existing.delete(id);
  const incoming = new Set<string>();
  for (const node of replacement)
    for (const id of collectIds(node)) {
      if (incoming.has(id) || existing.has(id))
        throw new OperationError("TARGET_CONFLICT", operationId);
      incoming.add(id);
    }
  return current.children.splice(
    current.start,
    Math.max(0, current.end - current.start + 1),
    ...replacement,
  );
}

function replacementRegion(original: Scope, blocks: Node[]): Scope {
  if (original.kind === "DOCUMENT") return original;
  const first = blocks[0]?.id;
  const last = blocks.at(-1)?.id;
  if (!first || !last) throw new OperationError("CONTENT_INVALID");
  return first === last
    ? { kind: "BLOCK", blockId: first }
    : { kind: "BLOCK_RANGE", startBlockId: first, endBlockId: last };
}

async function scopeValue(content: DocumentContent, scope: Scope): Promise<Json> {
  if (scope.kind === "DOCUMENT")
    return structuredClone(childArray(root(content)) ?? []) as unknown as Json;
  if (scope.kind === "BLOCK") {
    const node = findNode(root(content), scope.blockId);
    if (!node) throw new OperationError("REGION_NOT_FOUND");
    return structuredClone(node) as unknown as Json;
  }
  if (scope.kind === "TEXT_RANGE") {
    const offsets = await textOffsets(content, scope);
    const node = findNode(root(content), offsets.blockId);
    const inlines = node && inlineChildren(node);
    if (!inlines) throw new OperationError("REGION_NOT_FOUND");
    const copy = structuredClone(inlines);
    return replaceInlineRange(copy, offsets.from, offsets.to, []);
  }
  const found = findSibling(root(content), scope);
  if (!found) throw new OperationError("REGION_NOT_FOUND");
  return structuredClone(found.children.slice(found.start, found.end + 1)) as unknown as Json;
}

function inlineChildren(node: Node): Json[] | undefined {
  if (node.type === "paragraph" || node.type === "heading") return node.children as Json[];
  if (node.type === "toggle") return node.summary as Json[];
  return undefined;
}

async function textOffsets(
  content: DocumentContent,
  scope: Scope,
): Promise<{ blockId: string; from: number; to: number }> {
  if (scope.kind !== "TEXT_RANGE") throw new OperationError("BATCH_INVALID");
  const node = findNode(root(content), scope.blockId);
  const inlines = node && inlineChildren(node);
  if (!inlines || scope.from.offset > scope.to.offset) throw new OperationError("REGION_NOT_FOUND");
  const logical = inlineLogical(inlines);
  assertUtf16Boundary(logical, scope.from.offset);
  assertUtf16Boundary(logical, scope.to.offset);
  if (
    (await contextHash(logical, scope.from.offset)) !== scope.from.contextHash ||
    (await contextHash(logical, scope.to.offset)) !== scope.to.contextHash ||
    (await hashText(logical.slice(scope.from.offset, scope.to.offset))) !== scope.quoteHash
  )
    throw new OperationError("PRECONDITION_FAILED");
  return { blockId: scope.blockId, from: scope.from.offset, to: scope.to.offset };
}

async function makeTextRange(
  node: Node,
  blockId: string,
  from: number,
  to: number,
): Promise<Scope> {
  const logical = inlineLogical(inlineChildren(node) ?? []);
  return {
    kind: "TEXT_RANGE",
    blockId,
    from: { offset: from, affinity: "AFTER", contextHash: await contextHash(logical, from) },
    to: { offset: to, affinity: "BEFORE", contextHash: await contextHash(logical, to) },
    quoteHash: await hashText(logical.slice(from, to)),
  };
}

function splitInlines(inlines: Json[], offset: number): [Json[], Json[]] {
  const before: Json[] = [];
  const after: Json[] = [];
  let position = 0;
  for (const inline of inlines) {
    if (!inline || typeof inline !== "object" || Array.isArray(inline))
      throw new OperationError("SCHEMA_INVALID");
    const length = inlineLength([inline]);
    if (position + length <= offset) before.push(inline);
    else if (position >= offset) after.push(inline);
    else if (inline.type === "text" && typeof inline.text === "string") {
      const split = offset - position;
      if (split > 0) before.push({ ...inline, text: inline.text.slice(0, split) });
      if (split < length) after.push({ ...inline, text: inline.text.slice(split) });
    } else throw new OperationError("REGION_NOT_FOUND");
    position += length;
  }
  if (offset > position) throw new OperationError("REGION_NOT_FOUND");
  return [before, after];
}

function replaceInlineRange(
  inlines: Json[],
  from: number,
  to: number,
  replacement: Json[],
): Json[] {
  const [throughTo, after] = splitInlines(structuredClone(inlines), to);
  const [before, selected] = splitInlines(throughTo, from);
  inlines.splice(0, inlines.length, ...normalizeInlines([...before, ...replacement, ...after]));
  return selected;
}

function transformMarks(inlines: Json[], mode: string, marks: Json[]): Json[] {
  const requested = new Map(marks.map((mark) => [(mark as { type: string }).type, mark]));
  return normalizeInlines(
    inlines.map((inline) => {
      if (!inline || typeof inline !== "object" || Array.isArray(inline) || inline.type !== "text")
        return inline;
      let current = new Map(
        ((inline.marks as Json[] | undefined) ?? []).map((mark) => [
          (mark as { type: string }).type,
          mark,
        ]),
      );
      if (mode === "ADD") for (const entry of requested) current.set(...entry);
      else if (mode === "REMOVE") for (const key of requested.keys()) current.delete(key);
      else current = new Map(requested);
      const next = { ...inline };
      if (current.size > 0) next.marks = [...current.values()].sort(markOrder);
      else delete next.marks;
      return next;
    }),
  );
}

function normalize<T>(content: T): T {
  const visit = (node: Node): void => {
    const inlines = inlineChildren(node);
    if (inlines) inlines.splice(0, inlines.length, ...normalizeInlines(inlines));
    for (const child of childArray(node) ?? []) visit(child);
  };
  visit(root(content as DocumentContent));
  return content;
}

function normalizeInlines(inlines: Json[]): Json[] {
  const output: Json[] = [];
  for (const raw of inlines) {
    if (!raw || typeof raw !== "object" || Array.isArray(raw))
      throw new OperationError("SCHEMA_INVALID");
    const inline = structuredClone(raw);
    if (inline.type === "text") {
      if (typeof inline.text !== "string") throw new OperationError("SCHEMA_INVALID");
      if (inline.text.length === 0) continue;
      if (Array.isArray(inline.marks)) inline.marks.sort(markOrder);
      const previous = output.at(-1);
      if (
        previous &&
        typeof previous === "object" &&
        !Array.isArray(previous) &&
        previous.type === "text" &&
        canonical(previous.marks ?? null) === canonical(inline.marks ?? null)
      )
        previous.text = String(previous.text) + inline.text;
      else output.push(inline);
    } else if (inline.type === "hardBreak") output.push(inline);
    else throw new OperationError("SCHEMA_INVALID");
  }
  return output;
}

function markOrder(left: Json, right: Json): number {
  return (
    String((left as { type?: string }).type).localeCompare(
      String((right as { type?: string }).type),
    ) || canonical(left).localeCompare(canonical(right))
  );
}

function inlineLength(inlines: Json[]): number {
  let total = 0;
  for (const inline of inlines) {
    total +=
      (inline as { type?: string }).type === "hardBreak"
        ? 1
        : String((inline as { text?: string }).text ?? "").length;
  }
  return total;
}

function inlineLogical(inlines: Json[]): string {
  return inlines
    .map((inline) =>
      (inline as { type?: string }).type === "hardBreak"
        ? "\n"
        : String((inline as { text?: string }).text ?? ""),
    )
    .join("");
}

function assertUtf16Boundary(text: string, offset: number): void {
  if (offset < 0 || offset > text.length) throw new OperationError("REGION_NOT_FOUND");
  if (
    offset > 0 &&
    offset < text.length &&
    /[\uD800-\uDBFF]/.test(text[offset - 1] ?? "") &&
    /[\uDC00-\uDFFF]/.test(text[offset] ?? "")
  )
    throw new OperationError("REGION_NOT_FOUND");
}

async function contextHash(text: string, offset: number): Promise<string> {
  assertUtf16Boundary(text, offset);
  return hashText(
    `${text.slice(Math.max(0, offset - 32), offset)}\0${text.slice(offset, offset + 32)}`,
  );
}

function mutableAttrs(kind: string): Set<string> {
  const values: Record<string, string[]> = {
    heading: ["level"],
    callout: ["tone", "icon"],
    orderedList: ["start"],
    listItem: ["checked"],
    codeBlock: ["language"],
    tableCell: ["colspan", "rowspan"],
    tableHeader: ["colspan", "rowspan"],
    image: ["alt", "caption", "width"],
    file: ["caption"],
  };
  return new Set(values[kind] ?? []);
}

function validateContent(content: DocumentContent): void {
  if (content.schemaVersion !== 1 || root(content).type !== "doc")
    throw new OperationError("SCHEMA_INVALID");
  const ids = collectIds(root(content));
  if (ids.size > 50_000) throw new OperationError("LIMIT_EXCEEDED");
  const visit = (node: Node, parent: string | undefined, depth: number): number => {
    if (depth > 32) throw new OperationError("LIMIT_EXCEEDED");
    if (parent && !childAllowed(parent, node.type)) throw new OperationError("CONTENT_INVALID");
    if (
      node.type === "orderedList" &&
      (!Number.isInteger(node.start ?? 1) || Number(node.start ?? 1) < 1)
    )
      throw new OperationError("CONTENT_INVALID");
    if (node.type !== "orderedList" && "start" in node) throw new OperationError("CONTENT_INVALID");
    if (node.type === "listItem") {
      if (parent === "taskList" ? typeof node.checked !== "boolean" : "checked" in node)
        throw new OperationError("CONTENT_INVALID");
    }
    let bytes = 0;
    for (const inline of inlineChildren(node) ?? []) {
      bytes += new TextEncoder().encode(
        (inline as { type?: string }).type === "hardBreak"
          ? "\n"
          : String((inline as { text?: string }).text ?? ""),
      ).length;
      validateMarks((inline as { marks?: Json[] }).marks);
    }
    if (node.type === "codeBlock")
      bytes += new TextEncoder().encode(String(node.text ?? "")).length;
    const children = childArray(node) ?? [];
    if (
      [
        "quote",
        "callout",
        "bulletList",
        "orderedList",
        "taskList",
        "listItem",
        "table",
        "tableRow",
        "tableCell",
        "tableHeader",
      ].includes(node.type) &&
      children.length === 0
    )
      throw new OperationError("CONTENT_INVALID");
    if (node.type === "table") validateTable(children);
    for (const child of children) bytes += visit(child, node.type, depth + 1);
    return bytes;
  };
  if (visit(root(content), undefined, 0) > 10 * 1024 * 1024)
    throw new OperationError("LIMIT_EXCEEDED");
}

function childAllowed(parent: string, child: string): boolean {
  if (parent === "doc" || parent === "toggle")
    return [
      "paragraph",
      "heading",
      "quote",
      "callout",
      "bulletList",
      "orderedList",
      "taskList",
      "codeBlock",
      "table",
      "toggle",
      "divider",
      "image",
      "file",
    ].includes(child);
  if (["quote", "callout", "listItem"].includes(parent))
    return ["paragraph", "bulletList", "orderedList", "taskList"].includes(child);
  if (["bulletList", "orderedList", "taskList"].includes(parent)) return child === "listItem";
  if (parent === "table") return child === "tableRow";
  if (parent === "tableRow") return child === "tableCell" || child === "tableHeader";
  if (parent === "tableCell" || parent === "tableHeader")
    return ["paragraph", "bulletList", "orderedList", "taskList", "codeBlock"].includes(child);
  return false;
}

function validateMarks(marks: Json[] | undefined): void {
  const kinds = new Set<string>();
  for (const mark of marks ?? []) {
    if (
      !mark ||
      typeof mark !== "object" ||
      Array.isArray(mark) ||
      typeof mark.type !== "string" ||
      kinds.has(mark.type)
    )
      throw new OperationError("CONTENT_INVALID");
    kinds.add(mark.type);
    if (mark.type === "link") {
      if (
        typeof mark.href !== "string" ||
        [...mark.href].some((character) => {
          const code = character.codePointAt(0) ?? 0;
          return code <= 31 || code === 127;
        })
      )
        throw new OperationError("CONTENT_INVALID");
      try {
        const url = new URL(mark.href);
        if (
          !["http:", "https:", "mailto:"].includes(url.protocol) ||
          url.username !== "" ||
          url.password !== ""
        )
          throw new OperationError("CONTENT_INVALID");
      } catch (error) {
        if (error instanceof OperationError) throw error;
        throw new OperationError("CONTENT_INVALID");
      }
    }
  }
  if (kinds.has("subscript") && kinds.has("superscript"))
    throw new OperationError("CONTENT_INVALID");
}

function validateTable(rows: Node[]): void {
  const firstCells = rows[0] && childArray(rows[0]);
  const width = firstCells?.reduce((total, cell) => total + Number(cell.colspan ?? 1), 0) ?? 0;
  if (width < 1 || width > 100) throw new OperationError("CONTENT_INVALID");
  const occupied = rows.map(() => Array<boolean>(width).fill(false));
  rows.forEach((row, rowIndex) => {
    let column = 0;
    for (const cell of childArray(row) ?? []) {
      while (column < width && occupied[rowIndex]?.[column]) column += 1;
      const colspan = Number(cell.colspan ?? 1);
      const rowspan = Number(cell.rowspan ?? 1);
      if (
        !Number.isInteger(colspan) ||
        !Number.isInteger(rowspan) ||
        colspan < 1 ||
        rowspan < 1 ||
        column + colspan > width ||
        rowIndex + rowspan > rows.length
      )
        throw new OperationError("CONTENT_INVALID");
      for (let y = rowIndex; y < rowIndex + rowspan; y += 1)
        for (let x = column; x < column + colspan; x += 1) {
          const occupiedRow = occupied[y];
          if (!occupiedRow || occupiedRow[x]) throw new OperationError("CONTENT_INVALID");
          occupiedRow[x] = true;
        }
      column += colspan;
    }
    if (occupied[rowIndex]?.some((slot) => !slot)) throw new OperationError("CONTENT_INVALID");
  });
}

function validateReference(reference: Reference): void {
  if (
    !["DOCUMENT", "REGION", "DISCUSSION", "VOCABULARY", "EXTERNAL"].includes(
      reference.target.kind,
    ) ||
    reference.target.id.length > 2048
  )
    throw new OperationError("SCHEMA_INVALID");
}

function canonical(value: unknown): string {
  if (value === undefined) return "null";
  if (value === null || typeof value !== "object") return JSON.stringify(value);
  if (Array.isArray(value)) return `[${value.map(canonical).join(",")}]`;
  const record = value as Record<string, unknown>;
  return `{${Object.keys(record)
    .sort()
    .map((key) => `${JSON.stringify(key)}:${canonical(record[key])}`)
    .join(",")}}`;
}

async function hashJson(value: unknown): Promise<string> {
  return hashText(canonical(value as Json));
}

async function hashText(value: string): Promise<string> {
  const digest = await crypto.subtle.digest("SHA-256", new TextEncoder().encode(value));
  return [...new Uint8Array(digest)].map((byte) => byte.toString(16).padStart(2, "0")).join("");
}

async function uuidV5(name: string): Promise<string> {
  const namespace = "ad0c0000-0000-5000-8000-000000000009";
  const namespaceBytes = Uint8Array.from(
    namespace.replaceAll("-", "").match(/../g) ?? [],
    (value) => Number.parseInt(value, 16),
  );
  const nameBytes = new TextEncoder().encode(name);
  const bytes = new Uint8Array(namespaceBytes.length + nameBytes.length);
  bytes.set(namespaceBytes);
  bytes.set(nameBytes, namespaceBytes.length);
  const digest = new Uint8Array(await crypto.subtle.digest("SHA-1", bytes)).slice(0, 16);
  digest[6] = ((digest[6] ?? 0) & 0x0f) | 0x50;
  digest[8] = ((digest[8] ?? 0) & 0x3f) | 0x80;
  const hex = [...digest].map((byte) => byte.toString(16).padStart(2, "0")).join("");
  return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`;
}
