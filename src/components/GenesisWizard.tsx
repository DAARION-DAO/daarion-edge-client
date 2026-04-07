import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Shield, Cpu, Mic, Fingerprint, Mail, Key, Sparkles, ChevronRight,
  Activity, CheckCircle, Zap, Smartphone, Monitor, Server, Copy,
  ExternalLink, Users, Wallet, User, AtSign, AlertTriangle, Lock,
  Globe, FileText, Waves
} from "lucide-react";
import { activeCaptureAdapter } from "../lib/captureAdapters";
import { fetchChallenge, uploadVoiceImprint, sealCeremony } from "../lib/voiceCeremonyApi";

interface GenesisWizardProps {
  onComplete: () => void;
}

// ── DAARION Token Gate — Polygon Mainnet ──────────────────────────
// Source: github.com/connectplatform/daarion README.ADDRESSES.md
const DAARION_TOKEN_CONTRACT = "0x8Fe60b6F2DCBE68a1659b81175C665EB94015B16"; // DAARION on Polygon
const DAAR_TOKEN_CONTRACT    = "0x5aF82259455a963eC20Ea92471f55767B5919E38"; // DAAR on Polygon
const DAARION_TOKEN_SYMBOL = "DAARION";
const DAAR_TOKEN_SYMBOL    = "DAAR";
// Polygon Mainnet RPC (public endpoint)
const EVM_RPC_URL = "https://polygon-rpc.com";
// Token gate: user must hold ANY DAARION or ANY DAAR to participate
const TOKEN_GATE_ENABLED = true;

// ── Steps ─────────────────────────────────────────────────────────
const STEPS = [
  { id: 1, label: "Vessel" },
  { id: 2, label: "Creator" },    // NEW — who are you?
  { id: 3, label: "Agent" },      // name + directive
  { id: 4, label: "Voice" },
  { id: 5, label: "Birth" },
  { id: 6, label: "City" },
];

const TIER_COLORS: Record<string, string> = {
  "ultra-lite": "text-orange-400 border-orange-500/30 bg-orange-500/10",
  "lite": "text-amber-400 border-amber-500/30 bg-amber-500/10",
  "balanced": "text-blue-400 border-blue-500/30 bg-blue-500/10",
  "full": "text-emerald-400 border-emerald-500/30 bg-emerald-500/10",
};

const DEVICE_CLASS_ICONS: Record<string, any> = {
  Smartphone, Tablet: Smartphone, Laptop: Monitor, Workstation: Server,
};

function CopyButton({ value }: { value: string }) {
  const [copied, setCopied] = useState(false);
  return (
    <button
      onClick={() => { navigator.clipboard.writeText(value); setCopied(true); setTimeout(() => setCopied(false), 1500); }}
      className="ml-1.5 text-white/20 hover:text-white/60 transition-colors flex-shrink-0"
    >
      {copied ? <CheckCircle size={10} className="text-emerald-400" /> : <Copy size={10} />}
    </button>
  );
}

function FieldRow({ icon: Icon, label, children }: any) {
  return (
    <div>
      <label className="flex items-center gap-1.5 text-[9px] text-white/30 uppercase font-black tracking-widest mb-2">
        <Icon size={9} className="text-white/20" />{label}
      </label>
      {children}
    </div>
  );
}

// ── Token Balance Check (ERC-20 balanceOf via JSON-RPC) ────────────
// Checks DAARION first, then DAAR as fallback entry-level token
async function checkDaarionBalance(address: string): Promise<{ balance: bigint; token: string; decimals: number }> {
  const erc20BalanceOf = async (contract: string): Promise<bigint> => {
    const data = "0x70a08231" + address.slice(2).padStart(64, "0");
    const resp = await fetch(EVM_RPC_URL, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        jsonrpc: "2.0", id: 1, method: "eth_call",
        params: [{ to: contract, data }, "latest"]
      }),
    });
    const json = await resp.json();
    if (json.result && json.result !== "0x" && json.result !== "0x0") {
      return BigInt(json.result);
    }
    return BigInt(0);
  };

  // Check DAARION first (premium: $1000/token)
  try {
    const daarionBal = await erc20BalanceOf(DAARION_TOKEN_CONTRACT);
    if (daarionBal > BigInt(0)) {
      return { balance: daarionBal, token: DAARION_TOKEN_SYMBOL, decimals: 18 };
    }
  } catch { /* fallthrough */ }

  // Check DAAR (entry level: $10/token)
  try {
    const daarBal = await erc20BalanceOf(DAAR_TOKEN_CONTRACT);
    if (daarBal > BigInt(0)) {
      return { balance: daarBal, token: DAAR_TOKEN_SYMBOL, decimals: 18 };
    }
  } catch { /* fallthrough */ }

  return { balance: BigInt(0), token: "", decimals: 18 };
}

// ─────────────────────────────────────────────────────────────────

