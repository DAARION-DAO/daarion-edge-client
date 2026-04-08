/**
 * preflightApi.ts
 * Typed client for POST /device/preflight.
 * Graceful fallback: if API unreachable → safe default (remote_multimodal_ready).
 */
import { collectDevicePreflight, PreflightIntent, PreflightRequest } from "./devicePreflight";

const API_BASE = (
  import.meta.env.VITE_GENESIS_API_BASE ?? "https://api.daarion.city"
).replace(/\/$/, "");

// ── Response types (mirror backend) ──────────────────────────────────────────

export type PreflightProfile =
  | "remote_text_only"
  | "remote_multimodal_ready"
  | "remote_voice_ready"
  | "remote_voice_degraded"
  | "local_accel_candidate";

export interface CapabilityFlags {
  text_chat:               boolean;
  matrix_chat:             boolean;
  multimodal_upload:       boolean;
  voice_chat:              boolean;
  voice_ceremony:          boolean;
  install_prompt:          boolean;
  local_embedding_candidate: boolean;
  local_model_loading:     boolean;
}

export interface UIFlags {
  show_install_banner:   boolean;
  show_mic_cta:          boolean;
  show_push_to_talk:     boolean;
  show_voice_ceremony:   boolean;
  show_matrix_rooms:     boolean;
  show_low_power_mode:   boolean;
  show_edge_upgrade_cta: boolean;
}

export interface RecommendedAudio {
  mime_type:        string | null;
  max_duration_sec: number;
  sample_path:      "none" | "browser_mediarecorder" | "browser_webrtc" | "tauri_capture";
}

/** Present only when profile === "local_accel_candidate" */
export interface EdgeUpgrade {
  recommended_runtime:   string;  // "tauri"
  recommended_model:     string;  // e.g. "qwen3.5:4b"
  estimated_download_gb: number;
  reason:                string;  // "8 cores · ~16GB RAM · WebGPU: yes"
}

export interface PreflightResponse {
  profile:           PreflightProfile;
  capabilities:      CapabilityFlags;
  ui_flags:          UIFlags;
  reasons:           string[];
  recommended_audio: RecommendedAudio;
  upgrade?:          EdgeUpgrade;
  /** Echo of the device field from the request — included for client-side mic gate UI */
  device?: {
    microphone_permission: "granted" | "denied" | "prompt" | "unknown";
    [key: string]: unknown;
  };
}

// ── Safe default (network unavailable / beta fallback) ───────────────────────
const DEFAULT_RESPONSE: PreflightResponse = {
  profile: "remote_multimodal_ready",
  capabilities: {
    text_chat: true, matrix_chat: true, multimodal_upload: true,
    voice_chat: false, voice_ceremony: false, install_prompt: false,
    local_embedding_candidate: false, local_model_loading: false,
  },
  ui_flags: {
    show_install_banner: false, show_mic_cta: false, show_push_to_talk: false,
    show_voice_ceremony: false, show_matrix_rooms: true, show_low_power_mode: false,
    show_edge_upgrade_cta: false,
  },
  reasons: ["Preflight API unavailable — using safe default."],
  recommended_audio: { mime_type: null, max_duration_sec: 0, sample_path: "none" },
};

// ── API call ──────────────────────────────────────────────────────────────────
export async function runPreflight(
  intent: PreflightIntent,
  route: string,
): Promise<PreflightResponse> {
  let req: PreflightRequest;
  try {
    req = await collectDevicePreflight(intent, route);
  } catch (e) {
    console.warn("[Preflight] Collection failed:", e);
    return DEFAULT_RESPONSE;
  }

  try {
    const res = await fetch(`${API_BASE}/device/preflight`, {
      method:  "POST",
      headers: { "Content-Type": "application/json" },
      body:    JSON.stringify(req),
      signal:  AbortSignal.timeout(5000), // 5s max — don't block UX
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return (await res.json()) as PreflightResponse;
  } catch (e) {
    console.warn("[Preflight] API unreachable, using safe default:", e);
    return DEFAULT_RESPONSE;
  }
}

// ── Convenience helpers ───────────────────────────────────────────────────────

export function canRunVoiceCeremony(p: PreflightResponse): boolean {
  return p.capabilities.voice_ceremony;
}

export function isEdgeCandidate(p: PreflightResponse): boolean {
  return p.profile === "local_accel_candidate";
}

export function bestMime(p: PreflightResponse): string {
  return p.recommended_audio.mime_type ?? "audio/webm;codecs=opus";
}
