import { describe, expect, test } from "bun:test";
import type { DocumentOperation } from "@adoc/contracts";

import {
  OperationBuffer,
  RecoveryAuthenticationError,
  decryptRecoveryPayload,
  encryptRecoveryPayload,
  generateRecoveryKey,
  editorShortcut,
} from "../src";

const operation = {
  opId: "00000000-0000-4000-8000-000000000010",
  kind: "DELETE_BLOCK",
  scope: { kind: "BLOCK", blockId: "00000000-0000-4000-8000-000000000011" },
  precondition: { draftRevision: 4 },
  blockId: "00000000-0000-4000-8000-000000000011",
} as DocumentOperation;

describe("editor recovery", () => {
  test("authenticates draft identity and rejects another key", async () => {
    const key = await generateRecoveryKey();
    const record = await encryptRecoveryPayload(
      {
        workspaceId: "workspace",
        documentId: "document",
        draftId: "draft",
        recoverySessionId: "session",
        groupId: "group",
        baseRevision: 4,
        sequence: 1,
        operations: [operation],
        createdAt: new Date(0).toISOString(),
      },
      key,
      0,
    );
    expect((await decryptRecoveryPayload(record, key, 1)).operations).toEqual([operation]);
    await expect(
      decryptRecoveryPayload(record, await generateRecoveryKey(), 1),
    ).rejects.toBeInstanceOf(RecoveryAuthenticationError);
  });
});

describe("operation buffer", () => {
  test("never handles editor shortcuts during Korean IME composition", () => {
    expect(
      editorShortcut({ isComposing: true, keyCode: 229, key: "s", metaKey: true, ctrlKey: false }),
    ).toBeNull();
    expect(
      editorShortcut({ isComposing: false, keyCode: 83, key: "s", metaKey: false, ctrlKey: true }),
    ).toBe("SAVE");
  });

  test("persists before send and acknowledges one in-flight group", async () => {
    const events: string[] = [];
    const buffer = new OperationBuffer(
      4,
      {
        put: async () => {
          events.push("persist");
        },
        delete: async () => {
          events.push("delete");
        },
      },
      async (group) => {
        events.push("send");
        return {
          revision: group.baseRevision + 1,
          appliedOperationIds: [operation.opId],
          inverseOperations: [],
        };
      },
    );
    await buffer.enqueue([operation], "00000000-0000-4000-8000-000000000012");
    await buffer.flush();
    expect(events).toEqual(["persist", "send", "delete"]);
    expect(buffer.revision).toBe(5);
    expect(buffer.hasUnsynced).toBeFalse();
  });

  test("keeps the record and stops on an invalid ack", async () => {
    let deleted = false;
    const buffer = new OperationBuffer(
      4,
      {
        put: async () => {},
        delete: async () => {
          deleted = true;
        },
      },
      async () => ({ revision: 8, appliedOperationIds: [], inverseOperations: [] }),
    );
    await buffer.enqueue([operation]);
    await buffer.flush();
    expect(buffer.state).toBe("CONFLICT");
    expect(buffer.hasUnsynced).toBeTrue();
    expect(deleted).toBeFalse();
  });

  test("serializes pending groups and restamps their revisions", async () => {
    const sent: Array<{ revision: number; active: number }> = [];
    let active = 0;
    const buffer = new OperationBuffer(
      4,
      { put: async () => {}, delete: async () => {} },
      async (group) => {
        active += 1;
        sent.push({ revision: group.operations[0]?.precondition.draftRevision ?? -1, active });
        active -= 1;
        return {
          revision: group.baseRevision + 1,
          appliedOperationIds: group.operations.map((item) => item.opId),
          inverseOperations: [],
        };
      },
    );
    await buffer.enqueue([operation], "group-one");
    await buffer.enqueue(
      [{ ...operation, opId: "00000000-0000-4000-8000-000000000013" }],
      "group-two",
    );
    await buffer.flush();
    expect(sent).toEqual([
      { revision: 4, active: 1 },
      { revision: 5, active: 1 },
    ]);
    expect(buffer.revision).toBe(6);
  });

  test("restores only a contiguous revision chain", () => {
    const buffer = new OperationBuffer(
      4,
      { put: async () => {}, delete: async () => {} },
      async () => {
        throw new Error("not sent");
      },
    );
    buffer.restore([
      {
        groupId: "restored",
        sequence: 1,
        baseRevision: 4,
        operations: [operation],
        createdAt: new Date(0).toISOString(),
      },
    ]);
    expect(buffer.state).toBe("PENDING");
    expect(buffer.hasUnsynced).toBeTrue();
    const stale = new OperationBuffer(
      5,
      { put: async () => {}, delete: async () => {} },
      async () => {
        throw new Error("not sent");
      },
    );
    expect(() =>
      stale.restore([
        {
          groupId: "stale",
          sequence: 1,
          baseRevision: 4,
          operations: [operation],
          createdAt: new Date(0).toISOString(),
        },
      ]),
    ).toThrow("not contiguous");
  });
});
