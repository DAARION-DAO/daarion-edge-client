/**
 * voiceCeremonyApi.ts
 * Typed client for the three canonical Voice Ceremony endpoints.
 * All paths relative to VITE_GENESIS_API_BASE (default: https://api.daarion.city)
 */

const API_BASE = (import.meta.env.VITE_GENESIS_API_BASE ?? "https://api.daarion.city").replace(/\/$/, "");

export interface ChallengeResponse {
  ok: boolean;
  challenge_id: string;
  phrase: string;
  agent_name: string;
  expires_at: string;
}

export interface UploadResponse {
  ok: boolean;
  imprint_id: string;
  status: "captured";
  agent_name: string;
  sha256: string;
  file_key: string;
}

export interface SealResponse {
  ok: boolean;
  seal_id: string;
  imprint_id: string;
  status: "sealed";
  passport_id: string | null;
  timestamp: string;
  event: "VOICE_IMPRINT_SEALED";
}

// ── 1. Challenge ─────────────────────────────────────────────────────────────

export async function fetchChallenge(agentName: string): Promise<ChallengeResponse> {
  const res = await fetch(`${API_BASE}/genesis/voice-imprint/challenge`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ agent_name: agentName }),
  });
  if (!res.ok) throw new Error(`Challenge failed: ${res.status} ${await res.text()}`);
  return res.json();
}

// ── 2. Upload ─────────────────────────────────────────────────────────────────

export async function uploadVoiceImprint(
  challengeId: string,
  agentName: string,
  audioBlob: Blob,
  creatorWallet?: string,
): Promise<UploadResponse> {
  const form = new FormData();
  form.append("challenge_id", challengeId);
  form.append("agent_name", agentName);
  if (creatorWallet) form.append("creator_wallet", creatorWallet);
  form.append("audio", audioBlob, "voice-imprint.webm");

  const res = await fetch(`${API_BASE}/genesis/voice-imprint/upload`, {
    method: "POST",
    body: form,
  });
  if (!res.ok) throw new Error(`Upload failed: ${res.status} ${await res.text()}`);
  return res.json();
}

// ── 3. Seal ───────────────────────────────────────────────────────────────────

export async function sealCeremony(
  imprintId: string,
  challengeId: string,
  agentPassportId?: string,
): Promise<SealResponse> {
  const res = await fetch(`${API_BASE}/genesis/voice-imprint/complete`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      imprint_id: imprintId,
      challenge_id: challengeId,
      agent_passport_id: agentPassportId ?? null,
    }),
  });
  if (!res.ok) throw new Error(`Seal failed: ${res.status} ${await res.text()}`);
  return res.json();
}
