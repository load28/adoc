export type ProposalSelectionOperation = {
  opId: string;
  dependsOn?: string[];
};

export function selectProposalOperation(
  operations: readonly ProposalSelectionOperation[],
  selected: readonly string[],
  operationId: string,
): string[] {
  const selectedSet = new Set(selected);
  if (selectedSet.has(operationId)) {
    selectedSet.delete(operationId);
    let changed = true;
    while (changed) {
      changed = false;
      for (const operation of operations) {
        if (
          selectedSet.has(operation.opId) &&
          (operation.dependsOn ?? []).some((dependency) => !selectedSet.has(dependency))
        ) {
          selectedSet.delete(operation.opId);
          changed = true;
        }
      }
    }
  } else {
    const byId = new Map(operations.map((operation) => [operation.opId, operation]));
    const visiting = new Set<string>();
    const add = (id: string) => {
      if (selectedSet.has(id)) return;
      if (visiting.has(id)) throw new Error("proposal dependency cycle");
      const operation = byId.get(id);
      if (!operation) throw new Error("proposal dependency missing");
      visiting.add(id);
      for (const dependency of operation.dependsOn ?? []) add(dependency);
      visiting.delete(id);
      selectedSet.add(id);
    };
    add(operationId);
  }
  return operations
    .filter((operation) => selectedSet.has(operation.opId))
    .map((operation) => operation.opId);
}
