import { invoke } from "@tauri-apps/api/core";

export const storageRuntimeCommand = "get_storage_runtime_status" as const;

export type StorageRuntimeState =
  | "initializing"
  | "healthy"
  | "warning"
  | "unavailable"
  | "migration_failed"
  | "integrity_failed"
  | "locked"
  | "permission_denied"
  | "resource_limited";

export type DatabaseHealth = "ok" | "warning" | "error";

export type PersistenceState =
  | "created_new"
  | "reopened_existing"
  | "unknown";

export type StorageRuntimeErrorCode =
  | "path_invalid"
  | "permission_denied"
  | "locked"
  | "busy_timeout"
  | "migration_mismatch"
  | "newer_schema"
  | "migration_failed"
  | "integrity_failed"
  | "resource_limit"
  | "internal";

export interface StorageRuntimeStatus {
  state: StorageRuntimeState;
  initialized: boolean;
  schema_version: number | null;
  database_health: DatabaseHealth;
  database_size_bytes: number | null;
  storage_backend: "sqlite";
  sqlite_version: string | null;
  persistence_state: PersistenceState;
  last_start_time_ms: number;
  error_code: StorageRuntimeErrorCode | null;
}

export class StorageRuntimeClientError extends Error {
  readonly code = "storage_status_unavailable";

  constructor() {
    super("Local storage status is unavailable.");
    this.name = "StorageRuntimeClientError";
  }
}

export async function getStorageRuntimeStatus(): Promise<StorageRuntimeStatus> {
  try {
    return await invoke<StorageRuntimeStatus>(storageRuntimeCommand);
  } catch {
    throw new StorageRuntimeClientError();
  }
}
