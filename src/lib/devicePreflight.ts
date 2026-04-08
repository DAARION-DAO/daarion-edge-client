/**
 * devicePreflight.ts
 * Collects a privacy-respecting capability snapshot from the browser.
 *
 * Rules:
 * - No raw UA stored to DB
 * - No canvas/audio fingerprinting
 * - deviceMemory is already coarse/rounded by browser (privacy-preserving)
 * - Mic/camera permissions via Permissions API with fallback to "unknown"
 */

export type PlatformClass  = "desktop" | "tablet" | "mobile";
export type OSFamily       = "macos" | "windows" | "linux" | "ios" | "android" | "unknown";
export type BrowserFamily  = "chromium" | "safari" | "firefox" | "edge" | "unknown";
export type DisplayMode    = "browser" | "standalone" | "minimal-ui" | "fullscreen" | "window-controls-overlay" | "unknown";
export type PermState      = "granted" | "denied" | "prompt" | "unknown";
export type NetworkType    = "slow-2g" | "2g" | "3g" | "4g" | "unknown";
export type ScreenClass    = "compact" | "medium" | "wide";
export type PreflightIntent =
  | "install" | "text_chat" | "multimodal_chat" | "voice_chat" | "voice_ceremony";

export interface DeviceCapabilities {
  platform_class:           PlatformClass;
  os_family:                OSFamily;
  browser_family:           BrowserFamily;
  display_mode:             DisplayMode;
  is_pwa_installed:         boolean;

  secure_context:           boolean;
  logical_cpu_cores:        number | null;
  approx_device_memory_gb:  number | null;

  webgpu_available:         boolean;
  webrtc_available:         boolean;
  media_recorder_available: boolean;
  preferred_audio_mime:     string | null;

  microphone_permission:    PermState;
  camera_permission:        PermState;

  online:                   boolean;
  effective_network_type:   NetworkType;
  screen_class:             ScreenClass;
}

export interface PreflightRequest {
  client:  { app: string; version: string; build: string };
  context: { route: string; intent: PreflightIntent };
  device:  DeviceCapabilities;
}

// ── Display mode ──────────────────────────────────────────────────────────────
function detectDisplayMode(): DisplayMode {
  const modes: DisplayMode[] = [
    "window-controls-overlay", "standalone", "minimal-ui", "fullscreen", "browser",
  ];
  for (const m of modes) {
    if (window.matchMedia?.(`(display-mode: ${m})`)?.matches) return m;
  }
  return "unknown";
}

// ── OS + Browser ──────────────────────────────────────────────────────────────
function detectOS(ua: string): OSFamily {
  if (/iphone|ipad|ipod/.test(ua)) return "ios";
  if (/android/.test(ua))          return "android";
  if (/mac os x|macintosh/.test(ua)) return "macos";
  if (/windows/.test(ua))          return "windows";
  if (/linux/.test(ua))            return "linux";
  return "unknown";
}

function detectBrowser(ua: string): BrowserFamily {
  if (/edg\//.test(ua))                              return "edge";
  if (/chrome|chromium/.test(ua) && !/edg\//.test(ua)) return "chromium";
  if (/safari/.test(ua) && !/chrome/.test(ua))      return "safari";
  if (/firefox/.test(ua))                            return "firefox";
  return "unknown";
}

function detectPlatformClass(): PlatformClass {
  const w = window.innerWidth;
  if (w < 768)  return "mobile";
  if (w < 1024) return "tablet";
  return "desktop";
}

function detectScreenClass(): ScreenClass {
  const w = window.innerWidth;
  if (w < 640)  return "compact";
  if (w < 1280) return "medium";
  return "wide";
}

// ── Permission query (safe fallback) ─────────────────────────────────────────
async function queryPermission(name: PermissionName): Promise<PermState> {
  try {
    const result = await navigator.permissions?.query({ name });
    return (result?.state ?? "unknown") as PermState;
  } catch {
    return "unknown";
  }
}

// ── Audio MIME ────────────────────────────────────────────────────────────────
function bestAudioMime(): string | null {
  if (typeof window.MediaRecorder === "undefined") return null;
  const mimes = [
    "audio/webm;codecs=opus",
    "audio/webm",
    "audio/mp4",
    "audio/ogg;codecs=opus",
  ];
  return mimes.find((m) => MediaRecorder.isTypeSupported(m)) ?? null;
}

// ── Main collector ────────────────────────────────────────────────────────────
export async function collectDevicePreflight(
  intent: PreflightIntent,
  route: string,
): Promise<PreflightRequest> {
  const nav = navigator as Navigator & {
    deviceMemory?: number;
    connection?: { effectiveType?: string };
    gpu?: unknown;
  };

  const ua = navigator.userAgent.toLowerCase();
  const displayMode = detectDisplayMode();
  const mediaRecorderAvailable = typeof window.MediaRecorder !== "undefined";

  const [micPermission, camPermission] = await Promise.all([
    queryPermission("microphone" as PermissionName),
    queryPermission("camera"    as PermissionName),
  ]);

  const rawNetwork = nav.connection?.effectiveType;
  const effectiveNetworkType: NetworkType =
    rawNetwork === "slow-2g" || rawNetwork === "2g" ||
    rawNetwork === "3g"      || rawNetwork === "4g"
      ? (rawNetwork as NetworkType)
      : "unknown";

  return {
    client: {
      app:     "daarion-edge-pwa",
      version: import.meta.env.VITE_APP_VERSION ?? "0.1.0",
      build:   import.meta.env.VITE_APP_BUILD   ?? "dev",
    },
    context: { route, intent },
    device: {
      platform_class:          detectPlatformClass(),
      os_family:               detectOS(ua),
      browser_family:          detectBrowser(ua),
      display_mode:            displayMode,
      is_pwa_installed:        displayMode !== "browser" && displayMode !== "unknown",

      secure_context:          window.isSecureContext === true,
      logical_cpu_cores:       typeof navigator.hardwareConcurrency === "number"
                                 ? navigator.hardwareConcurrency : null,
      approx_device_memory_gb: typeof nav.deviceMemory === "number"
                                 ? nav.deviceMemory : null,

      webgpu_available:        !!nav.gpu,
      webrtc_available:        typeof window.RTCPeerConnection !== "undefined",
      media_recorder_available: mediaRecorderAvailable,
      preferred_audio_mime:    bestAudioMime(),

      microphone_permission:   micPermission,
      camera_permission:       camPermission,

      online:                  navigator.onLine,
      effective_network_type:  effectiveNetworkType,
      screen_class:            detectScreenClass(),
    },
  };
}
