export type BackendHealthState =
  | "pairing_required"
  | "offline"
  | "contract_invalid"
  | "version_mismatch"
  | "online"
  | "degraded"
  | "maintenance";

export interface BackendHealthStatus {
  state: BackendHealthState;
  checked_at: string;
  backend_label: string | null;
  environment: string | null;
  http_status: number | null;
  backend_status: string | null;
  backend_version: string | null;
  edge_protocol_version: string | null;
  min_edge_client_version: string | null;
  server_time: string | null;
  capabilities: Record<string, boolean> | null;
  message: string;
}

export function backendHealthLabel(state: BackendHealthState | null | undefined): string {
  switch (state) {
    case "pairing_required":
      return "Pairing required";
    case "offline":
      return "Offline";
    case "contract_invalid":
      return "Contract invalid";
    case "version_mismatch":
      return "Version mismatch";
    case "online":
      return "Online";
    case "degraded":
      return "Degraded";
    case "maintenance":
      return "Maintenance";
    default:
      return "Checking backend";
  }
}

export function backendHealthTextClass(state: BackendHealthState | null | undefined): string {
  switch (state) {
    case "online":
      return "text-emerald-400";
    case "degraded":
      return "text-amber-300";
    case "maintenance":
      return "text-blue-300";
    case "pairing_required":
    case "offline":
    case "contract_invalid":
    case "version_mismatch":
      return "text-red-300";
    default:
      return "text-white/35";
  }
}

export function backendHealthDotClass(state: BackendHealthState | null | undefined): string {
  switch (state) {
    case "online":
      return "bg-emerald-500 shadow-[0_0_10px_#10b981]";
    case "degraded":
      return "bg-amber-400 shadow-[0_0_10px_#fbbf24]";
    case "maintenance":
      return "bg-blue-400 shadow-[0_0_10px_#60a5fa]";
    case "pairing_required":
    case "offline":
    case "contract_invalid":
    case "version_mismatch":
      return "bg-red-400 shadow-[0_0_10px_#f87171]";
    default:
      return "bg-white/25";
  }
}

export function backendHealthPanelClass(state: BackendHealthState | null | undefined): string {
  switch (state) {
    case "online":
      return "border-emerald-500/15 bg-emerald-500/5";
    case "degraded":
      return "border-amber-500/20 bg-amber-500/10";
    case "maintenance":
      return "border-blue-500/20 bg-blue-500/10";
    case "pairing_required":
    case "offline":
    case "contract_invalid":
    case "version_mismatch":
      return "border-red-500/20 bg-red-500/10";
    default:
      return "border-white/10 bg-white/[0.03]";
  }
}

export function backendHealthFallbackStatus(
  state: BackendHealthState,
  message: string,
): BackendHealthStatus {
  return {
    state,
    checked_at: new Date().toISOString(),
    backend_label: null,
    environment: null,
    http_status: null,
    backend_status: null,
    backend_version: null,
    edge_protocol_version: null,
    min_edge_client_version: null,
    server_time: null,
    capabilities: null,
    message,
  };
}