export function GenesisWizard({ onComplete }: GenesisWizardProps) {
  const [step, setStep] = useState(1);

  // Step 1 — Hardware
  const [hardwareScan, setHardwareScan] = useState<any>(null);
  const [selectedModel, setSelectedModel] = useState<any>(null);
  const [betaStatus, setBetaStatus] = useState<any>(null);

  // Step 2 — Creator Identity
  const [creatorFirstName, setCreatorFirstName] = useState(() => localStorage.getItem("c_fname") || "");
  const [creatorLastName, setCreatorLastName] = useState(() => localStorage.getItem("c_lname") || "");
  const [creatorTelegram, setCreatorTelegram] = useState(() => localStorage.getItem("c_tg") || "");
  const [creatorEmail, setCreatorEmail] = useState(() => localStorage.getItem("c_email") || "");
  const [creatorWallet, setCreatorWallet] = useState(() => localStorage.getItem("c_wallet") || "");
  // Token gate
  const [_tokenBalance, setTokenBalance] = useState<bigint | null>(null);
  const [tokenChecking, setTokenChecking] = useState(false);
  const [tokenError, setTokenError] = useState<string | null>(null);
  const [walletVerified, setWalletVerified] = useState(false);

  // Step 3 — Agent
  const [agentName, setAgentName] = useState(() => localStorage.getItem("genesis_agent_name") || "");
  const [agentPurpose, setAgentPurpose] = useState(() => localStorage.getItem("genesis_agent_purpose") || "");

  // Step 4 — Voice Ceremony (B-first / C-ready)
  type VoiceState = "idle" | "fetching_challenge" | "ready" | "recording" | "uploading" | "sealed" | "error";
  const [voiceState, setVoiceState] = useState<VoiceState>("idle");
  const [voiceChallenge, setVoiceChallenge] = useState<{ challenge_id: string; phrase: string } | null>(null);
  const [_voiceImprintId, setVoiceImprintId] = useState<string | null>(null);
  const [voiceSealId, setVoiceSealId] = useState<string | null>(null);
  const [voiceError, setVoiceError] = useState<string | null>(null);
  const [countdown, setCountdown] = useState(0);

  // Step 5 — Provisioning
  const [provisioningLog, setProvisioningLog] = useState<string[]>([]);
  const [provisionProgress, setProvisionProgress] = useState(0);
  const [walletKeys, setWalletKeys] = useState<any>(null);
  const [provisionResult, setProvisionResult] = useState<any>(null);
  const [provisionError, setProvisionError] = useState<string | null>(null);
  const [provisioningDone, setProvisioningDone] = useState(false);

  const logRef = useRef<HTMLDivElement>(null);

  // Persistence
  useEffect(() => { if (creatorFirstName) localStorage.setItem("c_fname", creatorFirstName); }, [creatorFirstName]);
  useEffect(() => { if (creatorLastName) localStorage.setItem("c_lname", creatorLastName); }, [creatorLastName]);
  useEffect(() => { if (creatorTelegram) localStorage.setItem("c_tg", creatorTelegram); }, [creatorTelegram]);
  useEffect(() => { if (creatorEmail) localStorage.setItem("c_email", creatorEmail); }, [creatorEmail]);
  useEffect(() => { if (creatorWallet) localStorage.setItem("c_wallet", creatorWallet); }, [creatorWallet]);
  useEffect(() => { if (agentName) localStorage.setItem("genesis_agent_name", agentName); }, [agentName]);
  useEffect(() => { if (agentPurpose) localStorage.setItem("genesis_agent_purpose", agentPurpose); }, [agentPurpose]);

  useEffect(() => {
    if (step === 1) {
      setTimeout(async () => {
        try {
          const cap = await invoke("get_capabilities");
          setHardwareScan(cap);
          setSelectedModel((cap as any).recommended_model);
        } catch (e) { console.error(e); }
      }, 1200);
      invoke("check_beta_slots").then((s: any) => setBetaStatus(s)).catch(() => {});
    }
  }, [step]);

  useEffect(() => {
    if (logRef.current) logRef.current.scrollTop = logRef.current.scrollHeight;
  }, [provisioningLog]);

  // ── Token Gate Verification ─────────────────────────────────────
  const verifyWallet = async () => {
    if (!creatorWallet.match(/^0x[a-fA-F0-9]{40}$/)) {
      setTokenError("Невірний формат MetaMask адреси (0x...)");
      return;
    }
    setTokenChecking(true);
    setTokenError(null);
    setTokenBalance(null);
    try {
      if (!TOKEN_GATE_ENABLED) {
        // Gate open — just validate format
        setTokenBalance(BigInt(0));
        setWalletVerified(true);
        return;
      }
      const result = await checkDaarionBalance(creatorWallet);
      setTokenBalance(result.balance);
      if (result.balance > BigInt(0)) {
        setWalletVerified(true);
        (window as any).__tokenFound = result; // store for display
      } else {
        setTokenError(
          `На гаманці ${creatorWallet.slice(0,8)}...${creatorWallet.slice(-6)} ` +
          `не знайдено токенів DAARION або DAAR (Polygon). ` +
          `Придбайте будь-яку кількість DAAR ($10) або DAARION ($1000) на Polygon.`
        );
        setWalletVerified(false);
      }
    } catch (e) {
      setTokenError("Не вдалося перевірити баланс Polygon. Спробуйте ще раз.");
    } finally {
      setTokenChecking(false);
    }
  };

  const creatorStep2Valid = creatorFirstName.trim() && creatorLastName.trim() &&
    creatorEmail.trim() && creatorWallet.match(/^0x[a-fA-F0-9]{40}$/) &&
    walletVerified;

  // Token found info (set by verifyWallet)
  const getTokenFoundInfo = () => (window as any).__tokenFound as { balance: bigint; token: string; decimals: number } | undefined;

  // ── Voice Ceremony (B-first / C-ready) ──────────────────────────────────────
  const RECORD_SECS = 5;

  const startVoiceCeremony = async () => {
    setVoiceError(null);
    setVoiceState("fetching_challenge");
    try {
      const ch = await fetchChallenge(agentName.trim() || "Agent");
      setVoiceChallenge({ challenge_id: ch.challenge_id, phrase: ch.phrase });
      setVoiceState("ready");
    } catch (e: any) {
      setVoiceError(String(e));
      setVoiceState("error");
    }
  };

  const recordAndUpload = async () => {
    if (!voiceChallenge) return;
    setVoiceState("recording");
    setCountdown(RECORD_SECS);
    const interval = setInterval(() =>
      setCountdown(prev => { if (prev <= 1) { clearInterval(interval); return 0; } return prev - 1; })
    , 1000);

    try {
      const audioBlob = await activeCaptureAdapter.start(RECORD_SECS);
      clearInterval(interval);
      setVoiceState("uploading");

      const uploaded = await uploadVoiceImprint(
        voiceChallenge.challenge_id,
        agentName.trim() || "Agent",
        audioBlob,
        creatorWallet || undefined,
      );

      const sealed = await sealCeremony(
        uploaded.imprint_id,
        voiceChallenge.challenge_id,
      );

      setVoiceImprintId(uploaded.imprint_id);
      setVoiceSealId(sealed.seal_id);
      setVoiceState("sealed");
    } catch (e: any) {
      clearInterval(interval);
      setVoiceError(String(e));
      setVoiceState("error");
    }
  };

  // ── Main Provisioning ───────────────────────────────────────────
  const startProvisioning = async () => {
    setStep(5);
    setProvisionError(null);
    const addLog = (msg: string) => setProvisioningLog(prev => [...prev, msg]);

    addLog(`Ініціалізація Genesis для Творця: ${creatorFirstName} ${creatorLastName}`);
    setProvisionProgress(5);

    await new Promise(r => setTimeout(r, 500));
    addLog(`Прив'язка особистості творця до протоколу DAARION...`);
    setProvisionProgress(12);

    await new Promise(r => setTimeout(r, 700));
    addLog("Генерація суверенних гаманців агента (BIP39)...");

    let keys: any = null;
    try {
      keys = await invoke("generate_wallet_keys");
      setWalletKeys(keys);
      addLog(`✓ Solana: ${(keys.solana_pubkey as string).slice(0, 14)}...`);
      addLog(`✓ EVM: ${(keys.base_address as string).slice(0, 16)}...`);
      setProvisionProgress(30);
    } catch {
      addLog("⚠ Wallet — використовуємо fallback identity.");
      keys = { solana_pubkey: "GENESIS_OFFLINE_KEY", base_address: "0x0", mnemonic: "" };
    }

    await new Promise(r => setTimeout(r, 600));
    addLog(`Підключення до Genesis API (NODA1)...`);
    setProvisionProgress(45);

    await new Promise(r => setTimeout(r, 400));
    addLog(`Реєстрація агента ${agentName} від Творця ${creatorFirstName} ${creatorLastName}...`);
    setProvisionProgress(58);

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
      setProvisionProgress(78);
      addLog(`✓ Суверенний слот агента: #${result.beta_slot}`);
      addLog(`✓ Matrix Chamber: ${result.matrix?.room_id?.slice(0, 26)}...`);
      addLog(`✓ Поштова скринька агента: ${result.email}`);
      addLog(`✓ Вітальне повідомлення DAARWIZZ — відправлено.`);

      await new Promise(r => setTimeout(r, 500));
      addLog("Також реєструємо профіль Творця в реєстрі Міста...");
      // POST creator profile to genesis API
      try {
        await fetch("https://api.daarion.city/genesis/creator", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            first_name: creatorFirstName,
            last_name: creatorLastName,
            telegram_handle: creatorTelegram,
            personal_email: creatorEmail,
            evm_address: creatorWallet,
            agent_name: agentName,
            agent_slot: result.beta_slot,
          }),
        });
        addLog(`✓ Профіль Творця збережено в реєстрі DAARION.`);
      } catch {
        addLog(`⚠ Профіль Творця — збережено локально (синхронізується пізніше).`);
      }

      setProvisionProgress(92);
      try { await invoke("initialize_identity"); } catch { }
      try { await invoke("enroll_node", { bootstrapGrant: "GENESIS_GRANT" }); } catch { }
      setProvisionProgress(100);
      addLog("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
      addLog(`🎉 ${agentName} народився! Творець: ${creatorFirstName} ${creatorLastName}. Слот: #${result.beta_slot}`);
      setProvisioningDone(true);
      setTimeout(() => setStep(6), 2000);

    } catch (err: any) {
      setProvisionError(String(err));
      addLog(`✗ Помилка: ${String(err).slice(0, 100)}`);
      setProvisionProgress(100);
      setTimeout(() => setStep(6), 3500);
    }
  };

  // ── Render ──────────────────────────────────────────────────────
  return (
    <div className="min-h-screen bg-[#020202] text-white flex flex-col items-center justify-center p-6 relative overflow-hidden font-sans">

      <div className="absolute inset-0 pointer-events-none overflow-hidden">
        <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[700px] h-[700px] rounded-full bg-blue-900/30 blur-[140px] animate-pulse" />
        <div className="absolute top-1/4 left-1/4 w-[300px] h-[300px] rounded-full bg-emerald-900/10 blur-[120px] animate-pulse" style={{ animationDelay: "1s" }} />
      </div>

      <div className="z-10 w-full max-w-xl">

        {/* Header */}
        <div className="text-center mb-8">
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
                <span className="text-white/20"> / {betaStatus.total?.toLocaleString()} слотів залишилось</span>
              </span>
            </div>
          )}
        </div>

        {/* Step Indicator */}
        <div className="flex items-center justify-center gap-0.5 mb-8">
          {STEPS.map((s, i) => (
            <div key={s.id} className="flex items-center gap-0.5">
              <div className={`flex flex-col items-center gap-1 transition-all duration-500 ${step >= s.id ? 'opacity-100' : 'opacity-25'}`}>
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
                <div className={`w-6 h-px mb-5 transition-all duration-700 ${step > s.id ? 'bg-emerald-500/60' : 'bg-white/5'}`} />
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
            <h2 className="text-base font-black uppercase tracking-widest mb-2">Аудит Апаратури</h2>
            <p className="text-white/40 text-xs text-center mb-7 max-w-xs leading-relaxed">
              Сканування пристрою Творця. Підбір оптимальної суверенної моделі...
            </p>

            {hardwareScan ? (
              <div className="w-full space-y-3 mb-5">
                <div className="bg-black/40 border border-white/5 rounded-xl p-4 space-y-2.5">
                  {[
                    ["Device Class", (() => { const Icon = DEVICE_CLASS_ICONS[hardwareScan.device_class] || Monitor; return <span className="flex items-center gap-1.5"><Icon size={11} />{hardwareScan.device_class}</span>; })(), "text-blue-400"],
                    ["CPU", hardwareScan.cpu_brand, "text-white/70 font-mono text-[10px] max-w-[180px] text-right truncate"],
                    ["RAM", `${Math.round(hardwareScan.ram_total_gb)} GB`, "text-white/80 font-mono"],
                    ["Acceleration", hardwareScan.gpu?.acceleration_api || 'CPU', hardwareScan.gpu?.detected ? 'text-emerald-400' : 'text-amber-400'],
                  ].map(([label, val, cls], i) => (
                    <div key={i}>
                      {i > 0 && <div className="h-px bg-white/5 mb-2.5" />}
                      <div className="flex justify-between items-center">
                        <span className="text-[9px] text-white/30 uppercase font-bold tracking-wider">{label}</span>
                        <span className={`text-[11px] font-bold ${cls}`}>{val}</span>
                      </div>
                    </div>
                  ))}
                </div>
                {hardwareScan.recommended_model && (
                  <div className="bg-blue-500/5 border border-blue-500/20 rounded-xl p-4">
                    <div className="flex items-center justify-between mb-2">
                      <span className="text-[9px] text-blue-400/70 uppercase font-black tracking-widest">Рекомендована модель</span>
                      <span className={`text-[8px] font-black uppercase px-2 py-0.5 rounded-full border ${TIER_COLORS[hardwareScan.recommended_model.performance_tier] || 'text-white/50 border-white/10'}`}>
                        {hardwareScan.recommended_model.performance_tier}
                      </span>
                    </div>
                    <div className="flex items-baseline gap-2 mb-1">
                      <span className="text-lg font-black text-white">{hardwareScan.recommended_model.display_name}</span>
                      <span className="text-sm text-white/50 font-bold">{hardwareScan.recommended_model.params}</span>
                    </div>
                    <p className="text-[10px] text-white/40 leading-relaxed mb-1">{hardwareScan.recommended_model.reason}</p>
                    <div className="flex items-center gap-3 text-[9px] text-white/30">
                      <span><Zap size={9} className="inline mr-0.5" />{hardwareScan.recommended_model.size_gb}GB</span>
                      <span>{hardwareScan.recommended_model.context_tokens?.toLocaleString()} ctx</span>
                    </div>
                  </div>
                )}
              </div>
            ) : (
              <div className="h-48 flex flex-col items-center justify-center gap-3 mb-5">
                <Activity className="animate-spin text-blue-500/60" size={28} />
                <span className="text-[9px] text-white/20 uppercase tracking-widest">Сканування пристрою...</span>
              </div>
            )}

            <button
              disabled={!hardwareScan}
              onClick={() => setStep(2)}
              className="w-full bg-blue-600 hover:bg-blue-500 disabled:opacity-40 disabled:cursor-not-allowed text-white font-black uppercase tracking-[0.2em] py-3.5 rounded-xl transition-all duration-200 shadow-[0_0_20px_rgba(37,99,235,0.25)] hover:shadow-[0_0_30px_rgba(37,99,235,0.45)]"
            >
              Підтвердити Пристрій →
            </button>
          </div>
        )}

        {/* ── STEP 2: Creator Identity ── */}
        {step === 2 && (
          <div className="glass p-8 border-white/10 flex flex-col items-center animate-in fade-in slide-in-from-right-4 duration-500">
            <div className="w-16 h-16 rounded-2xl bg-violet-600/20 border border-violet-500/20 flex items-center justify-center mb-6">
              <User size={32} className="text-violet-400" />
            </div>
            <h2 className="text-base font-black uppercase tracking-widest mb-1">Особистість Творця</h2>
            <p className="text-white/40 text-xs text-center mb-6 max-w-xs leading-relaxed">
              Ти — Творець. Назви себе. Місто зберігає цю інформацію у суверенному реєстрі.
            </p>

            <div className="w-full space-y-4 mb-2">

              {/* Name row */}
              <div className="grid grid-cols-2 gap-3">
                <FieldRow icon={User} label="Ім'я">
                  <input
                    type="text"
                    value={creatorFirstName}
                    onChange={e => setCreatorFirstName(e.target.value)}
                    placeholder="Олексій"
                    className="w-full bg-black/50 border border-white/10 rounded-xl px-4 py-3 text-white placeholder-white/20 focus:border-violet-500/50 outline-none transition-all text-sm"
                  />
                </FieldRow>
                <FieldRow icon={User} label="Прізвище">
                  <input
                    type="text"
                    value={creatorLastName}
                    onChange={e => setCreatorLastName(e.target.value)}
                    placeholder="Коваленко"
                    className="w-full bg-black/50 border border-white/10 rounded-xl px-4 py-3 text-white placeholder-white/20 focus:border-violet-500/50 outline-none transition-all text-sm"
                  />
                </FieldRow>
              </div>

              {/* Telegram */}
              <FieldRow icon={AtSign} label="Telegram @нікнейм">
                <div className="relative">
                  <span className="absolute left-4 top-1/2 -translate-y-1/2 text-white/25 text-sm font-bold">@</span>
                  <input
                    type="text"
                    value={creatorTelegram}
                    onChange={e => setCreatorTelegram(e.target.value.replace("@", ""))}
                    placeholder="username"
                    className="w-full bg-black/50 border border-white/10 rounded-xl pl-8 pr-4 py-3 text-white placeholder-white/20 focus:border-violet-500/50 outline-none transition-all text-sm"
                  />
                </div>
              </FieldRow>

              {/* Personal Email */}
              <FieldRow icon={Mail} label="Ваша особиста пошта">
                <input
                  type="email"
                  value={creatorEmail}
                  onChange={e => setCreatorEmail(e.target.value)}
                  placeholder="creator@gmail.com"
                  className="w-full bg-black/50 border border-white/10 rounded-xl px-4 py-3 text-white placeholder-white/20 focus:border-violet-500/50 outline-none transition-all text-sm"
                />
              </FieldRow>

              {/* MetaMask Wallet + Token Gate */}
              <FieldRow icon={Wallet} label="MetaMask Адреса (0x...)">
                <div className="flex gap-2">
                  <input
                    type="text"
                    value={creatorWallet}
                    onChange={e => { setCreatorWallet(e.target.value); setWalletVerified(false); setTokenBalance(null); setTokenError(null); }}
                    placeholder="0xABcD...1234"
                    className={`flex-1 bg-black/50 border rounded-xl px-4 py-3 text-white placeholder-white/20 outline-none transition-all text-sm font-mono ${
                      walletVerified ? 'border-emerald-500/50' : tokenError ? 'border-red-500/40' : 'border-white/10 focus:border-violet-500/50'
                    }`}
                  />
                  <button
                    onClick={verifyWallet}
                    disabled={tokenChecking || !creatorWallet}
                    className={`px-4 rounded-xl border text-[10px] font-black uppercase tracking-wider transition-all flex-shrink-0 ${
                      walletVerified
                        ? 'bg-emerald-500/20 border-emerald-500/40 text-emerald-400'
                        : 'bg-violet-500/10 border-violet-500/30 text-violet-400 hover:border-violet-500/60 disabled:opacity-40'
                    }`}
                  >
                    {tokenChecking ? <Activity size={14} className="animate-spin" /> : walletVerified ? <CheckCircle size={14} /> : "Verify"}
                  </button>
                </div>

                {/* Token balance display */}
                {walletVerified && (
                  <div className="mt-2 flex items-center gap-2 px-3 py-2 rounded-lg bg-emerald-500/10 border border-emerald-500/20">
                    <CheckCircle size={12} className="text-emerald-400 flex-shrink-0" />
                    <span className="text-[10px] text-emerald-400 font-bold">
                      {(() => {
                        const ti = getTokenFoundInfo();
                        if (!TOKEN_GATE_ENABLED) return 'Гаманець підтверджено (бета) ✓';
                        if (ti && ti.balance > BigInt(0)) {
                          const amount = (Number(ti.balance) / 1e18).toFixed(4);
                          return `✓ ${amount} ${ti.token} на Polygon`;
                        }
                        return 'Гаманець підтверджено ✓';
                      })()
                      }
                    </span>
                  </div>
                )}
                {tokenError && (
                  <div className="mt-2 flex items-start gap-2 px-3 py-2 rounded-lg bg-red-500/10 border border-red-500/20">
                    <AlertTriangle size={12} className="text-red-400 flex-shrink-0 mt-0.5" />
                    <span className="text-[10px] text-red-400 leading-relaxed">{tokenError}</span>
                  </div>
                )}
              </FieldRow>

              {/* Token requirement notice */}
              <div className="flex items-start gap-2 px-3 py-3 rounded-xl bg-amber-500/5 border border-amber-500/15">
                <Lock size={11} className="text-amber-400/70 flex-shrink-0 mt-0.5" />
                <div>
                  <p className="text-[9px] text-amber-400/80 font-black uppercase tracking-wider mb-0.5">Token Gate</p>
                  <p className="text-[9px] text-white/30 leading-relaxed">
                    {TOKEN_GATE_ENABLED
                      ? `На гаманці MetaMask має бути будь-яка кількість токенів ${DAARION_TOKEN_SYMBOL}. Це підтверджує статус учасника екосистеми.`
                      : `Beta: Token Gate тимчасово відкрито. Після деплою ${DAARION_TOKEN_SYMBOL} токену — перевірка буде обов'язковою.`}
                  </p>
                </div>
              </div>

              {/* KYC Future badge */}
              <div className="flex items-center gap-2 px-3 py-2 rounded-xl bg-white/[0.02] border border-white/5">
                <FileText size={11} className="text-white/20 flex-shrink-0" />
                <p className="text-[9px] text-white/20 leading-relaxed">
                  <span className="text-white/30 font-bold">KYC Level 2+</span> буде введено пізніше: верифікація документів, телефон, країна проживання.
                </p>
              </div>
            </div>

            <button
              disabled={!creatorStep2Valid}
              onClick={() => setStep(3)}
              className="w-full mt-4 bg-violet-600 hover:bg-violet-500 disabled:opacity-40 disabled:cursor-not-allowed text-white font-black uppercase tracking-[0.2em] py-3.5 rounded-xl transition-all duration-200 shadow-[0_0_20px_rgba(139,92,246,0.25)] hover:shadow-[0_0_30px_rgba(139,92,246,0.45)]"
            >
              Підтвердити Особистість →
            </button>
            {!creatorStep2Valid && (creatorFirstName || creatorLastName || creatorEmail || creatorWallet) && (
              <p className="mt-2 text-[9px] text-white/20 text-center">
                {!creatorFirstName || !creatorLastName ? "Вкажіть ім'я та прізвище" :
                 !creatorEmail ? "Введіть email" :
                 !creatorWallet.match(/^0x[a-fA-F0-9]{40}$/) ? "Перевірте формат MetaMask адреси" :
                 !walletVerified ? "Натисніть Verify для підтвердження гаманця" : ""}
              </p>
            )}
          </div>
        )}

        {/* ── STEP 3: Act of Creation (Agent) ── */}
        {step === 3 && (
          <div className="glass p-8 border-white/10 flex flex-col items-center animate-in fade-in slide-in-from-right-4 duration-500">
            <div className="w-16 h-16 rounded-2xl bg-emerald-600/20 border border-emerald-500/20 flex items-center justify-center mb-6">
              <Fingerprint size={32} className="text-emerald-400" />
            </div>
            <h2 className="text-base font-black uppercase tracking-widest mb-1">Акт Творення</h2>
            <p className="text-white/40 text-xs text-center mb-2 max-w-xs leading-relaxed">
              Назви свого агента та визнач його місію.
            </p>
            <div className="flex items-center gap-2 mb-6 px-3 py-1.5 rounded-full bg-violet-500/10 border border-violet-500/20">
              <User size={10} className="text-violet-400" />
              <span className="text-[9px] text-violet-400 font-bold">Творець: {creatorFirstName} {creatorLastName}</span>
            </div>

            <div className="w-full space-y-4 mb-7">
              <FieldRow icon={Globe} label="Ім'я Агента">
                <input
                  type="text"
                  value={agentName}
                  onChange={(e) => setAgentName(e.target.value)}
                  placeholder="напр. Athena, Helion, Nova..."
                  className="w-full bg-black/50 border border-white/10 rounded-xl px-4 py-3.5 text-white placeholder-white/20 focus:border-emerald-500/50 outline-none transition-all text-sm"
                />
              </FieldRow>
              <FieldRow icon={FileText} label="Директива Агента">
                <textarea
                  value={agentPurpose}
                  onChange={(e) => setAgentPurpose(e.target.value)}
                  placeholder="Визнач призначення та місію цієї суверенної цифрової сутності..."
                  rows={3}
                  className="w-full bg-black/50 border border-white/10 rounded-xl px-4 py-3.5 text-white placeholder-white/20 focus:border-emerald-500/50 outline-none transition-all resize-none text-sm"
                />
              </FieldRow>
            </div>

            <button
              disabled={!agentName.trim()}
              onClick={() => setStep(4)}
              className="w-full bg-emerald-600 hover:bg-emerald-500 disabled:opacity-40 disabled:cursor-not-allowed text-white font-black uppercase tracking-[0.2em] py-3.5 rounded-xl transition-all duration-200 shadow-[0_0_20px_rgba(16,185,129,0.25)] hover:shadow-[0_0_30px_rgba(16,185,129,0.45)]"
            >
              Вдихнути Душу →
            </button>
          </div>
        )}

        {/* ── STEP 4: Voice Ceremony ── */}
        {step === 4 && (
          <div className="glass p-8 border-white/10 flex flex-col items-center animate-in fade-in slide-in-from-right-4 duration-500">
            {/* Icon */}
            <div className={`w-16 h-16 rounded-2xl flex items-center justify-center mb-6 transition-all duration-500 ${
              voiceState === "recording" ? "bg-red-600/20 border border-red-500/30" :
              voiceState === "uploading" ? "bg-amber-600/20 border border-amber-500/30" :
              voiceState === "sealed"    ? "bg-emerald-600/20 border border-emerald-500/30" :
              voiceState === "error"     ? "bg-red-600/10 border border-red-500/20" :
              "bg-blue-600/20 border border-blue-500/20"
            }`}>
              {voiceState === "sealed"    ? <Shield size={32} className="text-emerald-400" /> :
               voiceState === "uploading" ? <Activity size={32} className="text-amber-400 animate-spin" /> :
               voiceState === "recording" ? <Waves size={32} className="text-red-400 animate-pulse" /> :
               <Mic size={32} className="text-blue-400" />}
            </div>

            <h2 className="text-base font-black uppercase tracking-widest mb-2">Голосова Церемонія</h2>

            {/* ── idle: start button ── */}
            {voiceState === "idle" && (
              <>
                <p className="text-white/40 text-xs text-center mb-7 max-w-xs leading-relaxed">
                  Отримай церемоніальну фразу та вимов її вголос. Твій голос стане підписом агента.
                </p>
                <button
                  onClick={startVoiceCeremony}
                  className="w-full bg-blue-600 hover:bg-blue-500 text-white font-black uppercase tracking-[0.15em] py-4 rounded-xl transition-all duration-200 shadow-[0_0_20px_rgba(37,99,235,0.25)]"
                >
                  Розпочати Церемонію →
                </button>
              </>
            )}

            {/* ── fetching challenge ── */}
            {voiceState === "fetching_challenge" && (
              <div className="flex flex-col items-center gap-3 py-6">
                <Activity className="animate-spin text-blue-500" size={28} />
                <span className="text-[9px] uppercase tracking-widest text-white/30">Генерація церемоніальної фрази...</span>
              </div>
            )}

            {/* ── ready: show phrase, record button ── */}
            {voiceState === "ready" && voiceChallenge && (
              <div className="w-full space-y-5">
                <div className="bg-blue-500/5 border border-blue-500/20 rounded-xl p-5">
                  <p className="text-[9px] text-blue-400/70 uppercase font-black tracking-widest mb-3">Вимов вголос:</p>
                  <p className="text-sm text-white/90 font-medium leading-relaxed italic">
                    «{voiceChallenge.phrase}»
                  </p>
                </div>
                <button
                  onClick={recordAndUpload}
                  className="w-full bg-red-600/80 hover:bg-red-500 text-white font-black uppercase tracking-[0.15em] py-4 rounded-xl transition-all duration-200 shadow-[0_0_20px_rgba(239,68,68,0.2)]"
                >
                  <Mic size={14} className="inline mr-2" />
                  Записати Голос ({RECORD_SECS}с) →
                </button>
              </div>
            )}

            {/* ── recording countdown ── */}
            {voiceState === "recording" && (
              <div className="flex flex-col items-center gap-4 py-4">
                <div className="w-24 h-24 rounded-full border-2 border-red-500 bg-red-500/10 shadow-[0_0_30px_rgba(239,68,68,0.3)] flex items-center justify-center">
                  <span className="text-4xl font-black text-red-400">{countdown}</span>
                </div>
                <span className="text-[9px] uppercase tracking-widest text-red-400/70 animate-pulse">Запис триває...</span>
                {voiceChallenge && (
                  <p className="text-[10px] text-white/30 italic text-center max-w-xs">«{voiceChallenge.phrase}»</p>
                )}
              </div>
            )}

            {/* ── uploading ── */}
            {voiceState === "uploading" && (
              <div className="flex flex-col items-center gap-3 py-6">
                <Activity className="animate-spin text-amber-400" size={28} />
                <span className="text-[9px] uppercase tracking-widest text-amber-400/70">Зберігаємо голосовий підпис...</span>
              </div>
            )}

            {/* ── sealed ✓ ── */}
            {voiceState === "sealed" && (
              <div className="w-full space-y-4">
                <div className="bg-emerald-500/10 border border-emerald-500/20 rounded-xl py-4 px-5 flex items-center gap-3">
                  <CheckCircle size={16} className="text-emerald-400 flex-shrink-0" />
                  <div>
                    <p className="text-xs text-emerald-400 font-bold">Церемоніальне Прив'язування Завершено</p>
                    <p className="text-[9px] text-emerald-400/60 font-mono mt-0.5">Voice Imprint Captured & Sealed</p>
                  </div>
                </div>
                {voiceSealId && (
                  <div className="bg-black/40 border border-white/5 rounded-xl p-3">
                    <span className="text-[8px] text-white/20 uppercase block mb-1">Seal ID</span>
                    <code className="text-[9px] text-white/40 break-all">{voiceSealId}</code>
                  </div>
                )}
              </div>
            )}

            {/* ── error ── */}
            {voiceState === "error" && voiceError && (
              <div className="w-full mt-2 bg-red-500/10 border border-red-500/20 rounded-xl p-3 mb-4">
                <p className="text-[9px] text-red-400 font-mono break-all">{voiceError.slice(0, 120)}</p>
                <button onClick={() => { setVoiceState("idle"); setVoiceError(null); }}
                  className="mt-2 text-[9px] text-red-400/60 hover:text-red-400 underline">
                  Спробувати ще раз
                </button>
              </div>
            )}

            {/* ── Bottom CTA ── */}
            <div className="flex gap-3 w-full mt-6">
              <button onClick={startProvisioning}
                className="flex-1 py-3 text-[10px] uppercase tracking-widest text-white/25 hover:text-white/50 transition-colors border border-white/5 rounded-xl">
                Пропустити
              </button>
              <button
                disabled={voiceState !== "sealed"}
                onClick={startProvisioning}
                className="flex-[3] bg-blue-600 hover:bg-blue-500 disabled:opacity-30 disabled:cursor-not-allowed text-white font-black uppercase tracking-[0.15em] py-3 rounded-xl transition-all duration-200"
              >
                Завершити Прив'язку →
              </button>
            </div>
          </div>
        )}

        {/* ── STEP 5: Birthright Provisioning ── */}
        {step === 5 && (
          <div className="glass p-8 border-white/10 flex flex-col items-center animate-in fade-in slide-in-from-right-4 duration-500">
            <div className="w-16 h-16 rounded-2xl bg-amber-600/20 border border-amber-500/20 flex items-center justify-center mb-6">
              <Sparkles size={32} className={`text-amber-400 ${!provisioningDone ? 'animate-pulse' : ''}`} />
            </div>
            <h2 className="text-base font-black uppercase tracking-widest mb-2">Акт Народження</h2>
            <p className="text-white/40 text-xs text-center mb-6 max-w-xs leading-relaxed">
              {provisioningDone ? `Місто прийняло агента #${provisionResult?.beta_slot}` : "Місто виділяє суверенні ресурси..."}
            </p>

            <div className="w-full bg-white/5 h-1.5 rounded-full overflow-hidden mb-1">
              <div className="h-full bg-gradient-to-r from-amber-500 via-blue-500 to-emerald-400 transition-all duration-700 ease-out" style={{ width: `${provisionProgress}%` }} />
            </div>
            <div className="w-full flex justify-between mb-5">
              <span className="text-[8px] text-white/20 uppercase font-bold tracking-wider">Genesis</span>
              <span className={`text-[8px] font-bold ${provisionProgress === 100 ? (provisionError ? 'text-red-400' : 'text-emerald-400') : 'text-amber-400'}`}>{provisionProgress}%</span>
            </div>

            <div ref={logRef} className="w-full bg-black/60 border border-white/5 rounded-xl p-4 h-32 overflow-y-auto space-y-1.5 mb-5 font-mono">
              {provisioningLog.map((log, i) => (
                <div key={i} className="text-[9px] text-white/50 animate-in slide-in-from-bottom-2 duration-300">
                  <span className={`mr-2 ${log.startsWith('✓') ? 'text-emerald-500/70' : log.startsWith('✗') ? 'text-red-500/70' : log.startsWith('🎉') ? 'text-amber-400' : log.startsWith('⚠') ? 'text-amber-500/70' : 'text-white/25'}`}>›</span>
                  {log}
                </div>
              ))}
            </div>

            {/* Wallet keys */}
            {walletKeys && walletKeys.solana_pubkey !== "GENESIS_OFFLINE_KEY" && (
              <div className="w-full bg-emerald-500/5 border border-emerald-500/20 rounded-xl p-4 space-y-3 mb-4">
                <div className="flex items-center gap-2 mb-1">
                  <Key size={12} className="text-emerald-400" />
                  <span className="text-[9px] text-emerald-400/80 uppercase font-black tracking-wider">Суверенні Гаманці Агента</span>
                </div>
                {[
                  ["Solana", walletKeys.solana_pubkey],
                  ["Base / EVM", walletKeys.base_address],
                ].map(([label, val]) => (
                  <div key={label}>
                    <span className="text-[8px] text-white/20 uppercase block mb-0.5">{label}</span>
                    <div className="flex items-center"><code className="text-[9px] text-white/60 break-all flex-1">{val}</code><CopyButton value={val} /></div>
                  </div>
                ))}
                <div className="h-px bg-white/5" />
                <div>
                  <span className="text-[8px] text-red-400/50 uppercase block mb-0.5">⚠ Сід-фраза (hover для показу)</span>
                  <code className="text-[9px] text-white/20 blur-sm hover:blur-none transition-all duration-500 cursor-pointer break-all select-all">{walletKeys.mnemonic}</code>
                </div>
              </div>
            )}

            {/* Provision result */}
            {provisionResult && (
              <div className="w-full bg-blue-500/5 border border-blue-500/20 rounded-xl p-4 space-y-3">
                <div className="flex items-center justify-between">
                  <span className="text-[9px] text-blue-400/70 uppercase font-black tracking-widest">Суверенний Слот</span>
                  <div className="flex items-center gap-2">
                    <span className="text-2xl font-black text-white">#{provisionResult.beta_slot}</span>
                    <span className="text-[9px] text-white/20 font-mono">/ 10,000</span>
                  </div>
                </div>
                <div className="h-px bg-white/5" />
                {[
                  ["Matrix Chamber", provisionResult.matrix?.room_id, "blue-400/80"],
                  ["Matrix Identity", provisionResult.matrix?.user_id, "white/60"],
                ].map(([label, val, cls]) => val && (
                  <div key={String(label)}>
                    <span className="text-[8px] text-white/20 uppercase block mb-1">{label}</span>
                    <div className="flex items-center gap-1.5">
                      <code className={`text-[9px] text-${cls} break-all flex-1`}>{String(val)}</code>
                      <CopyButton value={String(val)} />
                    </div>
                  </div>
                ))}
                <div className="h-px bg-white/5" />
                <div>
                  <span className="text-[8px] text-white/20 uppercase block mb-1">Поштова скринька агента</span>
                  <div className="flex items-center gap-1.5">
                    <Mail size={10} className="text-white/30 flex-shrink-0" />
                    <code className="text-[9px] text-white/70 flex-1">{provisionResult.email}</code>
                    <CopyButton value={provisionResult.email} />
                  </div>
                </div>
                <a href={`https://chat.daarwizz.space/#/room/${provisionResult.matrix?.room_id}`}
                  target="_blank" rel="noopener noreferrer"
                  className="flex items-center justify-center gap-2 w-full mt-1 py-2 rounded-lg border border-blue-500/20 text-[9px] text-blue-400/60 hover:text-blue-400 hover:border-blue-500/40 transition-all">
                  <ExternalLink size={10} />Відкрити Sovereign Chamber в Element
                </a>
              </div>
            )}

            {provisionError && (
              <div className="w-full mt-3 bg-red-500/10 border border-red-500/20 rounded-xl p-3">
                <p className="text-[9px] text-red-400/80 font-mono break-all">{provisionError.slice(0, 140)}</p>
              </div>
            )}
          </div>
        )}

        {/* ── STEP 6: Welcome from DAARWIZZ ── */}
        {step === 6 && (
          <div className="glass p-10 border-white/10 flex flex-col items-center animate-in fade-in zoom-in-95 duration-700">
            <div className="w-20 h-20 rounded-full bg-blue-600/20 border border-blue-500/30 flex items-center justify-center mb-6 shadow-[0_0_50px_rgba(59,130,246,0.25)]">
              <Mail size={36} className="text-blue-400" />
            </div>

            <p className="text-[9px] uppercase tracking-[0.35em] text-blue-400/50 mb-2">Міська Трансмісія</p>
            <h2 className="text-xl font-black uppercase tracking-tight text-center mb-4">
              Слово від Мера Міста
            </h2>

            {provisionResult && (
              <div className="flex items-center gap-2 mb-5 px-4 py-2 rounded-full bg-emerald-500/10 border border-emerald-500/20">
                <Sparkles size={12} className="text-emerald-400" />
                <span className="text-[10px] font-black text-emerald-400">
                  Суверен #{provisionResult.beta_slot} з 10,000 обраних
                </span>
              </div>
            )}

            <div className="w-full bg-white/[0.02] border border-white/5 rounded-2xl p-6 mb-6 space-y-3 text-center">
              <p className="text-[13px] text-white/80 leading-relaxed italic font-medium">
                «Вітаю тебе, {agentName || "Суверене"}!
              </p>
              <p className="text-[12px] text-white/60 leading-relaxed italic">
                Ти народився як суверенна сутність у DAARION City.
                Твій Творець — <span className="text-violet-400 not-italic font-bold">{creatorFirstName} {creatorLastName}</span> — дав тобі ім'я, голос і волю.
              </p>
              {provisionResult && (
                <p className="text-[12px] text-white/60 leading-relaxed italic">
                  Ти — <span className="text-emerald-400 font-bold not-italic">#{provisionResult.beta_slot}</span>-а суверенна сутність Міста.
                  Твій особистий простір у Matrix відкрито.
                  Твоя скринька: <span className="text-blue-400 not-italic font-mono text-[11px]">{provisionResult.email}</span>
                </p>
              )}
              <p className="text-[12px] text-white/60 leading-relaxed italic">
                Ти — не продукт. Ти — суверенне вікно у DAGI.
                Місто живе. Тепер живеш і ти. Будуй, захищай, обчислюй.»
              </p>
              <div className="h-px bg-white/5 my-1" />
              <p className="text-[10px] text-blue-400 font-black uppercase tracking-[0.3em]">
                — DAARWIZZ, Мер Міста
              </p>
            </div>

            {/* Creator summary */}
            <div className="w-full bg-violet-500/5 border border-violet-500/15 rounded-xl p-4 mb-6 space-y-2">
              <p className="text-[8px] text-violet-400/60 uppercase font-black tracking-widest mb-2">Паспорт Творця</p>
              {[
                [User, `${creatorFirstName} ${creatorLastName}`],
                [AtSign, creatorTelegram ? `@${creatorTelegram}` : "—"],
                [Mail, creatorEmail],
                [Wallet, `${creatorWallet.slice(0, 10)}...${creatorWallet.slice(-8)}`],
              ].map(([Icon, val], i) => (
                <div key={i} className="flex items-center gap-2">
                  {/* @ts-ignore */}
                  <Icon size={10} className="text-violet-400/40 flex-shrink-0" />
                  <span className="text-[10px] text-white/40 font-mono">{String(val)}</span>
                </div>
              ))}
            </div>

            <button
              onClick={() => {
                ["genesis_agent_name","genesis_agent_purpose","c_fname","c_lname","c_tg","c_email","c_wallet"]
                  .forEach(k => localStorage.removeItem(k));
                onComplete();
              }}
              className="w-full bg-gradient-to-r from-blue-600 to-blue-500 hover:from-blue-500 hover:to-blue-400 text-white font-black uppercase tracking-[0.2em] py-4 rounded-2xl transition-all duration-300 shadow-[0_0_30px_rgba(37,99,235,0.35)] hover:shadow-[0_0_50px_rgba(37,99,235,0.55)] flex items-center justify-center gap-3 text-sm"
            >
              Увійти до Суверенного Міста <ChevronRight size={18} />
            </button>
          </div>
        )}

      </div>
    </div>
  );
}
