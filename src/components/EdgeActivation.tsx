import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Server, Cpu, CheckCircle2, AlertCircle, PlayCircle, DownloadCloud, Activity, Zap, Terminal, ChevronRight, Sparkles, Monitor, Key, Lock
} from "lucide-react";

interface CapabilitySummary {
  os: string;
  arch: string;
  hostname: string;
  cpu_count: number;
  cpu_brand: string;
  ram_total_gb: number;
  gpu: {
    detected: boolean;
    vendor: string;
    class: string;
    acceleration_api: string;
  };
}

export function EdgeActivation() {
  const [capabilities, setCapabilities] = useState<CapabilitySummary | null>(null);
  const [devMode, setDevMode] = useState(false);
  
  const [leaseStatus, setLeaseStatus] = useState<'idle' | 'requesting' | 'granted' | 'failed'>('idle');
  const [sessionInfo, setSessionInfo] = useState<{session_id: string, connection_token: string} | null>(null);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

  useEffect(() => {
    async function fetchCaps() {
      try {
        const caps = await invoke<CapabilitySummary>("get_capabilities");
        setCapabilities(caps);
      } catch (e) {
        console.error("Failed to fetch capabilities", e);
      }
    }
    fetchCaps();
  }, []);

  async function requestLease() {
    if (!capabilities) return;
    setLeaseStatus('requesting');
    setErrorMsg(null);
    
    try {
      const res = await fetch("http://127.0.0.1:8181/lease/request", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          node_pubkey: "client-edge-preview-key",
          capabilities: {
            hw_acceleration: capabilities.gpu.detected ? capabilities.gpu.acceleration_api : "none",
            vram_gb: 8, // mock fallback
            promised_minutes: 60
          }
        })
      });
      
      const data = await res.json();
      if (!res.ok) throw new Error(data.detail || "Failed to acquire lease");
      
      setSessionInfo({
         session_id: data.session_id,
         connection_token: data.connection_token
      });
      setLeaseStatus('granted');
    } catch (e) {
      setErrorMsg(String(e));
      setLeaseStatus('failed');
    }
  }

  return (
    <div className="space-y-6 max-w-4xl mx-auto py-4">
       <header className="mb-8">
          <h2 className="text-2xl font-black tracking-tight text-white/90">Desktop Worker <span className="text-blue-500 uppercase text-xs tracking-widest ml-2 border border-blue-500/20 bg-blue-500/10 px-2 py-1 rounded">Preview</span></h2>
          <p className="text-xs text-white/40 uppercase font-black tracking-[0.2em] mt-1">Planned Gateway-Relay Path</p>
       </header>

       <div className="glass p-6 border-blue-500/30 bg-blue-500/[0.05] rounded-2xl mb-8">
          <h3 className="text-sm font-black text-blue-400 uppercase tracking-widest mb-3 flex items-center gap-2">
             <AlertCircle size={16} /> Runtime Boundaries
          </h3>
          <ul className="space-y-3 text-sm text-blue-200/80 leading-relaxed list-disc list-inside">
             <li><strong className="text-white">Client App:</strong> UI, chat, telemetry.</li>
             <li><strong className="text-white">Infrastructure Worker (Canonical):</strong> Real deployment path limited to prepared Linux/Ubuntu.</li>
             <li><strong className="text-white">Desktop Worker Preview:</strong> Experimental outbound relay session. No inbound mesh access.</li>
          </ul>
       </div>

       {!devMode ? (
          <div className="p-8 border border-white/5 bg-white/[0.01] rounded-2xl flex flex-col items-center justify-center min-h-[250px]">
             <div className="w-16 h-16 rounded-full bg-white/5 flex items-center justify-center mb-6">
                <Lock size={24} className="text-white/20" />
             </div>
             <p className="text-sm text-white/40 text-center max-w-md leading-relaxed mb-6">
                Direct infrastructure onboarding is fenced. To test the experimental Gateway-Relay session architecture locally, enable Developer Preview Mode.
             </p>
             <button 
                onClick={() => setDevMode(true)}
                className="px-4 py-2 rounded-lg text-[10px] font-black uppercase tracking-[0.1em] bg-white/5 text-white/50 border border-white/10 hover:bg-white/10 transition-colors"
             >
                Enable Developer Preview Mode
             </button>
          </div>
       ) : (
          <div className="space-y-6 animate-in fade-in duration-500">
             
             {/* Attestation / Trust Class Panel */}
             <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div className="glass p-5 border-white/5">
                   <h3 className="text-[10px] font-black text-white/40 uppercase tracking-widest mb-4 flex items-center gap-2">
                      <Monitor size={14} /> Self-Reported Capabilities
                   </h3>
                   <div className="space-y-3 text-xs font-mono text-white/60">
                      <div className="flex justify-between"><span>Architecture:</span> <span className="text-white">{capabilities?.arch || 'Scanning...'}</span></div>
                      <div className="flex justify-between"><span>RAM (GB):</span> <span className="text-white">{Math.round(capabilities?.ram_total_gb || 0)}</span></div>
                      <div className="flex justify-between"><span>GPU Target:</span> <span className="text-emerald-400">{capabilities?.gpu.detected ? capabilities.gpu.class : 'CPU_ONLY'}</span></div>
                   </div>
                </div>

                <div className="glass p-5 border-yellow-500/20 bg-yellow-500/[0.02]">
                   <h3 className="text-[10px] font-black text-yellow-500/60 uppercase tracking-widest mb-4 flex items-center gap-2">
                      <Key size={14} /> Assigned Trust Class
                   </h3>
                   <div className="p-3 bg-black/40 border border-yellow-500/20 rounded-xl mb-3">
                      <div className="text-sm font-black text-yellow-500 uppercase tracking-widest mb-1">CLASS_0_SELF_REPORTED</div>
                      <p className="text-[10px] text-yellow-200/50 leading-relaxed font-sans">
                         No cryptographic hardware verification present. Capabilities are accepted cautiously and subject to runtime performance degradation mapping.
                      </p>
                   </div>
                </div>
             </div>

             {/* POC Dispatch Panel */}
             <div className="glass p-6 border-blue-500/20 bg-black/20">
                <div className="flex flex-col items-center justify-center py-6">
                   <Server size={32} className={`mb-4 ${leaseStatus === 'granted' ? 'text-emerald-400' : 'text-blue-500/40'}`} />
                   <h3 className="text-lg font-black text-white uppercase tracking-widest mb-2">Gateway-Relay Lease POC</h3>
                   
                   {leaseStatus === 'idle' || leaseStatus === 'failed' ? (
                      <>
                         <p className="text-xs text-white/40 text-center max-w-sm mb-6">
                            Requires active localhost boundary relay (127.0.0.1:8181) from the canonical orchestration repository.
                         </p>
                         <button 
                            onClick={requestLease}
                            disabled={leaseStatus === 'requesting'}
                            className="px-6 py-3 rounded-xl text-xs font-black uppercase tracking-[0.15em] bg-blue-600 hover:bg-blue-500 text-white shadow-xl shadow-blue-900/30 transition-all cursor-pointer"
                         >
                            {leaseStatus === 'requesting' ? 'Requesting...' : 'Request Local Sandbox Lease'}
                         </button>
                         {errorMsg && (
                            <div className="mt-4 text-[10px] text-red-400 font-mono bg-red-500/10 px-3 py-2 rounded">
                               {errorMsg}
                            </div>
                         )}
                      </>
                   ) : leaseStatus === 'granted' ? (
                      <div className="w-full max-w-md bg-emerald-500/10 border border-emerald-500/30 rounded-xl p-4 text-center">
                         <span className="text-[10px] font-black uppercase text-emerald-400 tracking-widest block mb-2">Ephemeral Lease Active</span>
                         <div className="text-[11px] font-mono text-emerald-300 break-all bg-black/50 p-2 rounded mb-3">
                            Session: {sessionInfo?.session_id}
                         </div>
                         <p className="text-[10px] text-emerald-400/60 leading-relaxed">
                            Websocket path allocated. Worker can now dial out to open bounded telemetry duplex.
                         </p>
                      </div>
                   ) : null}
                </div>
             </div>

          </div>
       )}
    </div>
  );
}
