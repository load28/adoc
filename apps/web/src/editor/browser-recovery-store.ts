import {
  type EncryptedRecoveryRecord,
  type OperationGroup,
  type RecoveryPayload,
  type RecoveryStore,
  decryptRecoveryPayload,
  encryptRecoveryPayload,
  exportRecoveryKey,
  generateRecoveryKey,
  importRecoveryKey,
  recoveryRecordKey,
  recoveryRecordPrefix,
} from "@adoc/ui-domain";

const databaseName = "adoc-editor-recovery-v1";
const objectStore = "records";
const sessionKeyName = "adoc.editor.recovery-key.v1";
type RecoverySession = { id: string; key: string };

export class BrowserRecoveryStore implements RecoveryStore {
  private constructor(
    private readonly workspaceId: string,
    private readonly documentId: string,
    private readonly draftId: string,
    private readonly recoverySessionId: string,
    private readonly key: CryptoKey,
  ) {}

  static async create(
    workspaceId: string,
    documentId: string,
    draftId: string,
  ): Promise<BrowserRecoveryStore> {
    const saved = readRecoverySession();
    const key = saved ? await importRecoveryKey(saved.key) : await generateRecoveryKey();
    const session = saved ?? { id: crypto.randomUUID(), key: await exportRecoveryKey(key) };
    if (!saved) sessionStorage.setItem(sessionKeyName, JSON.stringify(session));
    return new BrowserRecoveryStore(workspaceId, documentId, draftId, session.id, key);
  }

  async put(group: OperationGroup): Promise<void> {
    const record = await encryptRecoveryPayload(
      {
        workspaceId: this.workspaceId,
        documentId: this.documentId,
        draftId: this.draftId,
        recoverySessionId: this.recoverySessionId,
        groupId: group.groupId,
        baseRevision: group.baseRevision,
        sequence: group.sequence,
        operations: group.operations,
        createdAt: group.createdAt,
      },
      this.key,
    );
    await writeRecord(record);
  }

  async delete(groupId: string): Promise<void> {
    await deleteRecord(
      recoveryRecordKey(
        this.workspaceId,
        this.documentId,
        this.draftId,
        this.recoverySessionId,
        groupId,
      ),
    );
  }

  async load(): Promise<OperationGroup[]> {
    const prefix = recoveryRecordPrefix(
      this.workspaceId,
      this.documentId,
      this.draftId,
      this.recoverySessionId,
    );
    const records = await readRecords(prefix);
    const payloads = await Promise.all(
      records.map((record) => decryptRecoveryPayload(record, this.key)),
    );
    return payloads.map(toOperationGroup).sort((left, right) => left.sequence - right.sequence);
  }
}

function readRecoverySession(): RecoverySession | undefined {
  const raw = sessionStorage.getItem(sessionKeyName);
  if (!raw) return undefined;
  try {
    const value = JSON.parse(raw) as Partial<RecoverySession>;
    if (typeof value.id === "string" && typeof value.key === "string")
      return value as RecoverySession;
  } catch {
    // Invalid session metadata must not be reused as key material.
  }
  sessionStorage.removeItem(sessionKeyName);
  return undefined;
}

function toOperationGroup(payload: RecoveryPayload): OperationGroup {
  return {
    groupId: payload.groupId,
    sequence: payload.sequence,
    baseRevision: payload.baseRevision,
    operations: payload.operations,
    createdAt: payload.createdAt,
  };
}

async function openDatabase(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open(databaseName, 1);
    request.onupgradeneeded = () =>
      request.result.createObjectStore(objectStore, { keyPath: "key" });
    request.onerror = () => reject(request.error);
    request.onsuccess = () => resolve(request.result);
  });
}

async function writeRecord(record: EncryptedRecoveryRecord): Promise<void> {
  const database = await openDatabase();
  await transactionPromise(database, "readwrite", (store) => store.put(record));
  database.close();
}

async function deleteRecord(key: string): Promise<void> {
  const database = await openDatabase();
  await transactionPromise(database, "readwrite", (store) => store.delete(key));
  database.close();
}

async function readRecords(prefix: string): Promise<EncryptedRecoveryRecord[]> {
  const database = await openDatabase();
  const records = await new Promise<EncryptedRecoveryRecord[]>((resolve, reject) => {
    const transaction = database.transaction(objectStore, "readonly");
    const request = transaction.objectStore(objectStore).getAll();
    request.onsuccess = () =>
      resolve(
        (request.result as EncryptedRecoveryRecord[]).filter((item) => item.key.startsWith(prefix)),
      );
    request.onerror = () => reject(request.error);
  });
  database.close();
  return records;
}

function transactionPromise(
  database: IDBDatabase,
  mode: IDBTransactionMode,
  action: (store: IDBObjectStore) => IDBRequest,
): Promise<void> {
  return new Promise((resolve, reject) => {
    const transaction = database.transaction(objectStore, mode);
    action(transaction.objectStore(objectStore));
    transaction.oncomplete = () => resolve();
    transaction.onerror = () => reject(transaction.error);
    transaction.onabort = () => reject(transaction.error);
  });
}
