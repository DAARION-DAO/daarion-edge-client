import { invoke } from "@tauri-apps/api/core";

interface BackendConfigStatus {
  configured: boolean;
  backend_url: string | null;
  message: string;
}

export async function getEffectiveBackendUrl(): Promise<string> {
  const status = await invoke<BackendConfigStatus>("get_backend_config_status");
  if (!status.configured || !status.backend_url) {
    throw new Error(status.message || "Pairing required.");
  }
  return status.backend_url.replace(/\/$/, "");
}
