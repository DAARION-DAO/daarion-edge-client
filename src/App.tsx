import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Shield, Activity, XCircle, Zap, Terminal, Globe, Monitor, MessageSquare, LayoutDashboard, Cuboid, Sparkles, Clock, CheckCircle, AlertTriangle, RefreshCw } from "lucide-react";
import { MessagingPanel } from "./components/MessagingPanel";
import { EdgeActivation } from "./components/EdgeActivation";
import { LocalInferencePanel } from "./components/LocalInferencePanel";
import { GenesisWizard } from "./components/GenesisWizard";
interface IdentityStatus {
  initialized: boolean;
  node_id: string | null;
  public_key: string | null;
  storage_backend: string;
}

interface EnrollmentState {
  enrolled: boolean;
  node_id: string | null;
  credential_scope: string | null;
  environment: string | null;
  status: string | null;
  heartbeat_interval_sec: number;
  // Registry fields (Gate 1A/1B)
  trust_tier: string | null;
  registry_mode: boolean;
  last_enrollment_error: string | null;
  // Legacy trust chain fields (preserved, unused in registry mode)
  csr_generated: boolean;
  csr_submitted: boolean;
  certificate_issued: boolean;
  issuer_id: string | null;
  region_scope: string | null;
  district_scope: string | null;
  valid_until: number | null;
  next_renewal_at: number | null;
}

interface HeartbeatStatus {
  last_success_at: string | null;
  last_attempt_at: string | null;
  active: boolean;
  consecutive_failures: number;
  last_node_id_prefix: string | null;
  last_task_count: number;
  revoked: boolean;
}

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
  worker_ready: boolean;
  model_runtime_ready: boolean;
}

