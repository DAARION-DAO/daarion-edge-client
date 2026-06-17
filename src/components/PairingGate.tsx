import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { AlertTriangle, CheckCircle, ChevronDown, KeyRound, Loader2, ServerCog } from "lucide-react";

export interface PairingState {
  state: "paired" | "unpaired";
  backend_url: string | null;
  label: string | null;
  environment: string | null;
  source: "invite_payload" | "manual_advanced" | "env_import" | null;
  created_at: string | null;
  updated_at: string | null;
  connection_status: string;
  message: string;
}

interface PairingGateProps {
  onPaired: (state: PairingState) => void;
  message?: string | null;
}

export function PairingGate({ onPaired, message }: PairingGateProps) {
  const [inviteInput, setInviteInput] = useState("");
  const [manualUrl, setManualUrl] = useState("");
  const [advancedOpen, setAdvancedOpen] = useState(false);
  const [pairing, setPairing] = useState(false);
  const [advancedPairing, setAdvancedPairing] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function submitInvite() {
    if (!inviteInput.trim()) {
      setError("Invitation code is required.");
      return;
    }

    setPairing(true);
    setError(null);
    try {
      const state = await invoke<PairingState>("pair_backend", {
        input: inviteInput,
        manualAdvanced: false,
      });
      onPaired(state);
    } catch (e) {
      setError(String(e));
    } finally {
      setPairing(false);
    }
  }

  async function submitManualUrl() {
    if (!manualUrl.trim()) {
      setError("Developer backend URL is required.");
      return;
    }

    setAdvancedPairing(true);
    setError(null);
    try {
      const state = await invoke<PairingState>("pair_backend", {
        input: manualUrl,
        manualAdvanced: true,
      });
      onPaired(state);
    } catch (e) {
      setError(String(e));
    } finally {
      setAdvancedPairing(false);
    }
  }

  return (
    <div className="min-h-screen bg-[#020202] text-white flex items-center justify-center p-6 font-sans selection:bg-blue-500/30 overflow-hidden">
      <div className="relative z-10 w-full max-w-md">
        <div className="glass border-white/10 p-7">
          <div className="mb-6 flex items-start gap-4">
            <div className="flex h-12 w-12 flex-shrink-0 items-center justify-center rounded-2xl border border-blue-500/25 bg-blue-500/10">
              <KeyRound size={22} className="text-blue-400" />
            </div>
            <div>
              <p className="mb-1 text-[9px] font-black uppercase tracking-[0.35em] text-blue-400/60">
                DAARION Edge
              </p>
              <h1 className="text-xl font-black tracking-tight text-white">Connect DAARION Edge</h1>
              <p className="mt-2 text-xs leading-relaxed text-white/40">
                Connect this device to the operator profile before Genesis registration.
              </p>
            </div>
          </div>

          {message && (
            <div className="mb-4 flex items-start gap-2 rounded-xl border border-amber-500/20 bg-amber-500/10 px-3 py-2">
              <AlertTriangle size={13} className="mt-0.5 flex-shrink-0 text-amber-400" />
              <p className="text-[10px] leading-relaxed text-amber-100/70">{message}</p>
            </div>
          )}

          <div className="space-y-3">
            <textarea
              value={inviteInput}
              onChange={(event) => setInviteInput(event.target.value)}
              placeholder="Paste invitation code or link"
              className="h-28 w-full resize-none rounded-xl border border-white/10 bg-black/50 px-4 py-3 text-sm text-white outline-none transition-all placeholder:text-white/20 focus:border-blue-500/50"
            />
            <button
              onClick={submitInvite}
              disabled={pairing}
              className="flex w-full items-center justify-center gap-2 rounded-xl bg-blue-600 px-4 py-3 text-[10px] font-black uppercase tracking-[0.2em] text-white shadow-[0_0_20px_rgba(37,99,235,0.25)] transition-all hover:bg-blue-500 disabled:cursor-not-allowed disabled:opacity-50"
            >
              {pairing ? <Loader2 size={14} className="animate-spin" /> : <CheckCircle size={14} />}
              Enter invitation code
            </button>
          </div>

          {error && (
            <div className="mt-4 flex items-start gap-2 rounded-xl border border-red-500/20 bg-red-500/10 px-3 py-2">
              <AlertTriangle size={13} className="mt-0.5 flex-shrink-0 text-red-400" />
              <p className="text-[10px] leading-relaxed text-red-200/70">{error}</p>
            </div>
          )}

          <div className="mt-5 border-t border-white/5 pt-4">
            <button
              onClick={() => setAdvancedOpen((open) => !open)}
              className="flex w-full items-center justify-between rounded-xl border border-white/5 bg-white/[0.02] px-3 py-2 text-left text-[10px] font-black uppercase tracking-widest text-white/35 transition-colors hover:text-white/60"
            >
              <span className="flex items-center gap-2">
                <ServerCog size={12} />
                Advanced / Developer settings
              </span>
              <ChevronDown
                size={13}
                className={`transition-transform ${advancedOpen ? "rotate-180" : ""}`}
              />
            </button>

            {advancedOpen && (
              <div className="mt-3 space-y-3 rounded-xl border border-white/5 bg-black/30 p-3">
                <input
                  type="url"
                  value={manualUrl}
                  onChange={(event) => setManualUrl(event.target.value)}
                  placeholder="https://staging.daarion.city"
                  className="w-full rounded-xl border border-white/10 bg-black/50 px-3 py-2.5 font-mono text-xs text-white outline-none transition-all placeholder:text-white/20 focus:border-amber-500/40"
                />
                <button
                  onClick={submitManualUrl}
                  disabled={advancedPairing}
                  className="flex w-full items-center justify-center gap-2 rounded-xl border border-amber-500/25 bg-amber-500/10 px-3 py-2.5 text-[10px] font-black uppercase tracking-widest text-amber-200 transition-colors hover:border-amber-500/45 disabled:cursor-not-allowed disabled:opacity-50"
                >
                  {advancedPairing ? <Loader2 size={13} className="animate-spin" /> : <ServerCog size={13} />}
                  Use developer backend
                </button>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
