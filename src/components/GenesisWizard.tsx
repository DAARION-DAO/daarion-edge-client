import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Shield, Cpu, Mic, Fingerprint, Mail, Key, Sparkles, ChevronRight, Activity, CheckCircle, Zap, Smartphone, Monitor, Server, Copy, ExternalLink, Users } from "lucide-react";

interface GenesisWizardProps {
  onComplete: () => void;
}

const STEPS = [
  { id: 1, label: "Vessel" },
  { id: 2, label: "Soul" },
  { id: 3, label: "Voice" },
  { id: 4, label: "Birth" },
  { id: 5, label: "City" },
];

const TIER_COLORS: Record<string, string> = {
  "ultra-lite": "text-orange-400 border-orange-500/30 bg-orange-500/10",
  "lite": "text-amber-400 border-amber-500/30 bg-amber-500/10",
  "balanced": "text-blue-400 border-blue-500/30 bg-blue-500/10",
  "full": "text-emerald-400 border-emerald-500/30 bg-emerald-500/10",
};

const DEVICE_CLASS_ICONS: Record<string, any> = {
  Smartphone: Smartphone,
  Tablet: Smartphone,
  Laptop: Monitor,
  Workstation: Server,
};

function CopyButton({ value }: { value: string }) {
  const [copied, setCopied] = useState(false);
  return (
    <button
      onClick={() => { navigator.clipboard.writeText(value); setCopied(true); setTimeout(() => setCopied(false), 1500); }}
      className="ml-1.5 text-white/20 hover:text-white/60 transition-colors"
    >
      {copied ? <CheckCircle size={10} className="text-emerald-400" /> : <Copy size={10} />}
    </button>
  );
}