function App() {
  const [idStatus, setIdStatus] = useState<IdentityStatus | null>(null);
  const [enrollment, setEnrollment] = useState<EnrollmentState | null>(null);
  const [heartbeat, setHeartbeat] = useState<HeartbeatStatus | null>(null);
  const [capabilities, setCapabilities] = useState<CapabilitySummary | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<"dashboard" | "messaging" | "activation" | "inference">("dashboard");

  async function fetchData() {
    try {
      const [idRes, enrollRes, hbRes, capRes] = await Promise.all([
        invoke<IdentityStatus>("get_identity_status"),
        invoke<EnrollmentState>("get_enrollment_status"),
        invoke<HeartbeatStatus>("get_heartbeat_status"),
        invoke<CapabilitySummary>("get_capabilities")
      ]);
      setIdStatus(idRes);
      setEnrollment(enrollRes);
      setHeartbeat(hbRes);
      setCapabilities(capRes);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }



  useEffect(() => {
    fetchData();
    
    const unlisten = listen<HeartbeatStatus>("heartbeat-update", (event) => {
      setHeartbeat(event.payload);
    });

    return () => {
      unlisten.then(f => f());
    };
  }, []);

  if (loading && !idStatus) {
    return (
      <div className="flex items-center justify-center h-screen bg-[#050505] text-white font-sans">
        <Activity className="animate-spin mr-3 text-blue-500" size={24} />
        <span className="text-white/40 tracking-[0.3em] uppercase text-[10px] font-bold">Initializing DAARION Edge...</span>
      </div>
    );
  }

  // Registry lifecycle gate:
  // - Not initialized → GenesisWizard (identity + onboarding)
  // - Initialized but pending (node_id exists, enrolled=false, status="pending") → Pending Approval screen
  // - Enrolled (status="active") → Main dashboard
  const isPending = idStatus?.initialized && enrollment?.node_id && !enrollment?.enrolled;
  const isRevoked = heartbeat?.revoked;

  if (isRevoked) {
    return (
      <div className="flex flex-col items-center justify-center h-screen bg-[#050505] text-white font-sans gap-4 p-8">
        <div className="w-16 h-16 rounded-full bg-red-500/20 border border-red-500/30 flex items-center justify-center">
          <AlertTriangle size={28} className="text-red-400" />
        </div>
        <h1 className="text-lg font-black uppercase tracking-widest text-red-400">Node Revoked</h1>
        <p className="text-white/40 text-xs text-center max-w-sm leading-relaxed">
          This device has been revoked by a SOFIIA operator. Contact the network administrator to restore access.
        </p>
        <div className="px-4 py-2 bg-white/5 border border-white/10 rounded-xl">
          <span className="text-[9px] font-mono text-white/30">node: {enrollment?.node_id?.slice(0, 12)}...</span>
        </div>
      </div>
    );
  }

  if (isPending) {
    const hasError = enrollment?.last_enrollment_error;
    return (
      <div className="flex flex-col items-center justify-center h-screen bg-[#050505] text-white font-sans gap-6 p-8">
        <div className="absolute inset-0 pointer-events-none">
          <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[500px] h-[500px] rounded-full bg-amber-900/20 blur-[120px] animate-pulse" />
        </div>
        <div className="z-10 flex flex-col items-center gap-6 max-w-sm w-full">
          <div className="w-20 h-20 rounded-full bg-amber-500/10 border border-amber-500/25 flex items-center justify-center shadow-[0_0_40px_rgba(251,191,36,0.15)]">
            <Clock size={32} className="text-amber-400 animate-pulse" />
          </div>
          <div className="text-center">
            <p className="text-[9px] uppercase tracking-[0.4em] text-amber-400/50 mb-2">SOFIIA Worker Registry</p>
            <h1 className="text-2xl font-black tracking-tight text-white mb-2">Pending Approval</h1>
            <p className="text-white/40 text-xs leading-relaxed">
              Your device has been registered and is awaiting operator approval in the SOFIIA Console.
              Heartbeat is active — the operator can see your node.
            </p>
          </div>

          {/* Node info */}
          <div className="w-full bg-white/[0.03] border border-white/5 rounded-2xl p-4 space-y-3">
            <div className="flex justify-between items-center">
              <span className="text-[9px] text-white/30 uppercase font-black tracking-wider">Node ID</span>
              <span className="text-[10px] font-mono text-white/50">{enrollment?.node_id?.slice(0, 16)}...</span>
            </div>
            <div className="h-px bg-white/5" />
            <div className="flex justify-between items-center">
              <span className="text-[9px] text-white/30 uppercase font-black tracking-wider">Trust Tier</span>
              <span className="text-[10px] font-mono text-amber-400/70">{enrollment?.trust_tier || "low_trust_beta"}</span>
            </div>
            <div className="h-px bg-white/5" />
            <div className="flex justify-between items-center">
              <span className="text-[9px] text-white/30 uppercase font-black tracking-wider">Registry</span>
              <div className="flex items-center gap-1.5">
                <div className={`w-1.5 h-1.5 rounded-full ${heartbeat?.active ? 'bg-emerald-500 animate-pulse' : 'bg-red-500/40'}`} />
                <span className="text-[10px] text-white/50">{heartbeat?.active ? 'Heartbeat Active' : 'Heartbeat Halted'}</span>
              </div>
            </div>
            <div className="h-px bg-white/5" />
            <div className="flex justify-between items-center">
              <span className="text-[9px] text-white/30 uppercase font-black tracking-wider">Environment</span>
              <span className="text-[10px] font-mono text-white/40">{enrollment?.environment || 'beta'}</span>
            </div>
          </div>

          {/* Error state */}
          {hasError && (
            <div className="w-full bg-red-500/5 border border-red-500/20 rounded-xl p-3 flex items-start gap-2">
              <AlertTriangle size={12} className="text-red-400 flex-shrink-0 mt-0.5" />
              <p className="text-[10px] text-red-400/80">{hasError.slice(0, 120)}</p>
            </div>
          )}

          {/* Instructions */}
          <div className="w-full bg-white/[0.02] border border-white/5 rounded-xl p-4">
            <p className="text-[9px] text-white/30 uppercase font-black tracking-widest mb-2">Next Steps</p>
            <ol className="space-y-1.5">
              {[
                [CheckCircle, 'Device registered in SOFIIA registry'],
                [CheckCircle, 'Heartbeat and capabilities synced'],
                [Clock, 'Awaiting operator approval in SOFIIA Console'],
                [Clock, 'App will update automatically when approved'],
              ].map(([Icon, text], i) => (
                <li key={i} className="flex items-center gap-2">
                  {/* @ts-ignore */}
                  <Icon size={10} className={i < 2 ? 'text-emerald-400 flex-shrink-0' : 'text-amber-400/50 flex-shrink-0'} />
                  <span className="text-[10px] text-white/40">{text as string}</span>
                </li>
              ))}
            </ol>
          </div>

          <button
            onClick={() => fetchData()}
            className="flex items-center gap-2 px-6 py-2.5 rounded-xl border border-white/10 text-white/40 hover:text-white/70 hover:border-white/20 text-[10px] font-bold uppercase tracking-widest transition-all"
          >
            <RefreshCw size={12} />
            Check Status
          </button>
        </div>
      </div>
    );
  }

  if (!enrollment?.enrolled || !idStatus?.initialized) {
    return (
       <GenesisWizard onComplete={() => fetchData()} />
    );
  }

  return (
    <div className="min-h-screen bg-[#020202] text-white p-6 md:p-8 font-sans selection:bg-blue-500/30 overflow-x-hidden">
      {/* Top Navigation / Header */}
      <header className="flex justify-between items-start mb-8 max-w-7xl mx-auto">
        <div className="group">
          <h1 className="text-2xl font-black tracking-tighter bg-gradient-to-br from-white via-white to-white/30 bg-clip-text text-transparent group-hover:from-blue-400 group-hover:to-emerald-400 transition-all duration-700">
            DAARION<span className="text-blue-500 font-light ml-1">EDGE</span>
          </h1>
          <div className="flex items-center gap-2 mt-1">
            <span className="text-white/20 text-[9px] font-bold tracking-widest uppercase py-0.5 px-1.5 border border-white/5 rounded">M1 Beta</span>
            <span className={`text-[9px] font-bold tracking-widest uppercase ${
              enrollment?.status?.toLowerCase() === 'active' ? 'text-emerald-500/60' :
              enrollment?.status?.toLowerCase() === 'pending' ? 'text-amber-500/60' :
              enrollment?.status?.toLowerCase() === 'revoked' ? 'text-red-500/60' :
              'text-white/20'
            }`}>
              {enrollment?.status?.toUpperCase() || 'NOT REGISTERED'}
            </span>
            {enrollment?.trust_tier && (
              <span className="text-white/15 text-[8px] font-mono px-1.5 py-0.5 border border-white/5 rounded">{enrollment.trust_tier}</span>
            )}
          </div>
        </div>
        
        <div className="flex gap-2">
          <div className="glass px-3 py-2 border-white/5 flex flex-col items-end">
            <span className="text-[8px] text-white/30 uppercase tracking-widest font-bold">Infrastructure</span>
            <span className="text-[10px] font-mono text-white/70">{enrollment?.environment || 'LOCAL ONLY'}</span>
          </div>
          
          {/* Local Runtime Activity */}
          <div className="glass px-3 py-2 border-white/5 flex items-center gap-2">
            <div className={`w-2 h-2 rounded-full ${heartbeat?.active ? 'bg-emerald-500 shadow-[0_0_10px_#10b981]' : 'bg-red-500/40'}`} />
            <span className="text-[9px] font-bold uppercase tracking-widest text-white/50">{heartbeat?.active ? 'Runtime Active' : 'Runtime Halted'}</span>
          </div>

          {/* Network Sync Truth */}
          <div className="glass px-3 py-2 border-white/5 flex items-center gap-2">
            <div className={`w-2 h-2 rounded-full ${heartbeat?.last_success_at && heartbeat.consecutive_failures === 0 ? 'bg-blue-500 shadow-[0_0_10px_#3b82f6]' : 'bg-amber-500/50'}`} />
            <span className="text-[9px] font-bold uppercase tracking-widest text-white/50">{heartbeat?.last_success_at && heartbeat.consecutive_failures === 0 ? 'Network Sync' : 'Provisional Sync'}</span>
          </div>
        </div>
      </header>

      <main className="max-w-7xl mx-auto space-y-6">
        {/* Tab Switcher */}
        <div className="flex gap-1 p-1 bg-white/[0.03] border border-white/5 rounded-xl w-fit">
          <button 
            onClick={() => setActiveTab("dashboard")}
            className={`flex items-center gap-2 px-4 py-2 rounded-lg text-[10px] font-bold uppercase tracking-widest transition-all ${
              activeTab === "dashboard" ? "bg-white/10 text-white shadow-xl" : "text-white/30 hover:text-white/60"
            }`}
          >
            <LayoutDashboard size={14} /> Dashboard
          </button>
          <button
            onClick={() => setActiveTab("activation")}
            className={`flex items-center gap-2.5 px-6 py-3 rounded-xl text-[11px] font-black uppercase tracking-[0.15em] transition-all duration-300 ${activeTab === 'activation' ? 'bg-blue-600 text-white shadow-lg shadow-blue-900/30 border-blue-500/50' : 'text-white/30 hover:text-white/50 hover:bg-white/5 border-transparent'} border`}
          >
            <Cuboid size={14} className={activeTab === 'activation' ? 'animate-pulse' : ''} />
            Worker Mode <span className="text-[9px] px-1 py-0.5 bg-blue-500/20 rounded">Preview</span>
          </button>

          <button
            onClick={() => setActiveTab("inference")}
            className={`flex items-center gap-2.5 px-6 py-3 rounded-xl text-[11px] font-black uppercase tracking-[0.15em] transition-all duration-300 ${activeTab === 'inference' ? 'bg-emerald-600 text-white shadow-lg shadow-emerald-900/30 border-emerald-500/50' : 'text-white/30 hover:text-white/50 hover:bg-white/5 border-transparent'} border`}
          >
            <Sparkles size={14} className={activeTab === 'inference' ? 'animate-pulse' : ''} />
            Inference
          </button>
          <button 
            onClick={() => setActiveTab("messaging")}
            className={`flex items-center gap-2 px-4 py-2 rounded-lg text-[10px] font-bold uppercase tracking-widest transition-all ${
              activeTab === "messaging" ? "bg-emerald-500/10 text-emerald-400 shadow-xl border border-emerald-500/20" : "text-white/30 hover:text-white/60"
            }`}
          >
            <MessageSquare size={14} /> Matrix Control
          </button>
        </div>

        {activeTab === "dashboard" ? (
          <div className="grid grid-cols-1 lg:grid-cols-12 gap-6 animate-in fade-in duration-500">
          
          {/* Main Operational Status (7 cols) */}
          <div className="lg:col-span-7 space-y-6">
            
            {/* Genesis Wizard handles initial enrollment. */}
            <section className="glass overflow-hidden border-white/5">
              {enrollment?.enrolled && (
                <div className="border-t border-white/5 bg-white/[0.02] p-6">
                  <div className="flex items-center gap-3 text-white/40 mb-4">
                    <Shield size={14} className="text-blue-500" />
                    <h3 className="text-[9px] font-black uppercase tracking-[0.25em]">Hierarchical Trust Chain</h3>
                  </div>
                  <div className="grid grid-cols-2 lg:grid-cols-4 gap-6">
                    <div className="space-y-1">
                      <span className="text-[8px] text-white/20 uppercase font-black block">CSR Status</span>
                      <div className="flex items-center gap-2">
                        <div className={`w-1.5 h-1.5 rounded-full ${enrollment.csr_submitted ? 'bg-blue-500' : 'bg-white/10'}`} />
                        <span className="text-[10px] text-white/60 font-mono italic">{enrollment.csr_submitted ? 'SUBMITTED' : 'NOT_INITIATED'}</span>
                      </div>
                    </div>
                    <div className="space-y-1">
                      <span className="text-[8px] text-white/20 uppercase font-black block">Certificate Window</span>
                      <div className="flex items-center gap-2 text-[10px] text-white/60 font-mono">
                         {enrollment.certificate_issued ? (
                           <>
                             <span className="text-emerald-400 font-bold">ISSUED</span>
                             <span className="text-white/20">/</span>
                             <span>{new Date(enrollment.valid_until || 0).toLocaleDateString()}</span>
                           </>
                         ) : (
                           <span className="text-white/20 italic">PENDING_ISSUANCE</span>
                         )}
                      </div>
                    </div>
                    <div className="space-y-1">
                      <span className="text-[8px] text-white/20 uppercase font-black block">Trust Scope</span>
                      <div className="flex items-center gap-2">
                        <Globe size={10} className="text-white/30" />
                        <span className="text-[10px] text-white/60 font-mono uppercase tracking-tighter">
                          {enrollment.region_scope || 'GLOBAL'}:{enrollment.district_scope || 'ANY'}
                        </span>
                      </div>
                    </div>
                    <div className="space-y-1">
                      <span className="text-[8px] text-white/20 uppercase font-black block">Issuer Authority</span>
                      <div className="text-[10px] text-blue-400 font-mono truncate">{enrollment.issuer_id || 'LOCAL_ORCHESTRATOR'}</div>
                    </div>
                  </div>
                </div>
              )}
            </section>

            {/* Hardware Information Section */}
            <section className="glass p-6 border-white/5">
              <div className="flex items-center gap-3 text-white/40 mb-8 border-b border-white/5 pb-4">
                <Monitor size={16} />
                <h2 className="text-[10px] font-bold uppercase tracking-[0.2em]">Capability Inventory</h2>
              </div>
              
              <div className="grid grid-cols-2 md:grid-cols-4 gap-8">
                <div className="space-y-2">
                  <span className="text-[9px] text-white/10 uppercase font-black block">Processor</span>
                  <div className="flex items-baseline gap-1">
                    <span className="text-xl font-bold">{capabilities?.cpu_count || 0}</span>
                    <span className="text-[10px] text-white/30 font-bold uppercase">Cores</span>
                  </div>
                  <span className="text-[9px] text-white/20 font-medium truncate block">{capabilities?.cpu_brand}</span>
                </div>
                
                <div className="space-y-2">
                  <span className="text-[9px] text-white/10 uppercase font-black block">Memory</span>
                  <div className="flex items-baseline gap-1">
                    <span className="text-xl font-bold">{Math.round(capabilities?.ram_total_gb || 0)}</span>
                    <span className="text-[10px] text-white/30 font-bold uppercase">GB Total</span>
                  </div>
                  <div className="h-1 bg-white/5 rounded-full overflow-hidden w-full max-w-[80px]">
                    <div className="h-full bg-blue-500 w-[40%]" />
                  </div>
                </div>

                <div className="space-y-2">
                  <span className="text-[9px] text-white/10 uppercase font-black block">Compute Class</span>
                  <div className="flex items-baseline gap-1">
                    <span className="text-xl font-bold uppercase">{capabilities?.arch === 'aarch64' ? 'Silicon' : 'x64'}</span>
                  </div>
                  <span className="text-[9px] text-white/20 font-medium">{capabilities?.os}</span>
                </div>

                <div className="space-y-2">
                  <span className="text-[9px] text-white/10 uppercase font-black block">GPU Layer</span>
                  <div className="flex items-center gap-2">
                    {capabilities?.gpu.detected ? (
                      <div className="flex items-center gap-2">
                         <span className="text-xl font-bold uppercase text-emerald-400">Ready</span>
                         <div className="px-1.5 py-0.5 bg-emerald-500/10 border border-emerald-500/20 rounded text-[8px] text-emerald-400 font-bold uppercase">{capabilities.gpu.acceleration_api}</div>
                      </div>
                    ) : (
                      <span className="text-xl font-bold uppercase text-white/10">Disabled</span>
                    )}
                  </div>
                  <span className="text-[9px] text-white/20 font-medium uppercase tracking-tighter">{capabilities?.gpu.vendor} {capabilities?.gpu.class}</span>
                </div>
              </div>
            </section>
          </div>

          {/* Sidebar Telemetry & Diagnostics (5 cols) */}
          <div className="lg:col-span-5 space-y-6">
            
            {/* Live Data Link Section */}
            <section className="glass p-6 space-y-6 relative overflow-hidden group">
               <div className="absolute -right-4 -bottom-4 text-blue-500/5 rotate-12 group-hover:text-blue-500/10 transition-all duration-1000">
                  <Zap size={140} />
               </div>
              
               <div className="flex items-center justify-between border-b border-white/5 pb-4">
                  <div className="flex items-center gap-3 text-white/40">
                    <Activity size={16} />
                    <h2 className="text-[10px] font-bold uppercase tracking-[0.2em]">Signal Heartbeat</h2>
                  </div>
                  <div className="px-2 py-0.5 bg-blue-500/10 text-blue-400 text-[8px] font-bold uppercase rounded border border-blue-500/20">Live Sync</div>
               </div>

               <div className="space-y-6 py-2">
                 <div className="flex justify-between items-center group/item">
                   <div className="flex flex-col">
                      <span className="text-[11px] font-bold text-white/60">Synchronization</span>
                      <span className="text-[9px] text-white/20 font-black uppercase tracking-widest">{heartbeat?.active ? 'Active Loop' : 'System Halted'}</span>
                   </div>
                   <div className="flex items-center gap-1.5">
                      <div className="w-1 h-1 rounded-full bg-blue-500/40" />
                      <div className="w-1 h-1 rounded-full bg-blue-500/60" />
                      <div className="w-1 h-1 rounded-full bg-blue-500 animate-pulse" />
                   </div>
                 </div>

                 <div className="grid grid-cols-2 gap-4">
                   <div className="p-3 bg-white/[0.02] border border-white/5 rounded-xl">
                      <span className="text-[8px] text-white/20 uppercase font-black mb-2 block tracking-widest">Last Success</span>
                      <span className="text-[11px] font-mono text-emerald-400/70">
                        {heartbeat?.last_success_at ? new Date(heartbeat.last_success_at).toLocaleTimeString() : '---'}
                      </span>
                   </div>
                   <div className="p-3 bg-white/[0.02] border border-white/5 rounded-xl">
                      <span className="text-[8px] text-white/20 uppercase font-black mb-2 block tracking-widest">Failures</span>
                      <span className={`text-[11px] font-mono ${heartbeat?.consecutive_failures ? 'text-red-400' : 'text-white/20'}`}>
                         {heartbeat?.consecutive_failures || '0'}
                      </span>
                   </div>
                 </div>
               </div>
            </section>

            {/* Error / Diagnostics Panel */}
            {error && (
              <section className="bg-red-500/5 border border-red-500/20 p-5 rounded-2xl flex items-start gap-4 animate-in slide-in-from-right-4">
                 <div className="mt-1 p-1 bg-red-500/20 rounded">
                    <XCircle size={14} className="text-red-400" />
                 </div>
                 <div className="flex-1">
                    <h4 className="text-[10px] font-bold text-red-400 uppercase tracking-widest mb-1">Security Exception</h4>
                    <p className="text-[11px] text-red-200/60 leading-relaxed font-medium">{error}</p>
                 </div>
                 <button onClick={() => setError(null)} className="text-red-300/30 hover:text-red-300">✕</button>
              </section>
            )}

            {/* Execution Layer Roadmap */}
            <section className="glass p-6">
               <div className="flex items-center gap-3 text-white/40 mb-6">
                 <Terminal size={16} />
                 <h2 className="text-[10px] font-bold uppercase tracking-[0.2em]">Next Planned Capabilities</h2>
               </div>
               
               <div className="space-y-3">
                  <div className="flex items-center justify-between py-2 border-b border-white/5">
                     <span className="text-[11px] font-bold text-white/30 italic">Matrix Control Plane</span>
                     <span className="text-[9px] px-1.5 py-0.5 bg-white/5 text-white/20 rounded uppercase font-black">Slice B</span>
                  </div>
                  <div className="flex items-center justify-between py-2 border-b border-white/5">
                     <span className="text-[11px] font-bold text-white/30 italic">NATS Data Stream</span>
                     <span className="text-[9px] px-1.5 py-0.5 bg-white/5 text-white/20 rounded uppercase font-black">Slice B</span>
                  </div>
                  <div className="flex items-center justify-between py-2">
                     <span className="text-[11px] font-bold text-white/30 italic">Local Model Manager</span>
                     <span className="text-[9px] px-1.5 py-0.5 bg-white/5 text-white/20 rounded uppercase font-black">Slice C</span>
                  </div>
               </div>
            </section>

          </div>
        </div>
        ) : activeTab === "activation" ? (
          <div className="animate-in slide-in-from-bottom-4 duration-500">
             <EdgeActivation />
          </div>
        ) : activeTab === "inference" ? (
          <div className="animate-in slide-in-from-bottom-4 duration-500">
             <LocalInferencePanel />
          </div>
        ) : (
          <div className="animate-in slide-in-from-bottom-4 duration-500">
             <MessagingPanel />
          </div>
        )}
      </main>

      {/* Persistent System Info Bar */}
      <footer className="fixed bottom-0 left-0 right-0 py-3 px-8 bg-black border-t border-white/5 flex justify-between items-center z-50">
        <div className="flex items-center gap-6">
          <div className="flex items-center gap-2">
             <span className="text-[9px] text-white/20 font-black uppercase tracking-widest">Hostname:</span>
             <span className="text-[10px] font-mono text-white/40">{capabilities?.hostname}</span>
          </div>
          <div className="h-3 w-px bg-white/5" />
          <div className="flex items-center gap-2">
             <span className="text-[9px] text-white/20 font-black uppercase tracking-widest">Kernel:</span>
             <span className="text-[10px] font-mono text-white/40 capitalize">{capabilities?.os} {capabilities?.arch}</span>
          </div>
        </div>
        
        <div className="text-[9px] text-white/10 italic">
          DAARION Edge v0.1.0-beta
        </div>
      </footer>
    </div>
  );
}

export default App;
