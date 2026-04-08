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

export interface InstallSource {
  runtime: string;
  upstream_tag: string;
  local_alias: string;
  estimated_download_gb: number;
}

/** A single model candidate — populated from gateway models_registry.json */
export interface CandidateModel {
  id:                    string;    // Internal stable ID e.g. "qwen35-4b-prod"
  family:                string;    // "qwen" | "gemma4" | ...
  tier:                  string;    // "tiny" | "balanced" | "powerful"
  role:                  string;    // "agent" | "helper"
  stability:             string;    // "experimental" | "stable" | "production"
  capabilities:          string[];  // ["text","vision","tools","code","reasoning"]
  install_sources:       InstallSource[];
  is_recommended:        boolean;
}

/** Present only when profile === "local_accel_candidate" */
export interface EdgeUpgrade {
  recommended_runtime:   string;          // "tauri"
  recommended_model:     string;          // id of the is_recommended=True candidate
  estimated_download_gb: number;
  reason:                string;          // "10 cores · ~16GB RAM · WebGPU: yes"
  candidate_models:      CandidateModel[]; // full device-to-model policy matrix
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

// ── Registry API Strategy (3-Level Fallback) ───────────────────────────────────

function openDB(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const req = indexedDB.open("DaarionEdgeDB", 1);
    req.onupgradeneeded = () => {
      req.result.createObjectStore("registry_cache");
    };
    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error);
  });
}

async function setCachedRegistry(data: unknown) {
  try {
    const db = await openDB();
    db.transaction("registry_cache", "readwrite")
      .objectStore("registry_cache")
      .put(data, "last_known_good_registry");
  } catch (e) {
    console.warn("[RegistryCache] IndexedDB save failed", e);
  }
}

async function getCachedRegistry(): Promise<unknown | null> {
  try {
    const db = await openDB();
    return new Promise((resolve, reject) => {
      const req = db.transaction("registry_cache", "readonly")
        .objectStore("registry_cache")
        .get("last_known_good_registry");
      req.onsuccess = () => resolve(req.result || null);
      req.onerror = () => reject(req.error);
    });
  } catch (e) {
    console.warn("[RegistryCache] IndexedDB read failed", e);
    return null;
  }
}

export async function fetchRegistryStrategy(): Promise<unknown | null> {
  // 1. Network Try (supports ETag / 304 via browser native caching)
  try {
    const res = await fetch(`${API_BASE}/models/registry`, {
      method: "GET",
      signal: AbortSignal.timeout(5000),
    });
    if (res.ok) {
      const data = await res.json();
      await setCachedRegistry(data);
      return data;
    }
  } catch (e) {
    console.warn("[RegistryStrategy] Network fetch failed, trying DB cache", e);
  }

  // 2. Local Cache Try (IndexedDB)
  const cached = await getCachedRegistry();
  if (cached) {
    console.info("[RegistryStrategy] Loaded from IndexedDB cache");
    return cached;
  }

  // 3. Bundled Fallback Try
  try {
    const fallbackRes = await fetch("/fallback_registry.json");
    if (fallbackRes.ok) {
      console.info("[RegistryStrategy] Loaded from bundled fallback");
      return await fallbackRes.json();
    }
  } catch (e) {
    console.error("[RegistryStrategy] Bundled fallback failed", e);
  }

  return null;
}
