import { useCallback, useEffect, useState } from "react";
import {
  AlertTriangle,
  CheckCircle,
  Database,
  Loader2,
  RefreshCw,
  ShieldCheck,
} from "lucide-react";
import {
  getStorageRuntimeStatus,
  type StorageRuntimeErrorCode,
  type StorageRuntimeState,
  type StorageRuntimeStatus,
} from "../lib/storageRuntimeClient";

const stateLabels: Record<StorageRuntimeState, string> = {
  initializing: "Initializing",
  healthy: "Healthy",
  warning: "Warning",
  unavailable: "Unavailable",
  migration_failed: "Migration Failed",
  integrity_failed: "Integrity Failed",
  locked: "Locked",
  permission_denied: "Permission Denied",
  resource_limited: "Resource Limited",
};

const errorGuidance: Record<StorageRuntimeErrorCode, string> = {
  path_invalid: "The approved local storage location is unavailable.",
  permission_denied: "Restore private access to the app data directory.",
  locked: "Close other app instances and refresh this status.",
  busy_timeout: "Local storage remained busy beyond its bounded deadline.",
  migration_mismatch: "Migration evidence does not match the embedded schema.",
  newer_schema: "This app cannot safely open a newer storage schema.",
  migration_failed: "The store was not modified further. Review migration evidence.",
  integrity_failed: "The database was preserved for controlled recovery.",
  resource_limit: "Free local disk space before continuing.",
  internal: "Restart the app. Existing local data is preserved.",
};

function formatBytes(value: number | null): string {
  if (value === null) return "Unknown";
  if (value < 1024) return `${value} B`;
  const units = ["KiB", "MiB", "GiB", "TiB"];
  let size = value / 1024;
  let unit = units[0];
  for (let index = 1; index < units.length && size >= 1024; index += 1) {
    size /= 1024;
    unit = units[index];
  }
  return `${size.toFixed(size >= 10 ? 1 : 2)} ${unit}`;
}

function persistenceLabel(status: StorageRuntimeStatus): string {
  if (status.persistence_state === "created_new") return "New local store initialized";
  if (status.persistence_state === "reopened_existing") return "Existing local store reopened";
  return "Unavailable";
}

export function StorageRuntimeCard() {
  const [status, setStatus] = useState<StorageRuntimeStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [adapterError, setAdapterError] = useState(false);

  const refresh = useCallback(async () => {
    setLoading(true);
    setAdapterError(false);
    try {
      setStatus(await getStorageRuntimeStatus());
    } catch {
      setStatus(null);
      setAdapterError(true);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const state = status?.state ?? "unavailable";
  const positive = state === "healthy";
  const pending = state === "initializing" || loading;

  return (
    <section className="glass p-6 space-y-5 border-white/5" data-testid="storage-runtime-card">
      <div className="flex items-start justify-between gap-4 border-b border-white/5 pb-4">
        <div className="flex items-start gap-3">
          <div className="rounded-xl border border-blue-500/20 bg-blue-500/10 p-2 text-blue-400">
            <Database size={16} />
          </div>
          <div>
            <h2 className="text-[10px] font-bold uppercase tracking-[0.2em] text-white/60">
              Storage Runtime
            </h2>
            <p className="mt-1 text-[9px] font-mono text-white/30">
              SQLite · Local device only
            </p>
          </div>
        </div>
        <button
          type="button"
          onClick={() => void refresh()}
          disabled={loading}
          aria-label="Refresh local storage status"
          className="rounded-lg border border-white/10 p-2 text-white/40 transition-colors hover:text-white disabled:opacity-30"
        >
          <RefreshCw size={14} className={loading ? "animate-spin" : ""} />
        </button>
      </div>

      <div className="flex items-center justify-between rounded-xl border border-white/5 bg-white/[0.02] p-3">
        <div className="flex items-center gap-2">
          {pending ? (
            <Loader2 size={14} className="animate-spin text-blue-400" />
          ) : positive ? (
            <CheckCircle size={14} className="text-emerald-400" />
          ) : (
            <AlertTriangle size={14} className="text-amber-400" />
          )}
          <div>
            <div className="text-[8px] font-black uppercase tracking-widest text-white/20">
              Status
            </div>
            <div className="mt-0.5 text-[11px] font-bold text-white/70">
              {loading ? "Loading status" : adapterError ? "Unavailable" : stateLabels[state]}
            </div>
          </div>
        </div>
        <div className="text-right">
          <div className="text-[8px] font-black uppercase tracking-widest text-white/20">
            Database Health
          </div>
          <span
            className={`mt-1 inline-block rounded border px-2 py-0.5 text-[8px] font-black uppercase tracking-widest ${
              positive
                ? "border-emerald-500/20 bg-emerald-500/10 text-emerald-400"
                : "border-amber-500/20 bg-amber-500/10 text-amber-400"
            }`}
          >
            {status?.database_health ?? "error"}
          </span>
        </div>
      </div>

      <dl className="grid grid-cols-2 gap-3 text-left">
        <div className="rounded-xl border border-white/5 bg-white/[0.02] p-3">
          <dt className="text-[8px] font-black uppercase tracking-widest text-white/20">
            Initialized
          </dt>
          <dd className="mt-1 text-[11px] font-mono text-white/60">
            {status ? (status.initialized ? "Yes" : "No") : "Unavailable"}
          </dd>
        </div>
        <div className="rounded-xl border border-white/5 bg-white/[0.02] p-3">
          <dt className="text-[8px] font-black uppercase tracking-widest text-white/20">Schema</dt>
          <dd className="mt-1 text-[11px] font-mono text-white/60">
            {status?.schema_version == null ? "Unavailable" : `v${status.schema_version}`}
          </dd>
        </div>
        <div className="rounded-xl border border-white/5 bg-white/[0.02] p-3">
          <dt className="text-[8px] font-black uppercase tracking-widest text-white/20">
            Database size
          </dt>
          <dd className="mt-1 text-[11px] font-mono text-white/60">
            {formatBytes(status?.database_size_bytes ?? null)}
          </dd>
        </div>
        <div className="rounded-xl border border-white/5 bg-white/[0.02] p-3">
          <dt className="text-[8px] font-black uppercase tracking-widest text-white/20">SQLite</dt>
          <dd className="mt-1 text-[11px] font-mono text-white/60">
            {status?.sqlite_version ?? "Unavailable"}
          </dd>
        </div>
        <div className="rounded-xl border border-white/5 bg-white/[0.02] p-3">
          <dt className="text-[8px] font-black uppercase tracking-widest text-white/20">
            Persistence
          </dt>
          <dd className="mt-1 text-[10px] font-mono text-white/60">
            {status ? persistenceLabel(status) : "Unavailable"}
          </dd>
        </div>
      </dl>
      {!status?.initialized && !pending && (
        <p className="rounded-xl border border-amber-500/10 bg-amber-500/[0.04] p-3 text-[10px] leading-relaxed text-amber-200/60">
          {adapterError
            ? "The local status adapter could not be reached."
            : status?.error_code
              ? errorGuidance[status.error_code]
              : "The local store is not ready."}
        </p>
      )}

      <div className="flex items-center gap-2 border-t border-white/5 pt-4 text-[9px] font-bold uppercase tracking-widest text-white/25">
        <ShieldCheck size={12} className="text-emerald-400/60" />
        No cloud sync
      </div>
    </section>
  );
}
