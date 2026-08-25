import type { DocumentOperation } from "@adoc/contracts";

export type RecoveryPayload = {
  workspaceId: string;
  documentId: string;
  draftId: string;
  recoverySessionId: string;
  groupId: string;
  baseRevision: number;
  sequence: number;
  operations: DocumentOperation[];
  createdAt: string;
};

export type EncryptedRecoveryRecord = {
  key: string;
  schemaVersion: 1;
  expiresAt: string;
  iv: string;
  ciphertext: string;
};

export class RecoveryAuthenticationError extends Error {
  constructor() {
    super("recovery record authentication failed");
    this.name = "RecoveryAuthenticationError";
  }
}

export async function generateRecoveryKey(): Promise<CryptoKey> {
  return crypto.subtle.generateKey({ name: "AES-GCM", length: 256 }, true, ["encrypt", "decrypt"]);
}

export async function exportRecoveryKey(key: CryptoKey): Promise<string> {
  return encodeBytes(new Uint8Array(await crypto.subtle.exportKey("raw", key)));
}

export async function importRecoveryKey(encoded: string): Promise<CryptoKey> {
  return crypto.subtle.importKey("raw", toArrayBuffer(decodeBytes(encoded)), "AES-GCM", false, [
    "encrypt",
    "decrypt",
  ]);
}

export async function encryptRecoveryPayload(
  payload: RecoveryPayload,
  key: CryptoKey,
  now = Date.now(),
): Promise<EncryptedRecoveryRecord> {
  const iv = crypto.getRandomValues(new Uint8Array(12));
  const encoded = new TextEncoder().encode(JSON.stringify(payload));
  const ciphertext = await crypto.subtle.encrypt(
    { name: "AES-GCM", iv: toArrayBuffer(iv), additionalData: toArrayBuffer(recoveryAad(payload)) },
    key,
    encoded,
  );
  return {
    key: recoveryRecordKey(
      payload.workspaceId,
      payload.documentId,
      payload.draftId,
      payload.recoverySessionId,
      payload.groupId,
    ),
    schemaVersion: 1,
    expiresAt: new Date(now + 7 * 24 * 60 * 60 * 1000).toISOString(),
    iv: encodeBytes(iv),
    ciphertext: encodeBytes(new Uint8Array(ciphertext)),
  };
}

export async function decryptRecoveryPayload(
  record: EncryptedRecoveryRecord,
  key: CryptoKey,
  now = Date.now(),
): Promise<RecoveryPayload> {
  if (record.schemaVersion !== 1 || Date.parse(record.expiresAt) <= now) {
    throw new RecoveryAuthenticationError();
  }
  try {
    const draftIdentity = parseRecordKey(record.key);
    const plaintext = await crypto.subtle.decrypt(
      {
        name: "AES-GCM",
        iv: toArrayBuffer(decodeBytes(record.iv)),
        additionalData: toArrayBuffer(recoveryAad(draftIdentity)),
      },
      key,
      toArrayBuffer(decodeBytes(record.ciphertext)),
    );
    const payload = JSON.parse(new TextDecoder().decode(plaintext)) as RecoveryPayload;
    if (
      recoveryRecordKey(
        payload.workspaceId,
        payload.documentId,
        payload.draftId,
        payload.recoverySessionId,
        payload.groupId,
      ) !== record.key
    ) {
      throw new RecoveryAuthenticationError();
    }
    return payload;
  } catch {
    throw new RecoveryAuthenticationError();
  }
}

export function recoveryRecordKey(
  workspaceId: string,
  documentId: string,
  draftId: string,
  recoverySessionId: string,
  groupId: string,
): string {
  return `${recoveryRecordPrefix(workspaceId, documentId, draftId, recoverySessionId)}${encodeURIComponent(groupId)}`;
}

export function recoveryRecordPrefix(
  workspaceId: string,
  documentId: string,
  draftId: string,
  recoverySessionId: string,
): string {
  return `${[workspaceId, documentId, draftId, recoverySessionId].map(encodeURIComponent).join(":")}:`;
}

function parseRecordKey(
  key: string,
): Pick<
  RecoveryPayload,
  "workspaceId" | "documentId" | "draftId" | "recoverySessionId" | "groupId"
> {
  const parts = key.split(":").map(decodeURIComponent);
  if (parts.length !== 5 || parts.some((part) => part.length === 0))
    throw new RecoveryAuthenticationError();
  return {
    workspaceId: parts[0] ?? "",
    documentId: parts[1] ?? "",
    draftId: parts[2] ?? "",
    recoverySessionId: parts[3] ?? "",
    groupId: parts[4] ?? "",
  };
}

function recoveryAad(
  value: Pick<
    RecoveryPayload,
    "workspaceId" | "documentId" | "draftId" | "recoverySessionId" | "groupId"
  >,
): Uint8Array {
  return new TextEncoder().encode(
    JSON.stringify([
      value.workspaceId,
      value.documentId,
      value.draftId,
      value.recoverySessionId,
      value.groupId,
      1,
    ]),
  );
}

function encodeBytes(bytes: Uint8Array): string {
  let binary = "";
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return btoa(binary).replaceAll("+", "-").replaceAll("/", "_").replace(/=+$/, "");
}

function decodeBytes(value: string): Uint8Array {
  const normalized = value.replaceAll("-", "+").replaceAll("_", "/");
  const binary = atob(normalized.padEnd(Math.ceil(normalized.length / 4) * 4, "="));
  return Uint8Array.from(binary, (character) => character.charCodeAt(0));
}

function toArrayBuffer(bytes: Uint8Array): ArrayBuffer {
  return Uint8Array.from(bytes).buffer;
}
