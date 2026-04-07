/**
 * captureAdapters.ts
 * Adapter-based audio capture for Voice Ceremony.
 * Single interface, two implementations: browser (PWA) and Tauri (desktop).
 * B-first / C-ready: browser is default, Tauri is wired later.
 */

export interface VoiceCaptureAdapter {
  /** Record audio for `durationSec` seconds. Returns a Blob (audio/webm or audio/ogg). */
  start(durationSec: number): Promise<Blob>;
}

// ── Browser Adapter (PWA / default) ──────────────────────────────────────────

export const browserCaptureAdapter: VoiceCaptureAdapter = {
  async start(durationSec: number): Promise<Blob> {
    const stream = await navigator.mediaDevices.getUserMedia({ audio: true, video: false });

    // Pick best supported MIME
    const mimeType = MediaRecorder.isTypeSupported("audio/webm;codecs=opus")
      ? "audio/webm;codecs=opus"
      : MediaRecorder.isTypeSupported("audio/webm")
      ? "audio/webm"
      : "audio/ogg";

    const recorder = new MediaRecorder(stream, { mimeType });
    const chunks: Blob[] = [];

    recorder.ondataavailable = (e) => {
      if (e.data.size > 0) chunks.push(e.data);
    };

    return new Promise((resolve, reject) => {
      recorder.onerror = (e) => {
        stream.getTracks().forEach((t) => t.stop());
        reject(new Error(`MediaRecorder error: ${e}`));
      };

      recorder.onstop = () => {
        stream.getTracks().forEach((t) => t.stop());
        resolve(new Blob(chunks, { type: mimeType }));
      };

      recorder.start();
      setTimeout(() => recorder.stop(), durationSec * 1000);
    });
  },
};

// ── Tauri Adapter (C-ready placeholder) ──────────────────────────────────────
// When Tauri path is ready:
//   1. invoke("record_voice_imprint") writes .wav to app data dir
//   2. Read file as bytes, construct Blob
//   3. Return same interface
//
// For now: falls back to browser adapter so Step 4 works in both contexts.

export const tauriCaptureAdapter: VoiceCaptureAdapter = {
  async start(durationSec: number): Promise<Blob> {
    // TODO: replace with Rust invoke + file read when Tauri pipe is ready
    console.info("[tauriCaptureAdapter] Falling back to browserCaptureAdapter (path not yet wired)");
    return browserCaptureAdapter.start(durationSec);
  },
};

// ── Runtime selector ──────────────────────────────────────────────────────────

const isTauri = typeof (window as any).__TAURI__ !== "undefined";

export const activeCaptureAdapter: VoiceCaptureAdapter = isTauri
  ? tauriCaptureAdapter
  : browserCaptureAdapter;
