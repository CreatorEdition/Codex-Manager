function asRecord(value: unknown): Record<string, unknown> | null {
  return value && typeof value === "object" && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : null;
}

function readStringField(payload: unknown, key: string, fallback = ""): string {
  const source = asRecord(payload);
  const value = source?.[key];
  return typeof value === "string" ? value.trim() : fallback;
}

function readBooleanField(payload: unknown, key: string, fallback = false): boolean {
  const source = asRecord(payload);
  const value = source?.[key];
  return typeof value === "boolean" ? value : fallback;
}

function readNumberField(payload: unknown, key: string, fallback = 0): number {
  const source = asRecord(payload);
  const value = source?.[key];
  if (typeof value === "number" && Number.isFinite(value)) {
    return value;
  }
  if (typeof value === "string") {
    const parsed = Number(value);
    if (Number.isFinite(parsed)) {
      return parsed;
    }
  }
  return fallback;
}

function readStringArrayField(payload: unknown, key: string): string[] {
  const source = asRecord(payload);
  const value = source?.[key];
  return Array.isArray(value)
    ? value
        .map((item) => (typeof item === "string" ? item.trim() : ""))
        .filter(Boolean)
    : [];
}

export interface AccountImportError {
  index: number;
  message: string;
}

export interface AccountImportResult {
  canceled?: boolean;
  total?: number;
  created?: number;
  updated?: number;
  failed?: number;
  errors?: AccountImportError[];
  fileCount?: number;
  directoryPath?: string;
  contents?: string[];
}

export interface AccountExportResult {
  canceled?: boolean;
  exported?: number;
  outputDir?: string;
}

export interface DeleteUnavailableFreeResult {
  deleted?: number;
}

export interface DeleteAccountsByStatusesResult {
  scanned?: number;
  deleted?: number;
  skippedStatus?: number;
  targetStatuses?: string[];
  deletedAccountIds?: string[];
}

export interface AccountWarmupItemResult {
  accountId: string;
  accountName: string;
  ok: boolean;
  message: string;
}

export interface AccountWarmupResult {
  requested?: number;
  succeeded?: number;
  failed?: number;
  results?: AccountWarmupItemResult[];
}

export const ACCOUNT_IMPORT_RPC_BATCH_ITEM_LIMIT = 10;
export const ACCOUNT_IMPORT_RPC_BATCH_BODY_LIMIT_BYTES = 4 * 1024 * 1024;

export function estimateAccountImportRequestBytes(contents: string[]): number {
  return new Blob([JSON.stringify({ contents })]).size;
}

export function splitAccountImportContents(contents: string[]): string[][] {
  const chunks: string[][] = [];
  let current: string[] = [];

  for (const content of contents) {
    if (current.length >= ACCOUNT_IMPORT_RPC_BATCH_ITEM_LIMIT) {
      chunks.push(current);
      current = [];
    }

    const next = current.concat(content);
    if (
      current.length > 0 &&
      estimateAccountImportRequestBytes(next) >
        ACCOUNT_IMPORT_RPC_BATCH_BODY_LIMIT_BYTES
    ) {
      chunks.push(current);
      current = [content];
      if (
        estimateAccountImportRequestBytes(current) >
        ACCOUNT_IMPORT_RPC_BATCH_BODY_LIMIT_BYTES
      ) {
        throw new Error("单条导入内容过大，请拆分后重试");
      }
      continue;
    }

    current = next;
    if (
      current.length === 1 &&
      estimateAccountImportRequestBytes(current) >
        ACCOUNT_IMPORT_RPC_BATCH_BODY_LIMIT_BYTES
    ) {
      throw new Error("单条导入内容过大，请拆分后重试");
    }
  }

  if (current.length > 0) {
    chunks.push(current);
  }

  return chunks;
}

export function readAccountImportResult(payload: unknown): AccountImportResult {
  const source = asRecord(payload);
  const errors = Array.isArray(source?.errors)
    ? source.errors
        .map((item) => {
          const entry = asRecord(item);
          if (!entry) {
            return null;
          }
          return {
            index: readNumberField(entry, "index"),
            message: readStringField(entry, "message"),
          };
        })
        .filter((item): item is AccountImportError => Boolean(item))
    : [];

  return {
    canceled: readBooleanField(payload, "canceled"),
    total: readNumberField(payload, "total"),
    created: readNumberField(payload, "created"),
    updated: readNumberField(payload, "updated"),
    failed: readNumberField(payload, "failed"),
    errors,
    fileCount: readNumberField(payload, "fileCount"),
    directoryPath: readStringField(payload, "directoryPath"),
    contents: readStringArrayField(payload, "contents"),
  };
}

export function readAccountExportResult(payload: unknown): AccountExportResult {
  return {
    canceled: readBooleanField(payload, "canceled"),
    exported: readNumberField(payload, "exported"),
    outputDir: readStringField(payload, "outputDir"),
  };
}

export function readDeleteUnavailableFreeResult(
  payload: unknown
): DeleteUnavailableFreeResult {
  return {
    deleted: readNumberField(payload, "deleted"),
  };
}

export function readDeleteAccountsByStatusesResult(
  payload: unknown
): DeleteAccountsByStatusesResult {
  return {
    scanned: readNumberField(payload, "scanned"),
    deleted: readNumberField(payload, "deleted"),
    skippedStatus: readNumberField(payload, "skippedStatus"),
    targetStatuses: readStringArrayField(payload, "targetStatuses"),
    deletedAccountIds: readStringArrayField(payload, "deletedAccountIds"),
  };
}

export function readAccountWarmupResult(payload: unknown): AccountWarmupResult {
  const source = asRecord(payload);
  const results = Array.isArray(source?.results)
    ? source.results
        .map((item) => {
          const entry = asRecord(item);
          if (!entry) {
            return null;
          }
          return {
            accountId: readStringField(entry, "accountId"),
            accountName: readStringField(entry, "accountName"),
            ok: readBooleanField(entry, "ok"),
            message: readStringField(entry, "message"),
          };
        })
        .filter((item): item is AccountWarmupItemResult => Boolean(item))
    : [];

  return {
    requested: readNumberField(payload, "requested"),
    succeeded: readNumberField(payload, "succeeded"),
    failed: readNumberField(payload, "failed"),
    results,
  };
}

export function readApiKeySecret(payload: unknown): string {
  return readStringField(payload, "key");
}