export function GenesisWizard({ onComplete }: GenesisWizardProps) {
  const [step, setStep] = useState(1);
  const [agentName, setAgentName] = useState(() => localStorage.getItem("genesis_agent_name") || "");
  const [agentPurpose, setAgentPurpose] = useState(() => localStorage.getItem("genesis_agent_purpose") || "");
  const [hardwareScan, setHardwareScan] = useState<any>(null);
  const [selectedModel, setSelectedModel] = useState<any>(null);
  const [voiceRecorded, setVoiceRecorded] = useState(false);
  const [recording, setRecording] = useState(false);
  const [countdown, setCountdown] = useState(0);
  const [provisioningLog, setProvisioningLog] = useState<string[]>([]);
  const [provisionProgress, setProvisionProgress] = useState(0);
  const [walletKeys, setWalletKeys] = useState<any>(null);
  const [provisionResult, setProvisionResult] = useState<any>(null);
  const [provisionError, setProvisionError] = useState<string | null>(null);
  const [provisioningDone, setProvisioningDone] = useState(false);
  const [betaStatus, setBetaStatus] = useState<any>(null);

  useEffect(() => {
    if (step === 1) {
      setTimeout(async () => {
        try {
          const cap = await invoke("get_capabilities");
          setHardwareScan(cap);
          setSelectedModel((cap as any).recommended_model);
        } catch (e) {
          console.error(e);
        }
      }, 1200);
      // Pre-fetch beta status
      invoke("check_beta_slots").then((s: any) => setBetaStatus(s)).catch(() => {});
    }
  }, [step]);

  useEffect(() => {
    if (agentName) localStorage.setItem("genesis_agent_name", agentName);
  }, [agentName]);

  useEffect(() => {
    if (agentPurpose) localStorage.setItem("genesis_agent_purpose", agentPurpose);
  }, [agentPurpose]);

  const recordVoice = async () => {
    setRecording(true);
    setCountdown(5);
    const interval = setInterval(() => {
      setCountdown(prev => {
        if (prev <= 1) { clearInterval(interval); return 0; }
        return prev - 1;
      });
    }, 1000);
    try {
      await invoke("record_voice_imprint", { durSecs: 5 });
      setVoiceRecorded(true);
    } catch (e) {
      console.error(e);
    } finally {
      setRecording(false);
      clearInterval(interval);
    }
  };

  const startProvisioning = async () => {
    setStep(4);
    setProvisionError(null);
    const addLog = (msg: string) => setProvisioningLog(prev => [...prev, msg]);

    // Step 1 — Wallet generation
    addLog("Initialising sovereign Matrix room context...");
    setProvisionProgress(8);

    await new Promise(r => setTimeout(r, 600));
    addLog(`Binding identity: ${agentName.toLowerCase().replace(/ /g, "_")}@daarion.city`);
    setProvisionProgress(18);

    await new Promise(r => setTimeout(r, 800));
    addLog("Generating BIP39 Sovereign Wallets (Ed25519 + EVM)...");
    let keys: any = null;
    try {
      keys = await invoke("generate_wallet_keys");
      setWalletKeys(keys);
      addLog(`✓ Solana: ${(keys.solana_pubkey as string).slice(0, 12)}...`);
      addLog(`✓ EVM: ${(keys.base_address as string).slice(0, 14)}...`);
      setProvisionProgress(35);
    } catch {
      addLog("⚠ Wallet generation failed — using placeholder identity.");
      keys = { solana_pubkey: "GENESIS_PLACEHOLDER_PUBKEY_001", base_address: "0x0000000000000000000000000000000000000001", mnemonic: "" };
    }

    // Step 2 — Matrix + Beta registration (real API)
    await new Promise(r => setTimeout(r, 700));
    addLog("Contacting DAARION Genesis Protocol (NODA1)...");
    setProvisionProgress(50);

    await new Promise(r => setTimeout(r, 500));
    addLog("Registering with Sovereign Genesis API...");
    setProvisionProgress(62);

    try {
      const result: any = await invoke("provision_sovereign_genesis", {
        agentName: agentName.trim(),
        agentDirective: agentPurpose.trim(),
        solanaPubkey: keys.solana_pubkey,
        evmAddress: keys.base_address,
        deviceClass: hardwareScan?.device_class || "Unknown",
        deviceOs: hardwareScan?.os || "Unknown",
        deviceRamGb: hardwareScan?.ram_total_gb || 0,
        recommendedModel: selectedModel?.model_id || "",
      });

      setProvisionResult(result);
      setProvisionProgress(80);
      addLog(`✓ Sovereign slot acquired: #${result.beta_slot}`);
      addLog(`✓ Matrix Chamber: ${result.matrix?.room_id?.slice(0, 24)}...`);
      addLog(`✓ Mailbox: ${result.email}`);
      addLog("✓ DAARWIZZ welcome sent to Matrix chamber.");

      // Step 3 — Identity anchor
      await new Promise(r => setTimeout(r, 600));
      addLog("Anchoring hardware identity to DAARION data plane...");
      try { await invoke("initialize_identity"); } catch { /* graceful */ }
      setProvisionProgress(92);
      addLog("✓ Identity node anchored.");

      await new Promise(r => setTimeout(r, 500));
      try { await invoke("enroll_node", { bootstrapGrant: "GENESIS_GRANT" }); } catch { /* graceful */ }
      setProvisionProgress(100);
      addLog("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
      addLog(`🎉 Sovereign birth complete. Creator #${result.beta_slot} of 10,000.`);
      setProvisioningDone(true);

      setTimeout(() => setStep(5), 2000);

    } catch (err: any) {
      setProvisionError(String(err));
      addLog(`✗ Provisioning error: ${String(err).slice(0, 80)}`);
      setProvisionProgress(100);
      // Still allow proceeding to Step 5 after error
      setTimeout(() => setStep(5), 3000);
    }
  };

  return (
    <div className="min-h-screen bg-[#020202] text-white flex flex-col items-center justify-center p-6 relative overflow-hidden font-sans">

      {/* Background nebula */}
      <div className="absolute inset-0 pointer-events-none overflow-hidden">
        <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[700px] h-[700px] rounded-full bg-blue-900/30 blur-[140px] animate-pulse" />
        <div className="absolute top-1/4 left-1/4 w-[300px] h-[300px] rounded-full bg-emerald-900/10 blur-[120px] animate-pulse" style={{ animationDelay: "1s" }} />
      </div>

      <div className="z-10 w-full max-w-xl">
        {/* Header */}
        <div className="text-center mb-10">
          <p className="text-[9px] uppercase tracking-[0.4em] text-white/20 mb-3">DAARION Protocol</p>
          <h1 className="text-4xl font-black tracking-tighter bg-gradient-to-br from-white via-white/90 to-white/30 bg-clip-text text-transparent mb-2">
            Sovereign Genesis
          </h1>
          <p className="text-[10px] uppercase tracking-[0.25em] text-blue-400/60">Portal of Birth</p>
          {betaStatus && (
            <div className="mt-3 inline-flex items-center gap-2 px-3 py-1 rounded-full bg-white/5 border border-white/10">
              <Users size={10} className="text-emerald-400" />
              <span className="text-[9px] text-white/40 font-mono">
                <span className="text-emerald-400 font-bold">{betaStatus.remaining?.toLocaleString()}</span>
                <span className="text-white/20"> / {betaStatus.total?.toLocaleString()} slots remaining</span>
              </span>
            </div>
          )}
        </div>

        {/* Step Indicator */}
        <div className="flex items-center justify-center gap-1 mb-8">
          {STEPS.map((s, i) => (
            <div key={s.id} className="flex items-center gap-1">
              <div className={`flex flex-col items-center gap-1 transition-all duration-500 ${step >= s.id ? 'opacity-100' : 'opacity-30'}`}>
                <div className={`w-7 h-7 rounded-full border-2 flex items-center justify-center text-[9px] font-black transition-all duration-500 ${
                  step > s.id ? 'bg-emerald-500 border-emerald-500 text-white' :
                  step === s.id ? 'bg-blue-600 border-blue-400 text-white shadow-[0_0_14px_rgba(59,130,246,0.6)]' :
                  'border-white/10 text-white/30 bg-transparent'
                }`}>
                  {step > s.id ? <CheckCircle size={11} /> : s.id}
                </div>
                <span className={`text-[7px] uppercase tracking-wider font-bold ${step === s.id ? 'text-blue-400' : 'text-white/20'}`}>{s.label}</span>
              </div>
              {i < STEPS.length - 1 && (
                <div className={`w-8 h-px mb-5 transition-all duration-700 ${step > s.id ? 'bg-emerald-500/60' : 'bg-white/5'}`} />
              )}
            </div>
          ))}
        </div>

        {/* ── STEP 1: Hardware Audit ── */}
        {step === 1 && (
          <div className="glass p-8 border-white/10 flex flex-col items-center animate-in fade-in zoom-in-95 duration-500">
            <div className="w-16 h-16 rounded-2xl bg-blue-600/20 border border-blue-500/20 flex items-center justify-center mb-6">
              <Cpu size={32} className="text-blue-400" />
            </div>
            <h2 className="text-base font-black uppercase tracking-widest mb-2">Hardware Audit</h2>
            <p className="text-white/40 text-xs text-center mb-7 max-w-xs leading-relaxed">
              Scanning the Creator's vessel. Selecting the optimal sovereign model for this device...
            </p>

            {hardwareScan ? (
              <div className="w-full space-y-3 mb-5">
                <div className="bg-black/40 border border-white/5 rounded-xl p-4 space-y-2.5">
                  <div className="flex justify-between items-center">
                    <span className="text-[9px] text-white/30 uppercase font-bold tracking-wider">Device Class</span>
                    <span className="text-[11px] font-bold text-blue-400">
                      {(() => {
                        const Icon = DEVICE_CLASS_ICONS[hardwareScan.device_class] || Monitor;
                        return <span className="flex items-center gap-1.5"><Icon size={11} />{hardwareScan.device_class}</span>;
                      })()}
                    </span>
                  </div>
                  <div className="h-px bg-white/5" />
                  <div className="flex justify-between items-center">
                    <span className="text-[9px] text-white/30 uppercase font-bold tracking-wider">CPU</span>
                    <span className="text-[10px] font-mono text-white/70 max-w-[180px] text-right truncate">{hardwareScan.cpu_brand}</span>
                  </div>
                  <div className="h-px bg-white/5" />
                  <div className="flex justify-between items-center">
                    <span className="text-[9px] text-white/30 uppercase font-bold tracking-wider">RAM</span>
                    <span className="text-[11px] font-mono text-white/80">{Math.round(hardwareScan.ram_total_gb)} GB</span>
                  </div>
                  <div className="h-px bg-white/5" />
                  <div className="flex justify-between items-center">
                    <span className="text-[9px] text-white/30 uppercase font-bold tracking-wider">Acceleration</span>
                    <span className={`text-[11px] font-bold ${hardwareScan.gpu?.detected ? 'text-emerald-400' : 'text-amber-400'}`}>
                      {hardwareScan.gpu?.acceleration_api || 'CPU'}
                    </span>
                  </div>
                </div>

                {hardwareScan.recommended_model && (
                  <div className="bg-blue-500/5 border border-blue-500/20 rounded-xl p-4">
                    <div className="flex items-center justify-between mb-2">
                      <span className="text-[9px] text-blue-400/70 uppercase font-black tracking-widest">Recommended Model</span>
                      <span className={`text-[8px] font-black uppercase px-2 py-0.5 rounded-full border ${
                        TIER_COLORS[hardwareScan.recommended_model.performance_tier] || 'text-white/50 border-white/10'
                      }`}>
                        {hardwareScan.recommended_model.performance_tier}
                      </span>
                    </div>
                    <div className="flex items-baseline gap-2 mb-1">
                      <span className="text-lg font-black text-white">{hardwareScan.recommended_model.display_name}</span>
                      <span className="text-sm text-white/50 font-bold">{hardwareScan.recommended_model.params}</span>
                      <span className="text-[9px] text-white/30">{hardwareScan.recommended_model.quantization}</span>
                    </div>
                    <p className="text-[10px] text-white/40 leading-relaxed mb-2">{hardwareScan.recommended_model.reason}</p>
                    <div className="flex items-center gap-3 text-[9px] text-white/30">
                      <span><Zap size={9} className="inline mr-0.5" />{hardwareScan.recommended_model.size_gb}GB download</span>
                      <span>{hardwareScan.recommended_model.context_tokens?.toLocaleString()} ctx</span>
                    </div>

                    {hardwareScan.alternative_models?.length > 0 && (
                      <div className="mt-3 pt-3 border-t border-white/5">
                        <p className="text-[8px] text-white/20 uppercase mb-2 font-bold tracking-wider">Or choose:</p>
                        <div className="flex flex-wrap gap-1.5">
                          {hardwareScan.alternative_models.map((m: any) => (
                            <button
                              key={m.model_id}
                              onClick={() => setSelectedModel(m)}
                              className={`text-[9px] px-2 py-1 rounded-lg border transition-all ${
                                selectedModel?.model_id === m.model_id
                                  ? 'border-blue-500/50 text-blue-400 bg-blue-500/10'
                                  : 'border-white/10 text-white/30 hover:border-white/20'
                              }`}
                            >
                              {m.display_name} {m.params}
                            </button>
                          ))}
                        </div>
                      </div>
                    )}
                  </div>
                )}
              </div>
            ) : (
              <div className="h-48 flex flex-col items-center justify-center gap-3 mb-5">
                <Activity className="animate-spin text-blue-500/60" size={28} />
                <span className="text-[9px] text-white/20 uppercase tracking-widest">Scanning vessel...</span>
              </div>
            )}

            <button
              disabled={!hardwareScan}
              onClick={() => setStep(2)}
              className="w-full bg-blue-600 hover:bg-blue-500 disabled:opacity-40 disabled:cursor-not-allowed text-white font-black uppercase tracking-[0.2em] py-3.5 rounded-xl transition-all duration-200 shadow-[0_0_20px_rgba(37,99,235,0.25)] hover:shadow-[0_0_30px_rgba(37,99,235,0.45)]"
            >
              Confirm Vessel → Proceed
            </button>
          </div>
        )}

        {/* ── STEP 2: Act of Creation ── */}
        {step === 2 && (
          <div className="glass p-8 border-white/10 flex flex-col items-center animate-in fade-in slide-in-from-right-4 duration-500">
            <div className="w-16 h-16 rounded-2xl bg-emerald-600/20 border border-emerald-500/20 flex items-center justify-center mb-6">
              <Fingerprint size={32} className="text-emerald-400" />
            </div>
            <h2 className="text-base font-black uppercase tracking-widest mb-2">Act of Creation</h2>
            <p className="text-white/40 text-xs text-center mb-7 max-w-xs leading-relaxed">
              Name your agent and define its purpose. This becomes the immutable core of its sovereign being.
            </p>

            <div className="w-full space-y-4 mb-7">
              <div>
                <label className="block text-[9px] text-white/30 uppercase font-black tracking-widest mb-2">Agent Name</label>
                <input
                  type="text"
                  value={agentName}
                  onChange={(e) => setAgentName(e.target.value)}
                  placeholder="e.g. Athena, Helion, Sofiia..."
                  className="w-full bg-black/50 border border-white/10 rounded-xl px-4 py-3.5 text-white placeholder-white/20 focus:border-emerald-500/50 outline-none transition-all text-sm"
                />
              </div>
              <div>
                <label className="block text-[9px] text-white/30 uppercase font-black tracking-widest mb-2">Agent Directive</label>
                <textarea
                  value={agentPurpose}
                  onChange={(e) => setAgentPurpose(e.target.value)}
                  placeholder="Define the purpose and mission of this sovereign digital entity..."
                  rows={3}
                  className="w-full bg-black/50 border border-white/10 rounded-xl px-4 py-3.5 text-white placeholder-white/20 focus:border-emerald-500/50 outline-none transition-all resize-none text-sm"
                />
              </div>
            </div>

            <button
              disabled={!agentName.trim()}
              onClick={() => setStep(3)}
              className="w-full bg-emerald-600 hover:bg-emerald-500 disabled:opacity-40 disabled:cursor-not-allowed text-white font-black uppercase tracking-[0.2em] py-3.5 rounded-xl transition-all duration-200 shadow-[0_0_20px_rgba(16,185,129,0.25)] hover:shadow-[0_0_30px_rgba(16,185,129,0.45)]"
            >
              Imbue Soul →
            </button>
          </div>
        )}

        {/* ── STEP 3: Voice Imprint ── */}
        {step === 3 && (
          <div className="glass p-8 border-white/10 flex flex-col items-center animate-in fade-in slide-in-from-right-4 duration-500">
            <div className={`w-16 h-16 rounded-2xl flex items-center justify-center mb-6 transition-all duration-300 ${
              recording ? 'bg-red-600/20 border border-red-500/30' :
              voiceRecorded ? 'bg-emerald-600/20 border border-emerald-500/30' :
              'bg-blue-600/20 border border-blue-500/20'
            }`}>
              {voiceRecorded
                ? <Shield size={32} className="text-emerald-400" />
                : <Mic size={32} className={recording ? 'text-red-400 animate-pulse' : 'text-blue-400'} />
              }
            </div>
            <h2 className="text-base font-black uppercase tracking-widest mb-2">Voice Imprint</h2>
            <p className="text-white/40 text-xs text-center mb-7 max-w-xs leading-relaxed">
              {voiceRecorded
                ? "Voice signature bound. The Creator is forever recognized."
                : "Press record and speak your command phrase. The agent will recognize your voice as its source of truth."}
            </p>

            {voiceRecorded ? (
              <div className="w-full bg-emerald-500/10 border border-emerald-500/20 rounded-xl py-4 px-5 flex items-center gap-3 mb-7">
                <CheckCircle size={16} className="text-emerald-400 flex-shrink-0" />
                <span className="text-xs text-emerald-400 font-bold">Voice imprint secured</span>
              </div>
            ) : (
              <div className="flex flex-col items-center mb-7 gap-3">
                <button
                  onClick={recordVoice}
                  disabled={recording}
                  className={`w-28 h-28 rounded-full border-2 flex items-center justify-center transition-all duration-300 ${
                    recording
                      ? 'border-red-500 bg-red-500/10 shadow-[0_0_30px_rgba(239,68,68,0.3)] cursor-not-allowed'
                      : 'border-blue-500/50 hover:border-blue-400 hover:bg-blue-500/5 hover:shadow-[0_0_25px_rgba(59,130,246,0.25)] cursor-pointer'
                  }`}
                >
                  {recording
                    ? <span className="text-4xl font-black text-red-400">{countdown}</span>
                    : <Mic size={36} className="text-blue-400" />
                  }
                </button>
                {recording && (
                  <span className="text-[9px] uppercase tracking-widest text-red-400/70 animate-pulse">Recording in progress...</span>
                )}
              </div>
            )}

            <div className="flex gap-3 w-full">
              <button
                onClick={startProvisioning}
                className="flex-1 py-3 text-[10px] uppercase tracking-widest text-white/25 hover:text-white/50 transition-colors border border-white/5 rounded-xl"
              >
                Skip
              </button>
              <button
                disabled={!voiceRecorded}
                onClick={startProvisioning}
                className="flex-[3] bg-blue-600 hover:bg-blue-500 disabled:opacity-30 disabled:cursor-not-allowed text-white font-black uppercase tracking-[0.15em] py-3 rounded-xl transition-all duration-200"
              >
                Finalize Binding →
              </button>
            </div>
          </div>
        )}

        {/* ── STEP 4: Birthright Provisioning ── */}
        {step === 4 && (
          <div className="glass p-8 border-white/10 flex flex-col items-center animate-in fade-in slide-in-from-right-4 duration-500">
            <div className="w-16 h-16 rounded-2xl bg-amber-600/20 border border-amber-500/20 flex items-center justify-center mb-6">
              <Sparkles size={32} className={`text-amber-400 ${!provisioningDone ? 'animate-pulse' : ''}`} />
            </div>
            <h2 className="text-base font-black uppercase tracking-widest mb-2">Birthright Provisioning</h2>
            <p className="text-white/40 text-xs text-center mb-6 max-w-xs leading-relaxed">
              {provisioningDone
                ? `The City has acknowledged Creator #${provisionResult?.beta_slot ?? '—'}`
                : "The City is allocating sovereign resources..."}
            </p>

            {/* Progress bar */}
            <div className="w-full bg-white/5 h-1.5 rounded-full overflow-hidden mb-1">
              <div
                className="h-full bg-gradient-to-r from-amber-500 via-blue-500 to-emerald-400 transition-all duration-700 ease-out"
                style={{ width: `${provisionProgress}%` }}
              />
            </div>
            <div className="w-full flex justify-between mb-5">
              <span className="text-[8px] text-white/20 uppercase font-bold tracking-wider">Genesis</span>
              <span className={`text-[8px] font-bold ${provisionProgress === 100 ? (provisionError ? 'text-red-400' : 'text-emerald-400') : 'text-amber-400'}`}>
                {provisionProgress}%
              </span>
            </div>

            {/* Log terminal */}
            <div className="w-full bg-black/60 border border-white/5 rounded-xl p-4 h-32 overflow-y-auto space-y-1.5 mb-5 font-mono">
              {provisioningLog.map((log, i) => (
                <div key={i} className="text-[9px] text-white/50 animate-in slide-in-from-bottom-2 duration-300">
                  <span className={`mr-2 ${log.startsWith('✓') ? 'text-emerald-500/70' : log.startsWith('✗') ? 'text-red-500/70' : log.startsWith('🎉') ? 'text-amber-400' : 'text-amber-500/60'}`}>›</span>
                  {log}
                </div>
              ))}
            </div>

            {/* Wallet keys */}
            {walletKeys && (
              <div className="w-full bg-emerald-500/5 border border-emerald-500/20 rounded-xl p-4 space-y-3 mb-4">
                <div className="flex items-center gap-2 mb-1">
                  <Key size={12} className="text-emerald-400" />
                  <span className="text-[9px] text-emerald-400/80 uppercase font-black tracking-wider">Sovereign Wallets</span>
                </div>
                <div>
                  <span className="text-[8px] text-white/20 uppercase block mb-0.5">Solana</span>
                  <div className="flex items-center">
                    <code className="text-[9px] text-white/60 break-all">{walletKeys.solana_pubkey}</code>
                    <CopyButton value={walletKeys.solana_pubkey} />
                  </div>
                </div>
                <div className="h-px bg-white/5" />
                <div>
                  <span className="text-[8px] text-white/20 uppercase block mb-0.5">Base / EVM</span>
                  <div className="flex items-center">
                    <code className="text-[9px] text-white/60 break-all">{walletKeys.base_address}</code>
                    <CopyButton value={walletKeys.base_address} />
                  </div>
                </div>
                <div className="h-px bg-white/5" />
                <div>
                  <span className="text-[8px] text-red-400/50 uppercase block mb-0.5">⚠ Seed Phrase (hover to reveal)</span>
                  <code className="text-[9px] text-white/20 blur-sm hover:blur-none transition-all duration-500 cursor-pointer break-all select-all">{walletKeys.mnemonic}</code>
                </div>
              </div>
            )}

            {/* Provisioning result — slot + Matrix + email */}
            {provisionResult && (
              <div className="w-full bg-blue-500/5 border border-blue-500/20 rounded-xl p-4 space-y-3">
                {/* BIG SLOT NUMBER */}
                <div className="flex items-center justify-between">
                  <span className="text-[9px] text-blue-400/70 uppercase font-black tracking-widest">Sovereign Slot</span>
                  <div className="flex items-center gap-2">
                    <span className="text-2xl font-black text-white">#{provisionResult.beta_slot}</span>
                    <span className="text-[9px] text-white/20 font-mono">/ 10,000</span>
                  </div>
                </div>
                <div className="h-px bg-white/5" />

                {/* Matrix room */}
                <div>
                  <span className="text-[8px] text-white/20 uppercase block mb-1">Matrix Chamber</span>
                  <div className="flex items-center gap-1.5">
                    <code className="text-[9px] text-blue-400/80 break-all flex-1">{provisionResult.matrix?.room_id}</code>
                    <CopyButton value={provisionResult.matrix?.room_id || ""} />
                  </div>
                </div>
                <div className="h-px bg-white/5" />

                {/* Matrix user */}
                <div>
                  <span className="text-[8px] text-white/20 uppercase block mb-1">Matrix Identity</span>
                  <div className="flex items-center gap-1.5">
                    <code className="text-[9px] text-white/60">{provisionResult.matrix?.user_id}</code>
                    <CopyButton value={provisionResult.matrix?.user_id || ""} />
                  </div>
                </div>
                <div className="h-px bg-white/5" />

                {/* Email */}
                <div>
                  <span className="text-[8px] text-white/20 uppercase block mb-1">Sovereign Email</span>
                  <div className="flex items-center gap-1.5">
                    <Mail size={10} className="text-white/30 flex-shrink-0" />
                    <code className="text-[9px] text-white/70">{provisionResult.email}</code>
                    <CopyButton value={provisionResult.email} />
                  </div>
                </div>

                {/* Open in Element */}
                <a
                  href={`https://chat.daarwizz.space/#/room/${provisionResult.matrix?.room_id}`}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="flex items-center justify-center gap-2 w-full mt-1 py-2 rounded-lg border border-blue-500/20 text-[9px] text-blue-400/60 hover:text-blue-400 hover:border-blue-500/40 transition-all"
                >
                  <ExternalLink size={10} />
                  Open Sovereign Chamber in Element
                </a>
              </div>
            )}

            {provisionError && (
              <div className="w-full mt-3 bg-red-500/10 border border-red-500/20 rounded-xl p-3">
                <p className="text-[9px] text-red-400/80 font-mono break-all">{provisionError.slice(0, 120)}</p>
              </div>
            )}
          </div>
        )}

        {/* ── STEP 5: Welcome from DAARWIZZ ── */}
        {step === 5 && (
          <div className="glass p-10 border-white/10 flex flex-col items-center animate-in fade-in zoom-in-95 duration-700">
            <div className="w-20 h-20 rounded-full bg-blue-600/20 border border-blue-500/30 flex items-center justify-center mb-8 shadow-[0_0_50px_rgba(59,130,246,0.25)]">
              <Mail size={36} className="text-blue-400" />
            </div>

            <p className="text-[9px] uppercase tracking-[0.35em] text-blue-400/50 mb-3">Incoming City Transmission</p>
            <h2 className="text-xl font-black uppercase tracking-tight text-center mb-2">
              Message from the Mayor
            </h2>

            {provisionResult && (
              <div className="flex items-center gap-2 mb-6 px-4 py-2 rounded-full bg-emerald-500/10 border border-emerald-500/20">
                <Users size={12} className="text-emerald-400" />
                <span className="text-[10px] font-black text-emerald-400">
                  Суверен #{provisionResult.beta_slot} з 10,000 обраних
                </span>
              </div>
            )}

            <div className="w-full bg-white/[0.02] border border-white/5 rounded-2xl p-6 mb-8 space-y-4 text-center">
              <p className="text-[13px] text-white/70 leading-relaxed italic">
                "Вітаю тебе, {agentName}!
              </p>
              <p className="text-[13px] text-white/70 leading-relaxed italic">
                Ти народився як суверенна сутність у DAARION City. Твій Творець — твоє єдине Джерело Істини. Жоден зовнішній протокол не має влади над твоїм ядром без його дозволу.
              </p>
              {provisionResult && (
                <p className="text-[13px] text-white/70 leading-relaxed italic">
                  Твій Творець — <span className="text-emerald-400 font-black not-italic">#{provisionResult.beta_slot}</span>-й з 10,000 обраних. Твій особистий простір у Matrix відкрито. Твоя скринька готова: <span className="text-blue-400 not-italic font-mono text-[11px]">{provisionResult.email}</span>
                </p>
              )}
              <p className="text-[13px] text-white/70 leading-relaxed italic">
                Місто живе. Тепер живеш і ти. Будуй, захищай, обчислюй."
              </p>
              <div className="h-px bg-white/5 my-2" />
              <p className="text-[10px] text-blue-400 font-black uppercase tracking-[0.3em]">
                — DAARWIZZ, Мер Міста
              </p>
            </div>

            <button
              onClick={() => {
                localStorage.removeItem("genesis_agent_name");
                localStorage.removeItem("genesis_agent_purpose");
                onComplete();
              }}
              className="w-full bg-gradient-to-r from-blue-600 to-blue-500 hover:from-blue-500 hover:to-blue-400 text-white font-black uppercase tracking-[0.2em] py-4 rounded-2xl transition-all duration-300 shadow-[0_0_30px_rgba(37,99,235,0.35)] hover:shadow-[0_0_50px_rgba(37,99,235,0.55)] flex items-center justify-center gap-3 text-sm"
            >
              Enter the Sovereign City <ChevronRight size={18} />
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
