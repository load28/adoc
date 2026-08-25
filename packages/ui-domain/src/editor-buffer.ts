import type { DocumentOperation } from "@adoc/contracts";

import { resolveEditorShortcut } from "./editor-commands";

export type OperationGroup = {
  groupId: string;
  sequence: number;
  baseRevision: number;
  operations: DocumentOperation[];
  createdAt: string;
};

export type BufferAck = {
  revision: number;
  appliedOperationIds: string[];
  inverseOperations: DocumentOperation[];
};

export type OperationBufferState = "IDLE" | "PENDING" | "SENDING" | "CONFLICT" | "OFFLINE";

export interface RecoveryStore {
  put(group: OperationGroup): Promise<void>;
  delete(groupId: string): Promise<void>;
}

export type OperationSender = (group: OperationGroup, idempotencyKey: string) => Promise<BufferAck>;

export function editorShortcut(
  event: Pick<KeyboardEvent, "isComposing" | "keyCode" | "key" | "metaKey" | "ctrlKey">,
): "SAVE" | "UNDO" | null {
  const command = resolveEditorShortcut({ ...event, shiftKey: false, altKey: false });
  if (command === "editor.save") return "SAVE";
  if (command === "editor.undo") return "UNDO";
  return null;
}

export class OperationBuffer {
  #pending: OperationGroup[] = [];
  #sending = false;
  #online = true;
  #state: OperationBufferState = "IDLE";
  #revision: number;
  #sequence = 0;
  #inverse: DocumentOperation[][] = [];

  constructor(
    revision: number,
    private readonly store: RecoveryStore,
    private readonly sender: OperationSender,
    private readonly onState: (state: OperationBufferState) => void = () => {},
  ) {
    this.#revision = revision;
  }

  get state(): OperationBufferState {
    return this.#state;
  }

  get revision(): number {
    return this.#revision;
  }

  get hasUnsynced(): boolean {
    return this.#pending.length > 0 || this.#sending;
  }

  takeUndo(): DocumentOperation[] | undefined {
    return this.#inverse.pop();
  }

  restore(groups: OperationGroup[]): void {
    if (this.#pending.length > 0 || this.#sending || this.#state === "CONFLICT") {
      throw new Error("operation buffer cannot restore into a non-empty state");
    }
    const ordered = [...groups].sort((left, right) => left.sequence - right.sequence);
    let expectedRevision = this.#revision;
    for (const group of ordered) {
      if (group.baseRevision !== expectedRevision)
        throw new Error("recovery revision is not contiguous");
      if (
        group.operations.some(
          (operation) => operation.precondition.draftRevision !== expectedRevision,
        )
      ) {
        throw new Error("recovery operation revision does not match its group");
      }
      expectedRevision += 1;
    }
    this.#pending = ordered;
    this.#sequence = Math.max(this.#sequence, ...ordered.map((group) => group.sequence), 0);
    if (ordered.length > 0) this.#setState(this.#online ? "PENDING" : "OFFLINE");
  }

  async enqueue(
    operations: DocumentOperation[],
    groupId: string = crypto.randomUUID(),
  ): Promise<void> {
    if (this.#state === "CONFLICT") throw new Error("operation buffer is conflicted");
    if (operations.length === 0) return;
    const baseRevision = this.#revision + this.#pending.length;
    const group: OperationGroup = {
      groupId,
      sequence: ++this.#sequence,
      baseRevision,
      operations: operations.map((operation) => ({
        ...operation,
        precondition: { ...operation.precondition, draftRevision: baseRevision },
      })),
      createdAt: new Date().toISOString(),
    };
    await this.store.put(group);
    this.#pending.push(group);
    this.#setState(this.#online ? "PENDING" : "OFFLINE");
  }

  async flush(): Promise<void> {
    if (this.#sending || !this.#online || this.#state === "CONFLICT") return;
    const group = this.#pending[0];
    if (!group) {
      this.#setState("IDLE");
      return;
    }
    this.#sending = true;
    this.#setState("SENDING");
    try {
      const ack = await this.sender(group, group.groupId);
      const requested = group.operations.map((operation) => operation.opId).sort();
      const applied = [...ack.appliedOperationIds].sort();
      if (
        ack.revision !== group.baseRevision + 1 ||
        JSON.stringify(requested) !== JSON.stringify(applied)
      ) {
        this.#setState("CONFLICT");
        return;
      }
      this.#revision = ack.revision;
      this.#inverse.push(ack.inverseOperations);
      this.#pending.shift();
      await this.store.delete(group.groupId);
      this.#setState(this.#pending.length > 0 ? "PENDING" : "IDLE");
    } catch (error) {
      if (isConflict(error)) this.#setState("CONFLICT");
      else this.#setState(this.#online ? "PENDING" : "OFFLINE");
      throw error;
    } finally {
      this.#sending = false;
    }
    if (this.#pending.length > 0) await this.flush();
  }

  setOnline(online: boolean): void {
    this.#online = online;
    if (!online && this.hasUnsynced) this.#setState("OFFLINE");
    else if (online && this.#state === "OFFLINE")
      this.#setState(this.#pending.length ? "PENDING" : "IDLE");
  }

  #setState(state: OperationBufferState): void {
    if (state === this.#state) return;
    this.#state = state;
    this.onState(state);
  }
}

function isConflict(error: unknown): boolean {
  if (!error || typeof error !== "object") return false;
  const code = (error as { problem?: { code?: unknown } }).problem?.code;
  return (
    typeof code === "string" &&
    [
      "REVISION_CONFLICT",
      "DRAFT_REVISION_STALE",
      "EDIT_LEASE_INVALID",
      "EDIT_LEASE_HELD",
      "EDIT_LEASE_EXPIRED",
      "PRECONDITION_FAILED",
    ].includes(code)
  );
}
