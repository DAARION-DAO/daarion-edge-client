import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  ShieldCheck, 
  Zap, 
  Activity, 
  History, 
  Fingerprint,
  Cuboid, Download, Play, Square, CheckCircle, AlertCircle, Loader2, Database, Globe, TrendingUp,
  ArrowUpRight,
  GitBranch,
  RefreshCcw,
  Compass
} from 'lucide-react';

interface ModelRegistryEntry {
  model_id: string;
  name: string;
  family: string;
  quantization: string;
  size_gb: number;
  runtime: string;
  min_ram_gb: number;
}

interface ModelResidencyStats {
  hits_5m: number;
  hits_15m: number;
  hits_60m: number;
  avg_load_ms: number;
}


interface OptimizationPlan {
  plan_id: string;
  plan_type: string;
  target_scope: string;
  recommended_actions: any[];
  confidence: number;
  priority: "Low" | "Medium" | "High" | "Critical";
  telemetry_basis: string;
}

interface PlacementRecommendation {
  model_id: string;
  scope: "Node" | "District" | "Region";
  decision: "Persistent" | "WarmPreferred" | "ColdOnly" | "AvoidPlacement";
  gravity_score: number;
  confidence: number;
  telemetry_basis: string;
}

interface ApprovalProposal {
  proposal_id: string;
  agent_id: string;
  target_model_id: string;
  action: "PromoteToWarm" | "PromoteToPersistent" | "DowngradeToCold" | "AvoidPlacement" | "DistrictCoDownload";
  scope: "Node" | "District" | "Region";
  reason_summary: string;
  confidence: number;
  trust_scope: string;
  status: "Pending" | "Approved" | "Rejected" | "Vetoed" | "Escalated";
}

interface GovernanceReview {
  review_id: string;
  proposal_id: string;
  reviewing_agent_id: string;
  governance_role: "Architecture" | "Security" | "Operations" | "Placement" | "Compliance";
  decision: "Approve" | "Reject" | "Veto" | "Escalate" | "NeedsHumanReview";
  confidence: number;
  reason_summary: string;
  created_at: number;
}

type ArtifactHolderRole = "PrimaryHolder" | "WarmHolder" | "ColdHolder" | "AvoidHolder";

interface DistrictSpecializationRecommendation {
  district_id: string;
  recommended_class: "Routing" | "SmallLLM" | "Embedding" | "Vision" | "Mixed";
  confidence: number;
  primary_reason: string;
  hardware_fit_score: number;
  demand_alignment: number;
}

interface DistrictSpecializationGap {
  class: "Routing" | "SmallLLM" | "Embedding" | "Vision" | "Mixed";
  gap_delta: number;
  recommendation: string;
}

interface MarketRecommendation {
  recommendation_id: string;
  model_id: string;
  district_id: string;
  action: "IncreaseReplication" | "DecreaseReplication" | "ShiftSpecialization" | "PromoteToPersistent" | "DecommissionArtifact";
  replication_level: number;
  gravity_alignment: number;
  confidence: number;
  reason: string;
}

interface AuthorityTraceEntry {
  layer: string;
  decision: string;
  reason: string;
  latency_ms: number;
}

interface TrustLink {
  type: string;
  id: string;
  scope: string;
  expires_at: string;
  revoked: boolean;
}

interface SentinelSignal {
  signal_id: string;
  class: string;
  raw_value: string;
  normalized_value: number;
  confidence: number;
  freshness: "Fresh" | "Stale" | "Expired";
}

interface DistrictSpecializationActivationIntent {
  intent_id: string;
  district_id: string;
  current_specialization: string;
  proposed_specialization: string;
  rollout_state: "Proposed" | "UnderReview" | "Approved" | "Staged" | "Active" | "Rejected" | "RolledBack";
  confidence: number;
  evidence_basis_ref?: string;
  created_at: string;
}

interface DistrictSpecializationTransition {
  transition_id: string;
  intent: DistrictSpecializationActivationIntent;
  current_completion_percentage: number;
}

interface AuthorityProtocolResult {
  status: "Allowed" | "Blocked" | "Vetoed" | "Escalated" | "RequiresHumanReview";
  source_layer?: string;
  reason: string;
  trace: AuthorityTraceEntry[];
  evidence_confidence: number;
  trust_chain?: TrustLink[];
  sentinel_signals?: SentinelSignal[];
}

interface ModelStatus {
  model_id: string;
  state: "Unloaded" | "Cold" | "Loading" | "Warm" | "Error" | "Downloading";
  progress?: number;
  residency_score?: number;
  stats?: ModelResidencyStats;
  recommendation?: PlacementRecommendation;
  district_role?: ArtifactHolderRole;
  coordination_status?: "Requested" | "Coordinating" | "Confirmed" | "Idle";
  aip_resolution?: AuthorityProtocolResult;
}

export function LocalModelsPanel() {
  const [supportedModels, setSupportedModels] = useState<ModelRegistryEntry[]>([]);
  const [localModels, setLocalModels] = useState<string[]>([]);
  const [modelStatuses, setModelStatuses] = useState<Record<string, ModelStatus>>({});
  const [loading, setLoading] = useState(true);
  const [proposals] = useState<ApprovalProposal[]>([]);
  const [activePlans] = useState<OptimizationPlan[]>([]);
  const [governanceReviews] = useState<Record<string, GovernanceReview[]>>({});
  const [districtRecommendation, setDistrictRecommendation] = useState<DistrictSpecializationRecommendation | null>(null);
  const [activeTransition, setActiveTransition] = useState<DistrictSpecializationTransition | null>(null);
  const [specializationGaps, setSpecializationGaps] = useState<DistrictSpecializationGap[]>([]);
  const [marketRecommendations, setMarketRecommendations] = useState<MarketRecommendation[]>([]);
  async function init() {
    try {
      const [supported, local] = await Promise.all([
        invoke<ModelRegistryEntry[]>("get_supported_models"),
        invoke<string[]>("get_local_models")
      ]);
      setSupportedModels(supported);
      setLocalModels(local);
      
      // Initialize statuses
      const initialStatuses: Record<string, ModelStatus> = {};
      supported.forEach(m => {
        const isLocal = local.includes(`${m.model_id}.gguf`);
        initialStatuses[m.model_id] = {
          model_id: m.model_id,
          state: isLocal ? "Unloaded" : "Cold"
        };
      });
      setModelStatuses(initialStatuses);

      // Simulated fetch of recommendations based on gravity scores
      const mockRecommendations: PlacementRecommendation[] = supported.map(m => ({
        model_id: m.model_id,
        scope: "District",
        decision: Math.random() > 0.7 ? "Persistent" : "WarmPreferred",
        gravity_score: Math.random(),
        confidence: 0.82 + (Math.random() * 0.1),
        telemetry_basis: `High regional demand detected for ${m.family} across ua-west.`
      }));
      
      // We removed the setGlobalRecommendations call here
      
      // Mock Specialization Recommendation
            setDistrictRecommendation({
              district_id: "district-alpha-9",
              recommended_class: "Vision",
              confidence: 0.92,
              primary_reason: "Surge in multi-modal demand; GPU capacity optimal.",
              hardware_fit_score: 0.95,
              demand_alignment: 0.88
            });

            // Mock an active transition for visualization
            setActiveTransition({
              transition_id: "trans_0x456",
              intent: {
                intent_id: "int_0x789",
                district_id: "district-alpha-9",
                current_specialization: "Mixed",
                proposed_specialization: "Vision",
                rollout_state: "Staged",
                confidence: 0.92,
                created_at: new Date().toISOString()
              },
              current_completion_percentage: 45
            });

      setSpecializationGaps([
        {
          class: "Embedding",
          gap_delta: -0.42,
          recommendation: "District shows 42% undersupply in high-memory embedding context."
        }
      ]);

      setMarketRecommendations([
        {
          recommendation_id: "mkt-global-1",
          model_id: "llama-3-8b",
          district_id: "district-ua-west-1",
          action: "IncreaseReplication",
          replication_level: 3,
          gravity_alignment: 0.82,
          confidence: 0.94,
          reason: "Surging demand for 8B-class inference detected across the EU-East backbone. Recommending expansion of persistent holders."
        },
        {
          recommendation_id: "mkt-global-2",
          model_id: "mistral-7b-v0.2",
          district_id: "district-ua-east-4",
          action: "ShiftSpecialization",
          replication_level: 1,
          gravity_alignment: 0.74,
          confidence: 0.81,
          reason: "District hardware fit indicates prime eligibility for embedding workloads to support RAG pipelines."
        }
      ]);
      
      // Attach to status objects as well
      const updatedStatuses = { ...initialStatuses };
      mockRecommendations.forEach(rec => {
        if (updatedStatuses[rec.model_id]) {
            updatedStatuses[rec.model_id].recommendation = rec;
            // Simulated AIP Resolution (v2 Deep Mode)
            updatedStatuses[rec.model_id].aip_resolution = {
              status: Math.random() > 0.1 ? "Allowed" : "Vetoed",
              source_layer: Math.random() > 0.5 ? "Security" : "Architecture",
              reason: "AIP v2 Precedence Gate Passed.",
              evidence_confidence: 0.85 + Math.random() * 0.1,
              trace: [
                { layer: "Identity", decision: "Allow", reason: "DAIS trust binding valid.", latency_ms: 12 },
                { layer: "Security", decision: "Allow", reason: "AISTALK posture nominal.", latency_ms: 24 },
                { layer: "Observability", decision: "Allow", reason: "SENTINEL evidence normalized.", latency_ms: 45 },
                { layer: "Architecture", decision: "Allow", reason: "SOFIIA integrity confirmed.", latency_ms: 8 },
                { layer: "Orchestration", decision: "Allow", reason: "DAARWIZZ intent approved.", latency_ms: 15 },
              ],
              trust_chain: [
                { type: "Identity", id: "did:dais:0x123", scope: "global", expires_at: "2027-01-01", revoked: false },
                { type: "Certificate", id: "cert:0x456", scope: "regional.eu", expires_at: "2026-12-31", revoked: false },
                { type: "Session", id: "sess:0x789", scope: "inference.chat", expires_at: "2026-06-30", revoked: false },
                { type: "Lease", id: "lease:0xabc", scope: "model.activation", expires_at: "2026-03-20", revoked: false },
              ],
              sentinel_signals: [
                { signal_id: "sig_1", class: "Load", raw_value: "CPU: 42%", normalized_value: 0.42, confidence: 0.98, freshness: "Fresh" },
                { signal_id: "sig_2", class: "Health", raw_value: "healthy", normalized_value: 1.0, confidence: 1.0, freshness: "Fresh" },
                { signal_id: "sig_3", class: "Capacity", raw_value: "RAM: 12GB Free", normalized_value: 0.75, confidence: 0.92, freshness: "Fresh" },
              ]
            };
        }
      });
      setModelStatuses(updatedStatuses);

    } catch (e) {
      console.error("Failed to fetch models", e);
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    init();

    const deprogress = listen<{ model_id: string; progress: number }>(
      "model-download-progress",
      (event) => {
        setModelStatuses(prev => ({
          ...prev,
          [event.payload.model_id]: {
            ...prev[event.payload.model_id],
            state: "Downloading",
            progress: event.payload.progress
          }
        }));
      }
    );

    const definished = listen<string>("model-download-finished", (event) => {
      setLocalModels(prev => [...prev, `${event.payload}.gguf`]);
      setModelStatuses(prev => ({
        ...prev,
        [event.payload]: { ...prev[event.payload], state: "Unloaded", progress: 100 }
      }));
    });

    const destatus = listen<{ model_id: string; state: any }>("model-status-changed", (event) => {
        setModelStatuses(prev => ({
            ...prev,
            [event.payload.model_id]: { ...prev[event.payload.model_id], state: event.payload.state }
        }));
    });

    return () => {
      deprogress.then(f => f());
      definished.then(f => f());
      destatus.then(f => f());
    };
  }, []);

  async function handleDownload(model: ModelRegistryEntry) {
    try {
      await invoke("download_model", { entry: model });
    } catch (e) {
      console.error(e);
    }
  }

  async function handleLoad(modelId: string) {
    try {
      await invoke("load_model", { modelId });
    } catch (e) {
      console.error(e);
    }
  }

  async function handleUnload(modelId: string) {
    try {
      await invoke("unload_model", { modelId });
    } catch (e) {
      console.error(e);
    }
  }

  if (loading) return (
    <div className="flex items-center justify-center p-20 text-white/20">
      <Loader2 className="animate-spin mr-2" size={16} />
      <span className="text-[10px] font-bold uppercase tracking-widest">Scanning Registry...</span>
    </div>
  );

  return (
    <div className="space-y-6">
      <header className="flex justify-between items-center mb-2">
        <div>
          <h2 className="text-sm font-bold tracking-tight text-white/80">Local Model Inventory</h2>
          <p className="text-[10px] text-white/30 uppercase font-bold tracking-widest mt-1">Distributed Placement Activation Layer</p>
        </div>
        <div className="flex gap-2">
          <div className="glass px-3 py-1.5 border-white/5 flex items-center gap-2">
            <Database size={12} className="text-blue-500/50" />
            <span className="text-[10px] text-white/40 font-bold uppercase tracking-widest">{localModels.length} ARTIFACTS</span>
          </div>
        </div>
      </header>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {supportedModels.map(model => {
          const status = modelStatuses[model.model_id];
          const isDownloaded = localModels.includes(`${model.model_id}.gguf`);
          
          return (
            <div key={model.model_id} className={`glass p-5 border-white/5 relative overflow-hidden group transition-all duration-500 ${status?.state === 'Warm' ? 'border-emerald-500/20 bg-emerald-500/[0.02]' : ''}`}>
              {status?.state === 'Warm' && (
                <div className="absolute top-0 right-0 p-3">
                  <span className="flex h-2 w-2">
                    <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
                    <span className="relative inline-flex rounded-full h-2 w-2 bg-emerald-500"></span>
                  </span>
                </div>
              )}
              
              <div className="flex justify-between items-start mb-4">
                <div className="flex items-center gap-3">
                  <div className={`p-2 rounded-lg ${isDownloaded ? 'bg-blue-500/10 text-blue-400' : 'bg-white/5 text-white/20'}`}>
                    <Cuboid size={18} />
                  </div>
                  <div>
                    <h3 className="text-xs font-bold text-white/90">{model.name}</h3>
                    <div className="flex gap-2 mt-1">
                      <span className="text-[8px] text-white/20 font-black uppercase tracking-widest">{model.family}</span>
                      <span className="text-[8px] text-white/40 font-bold">{model.quantization}</span>
                      {status?.recommendation && (
                        <span className={`text-[8px] font-black uppercase tracking-widest px-1 rounded bg-blue-500/10 text-blue-400`}>
                          Gravity: {Math.round(status.recommendation.gravity_score * 100)}G
                        </span>
                      )}
                      {status?.district_role && (
                        <span className="text-[8px] font-black uppercase tracking-widest px-1 rounded bg-white/5 text-white/40 border border-white/5">
                          {status.district_role}
                        </span>
                      )}
                    </div>
                  </div>
                </div>
                <div className="text-right">
                  <span className="text-[10px] font-mono text-white/30">{model.size_gb} GB</span>
                </div>
              </div>

               {status?.recommendation && (
                <div className="mb-4 p-3 rounded-lg bg-blue-500/[0.03] border border-blue-500/10">
                   <div className="flex justify-between items-center mb-1">
                      <div className="flex items-center gap-2">
                        <ShieldCheck size={10} className="text-blue-400" />
                        <span className="text-[8px] text-white/20 uppercase font-black tracking-widest">Placement Recommendation</span>
                      </div>
                      <span className={`text-[8px] font-black uppercase ${
                        status.recommendation.decision === 'Persistent' ? 'text-emerald-400' : 'text-blue-400'
                      }`}>
                         {status.recommendation.decision} ({status.recommendation.scope})
                      </span>
                   </div>
                   <p className="text-[9px] text-white/40 italic leading-tight mb-2">
                      "{status.recommendation.telemetry_basis}"
                   </p>
                   
                   {/* AIP Resolution Badge (v2 Deep Mode) */}
                   <div className={`mb-4 rounded-xl border overflow-hidden ${
                     status.aip_resolution?.status === 'Allowed' ? 'bg-emerald-500/[0.03] border-emerald-500/20' : 
                     status.aip_resolution?.status === 'Vetoed' ? 'bg-red-500/[0.03] border-red-500/20' : 'bg-white/5 border-white/10'
                   }`}>
                      <div className="p-3 border-b border-white/5 flex items-center justify-between">
                         <div className="flex items-center gap-2">
                            <ShieldCheck size={12} className={status.aip_resolution?.status === 'Allowed' ? 'text-emerald-400' : 'text-red-400'} />
                            <span className="text-[9px] text-white/60 font-black uppercase tracking-widest">AIP v2 Enforcement</span>
                         </div>
                         <div className="flex items-center gap-2">
                             <span className="text-[8px] text-white/20 font-bold uppercase">Evidence:</span>
                             <span className="text-[8px] text-emerald-400 font-mono">{(status.aip_resolution?.evidence_confidence || 0 * 100).toFixed(0)}%</span>
                             <span className={`px-1.5 py-0.5 rounded text-[7px] font-black uppercase ${
                               status.aip_resolution?.status === 'Allowed' ? 'bg-emerald-500/10 text-emerald-400' : 'bg-red-500/10 text-red-400'
                             }`}>
                                {status.aip_resolution?.status || 'GATE_PENDING'}
                             </span>
                         </div>
                      </div>
                      
                      {status.aip_resolution?.sentinel_signals && (
                        <div className="p-3 bg-emerald-500/[0.02] border-t border-white/5">
                           <div className="flex items-center gap-2 mb-2">
                             <TrendingUp size={10} className="text-emerald-400" />
                             <span className="text-[7px] text-white/40 font-black uppercase tracking-widest">Standardized Evidence</span>
                           </div>
                           <div className="space-y-1.5 ml-1 border-l border-white/5 pl-3">
                              {status.aip_resolution.sentinel_signals.map((signal, idx) => (
                                <div key={idx} className="flex flex-col">
                                   <div className="flex items-center justify-between">
                                      <span className="text-[8px] text-white/80 font-bold uppercase">{signal.class}</span>
                                      <span className="text-[7px] text-emerald-400 font-mono font-black">{(signal.normalized_value * 100).toFixed(0)}%</span>
                                   </div>
                                   <div className="flex items-center justify-between">
                                      <span className="text-[7px] text-white/40 italic">Raw: {signal.raw_value}</span>
                                      <div className="flex items-center gap-2">
                                         <span className="text-[6px] text-white/20 uppercase font-bold">Conf: {(signal.confidence * 100).toFixed(0)}%</span>
                                         <span className="text-[6px] text-emerald-500/60 uppercase font-black">{signal.freshness}</span>
                                      </div>
                                   </div>
                                </div>
                              ))}
                           </div>
                        </div>
                      )}
                      
                      {status.aip_resolution?.trust_chain && (
                        <div className="p-3 bg-white/[0.02] border-t border-white/5">
                           <div className="flex items-center gap-2 mb-2">
                             <Fingerprint size={10} className="text-blue-400" />
                             <span className="text-[7px] text-white/40 font-black uppercase tracking-widest">Propagation Chain</span>
                           </div>
                           <div className="space-y-1.5 ml-1 border-l border-white/5 pl-3">
                              {status.aip_resolution.trust_chain.map((link, idx) => (
                                <div key={idx} className="flex flex-col">
                                   <div className="flex items-center justify-between">
                                      <span className="text-[8px] text-white/80 font-bold">{link.type}</span>
                                      <span className="text-[6px] text-white/20 font-mono">{link.id}</span>
                                   </div>
                                   <div className="flex items-center justify-between">
                                      <span className="text-[7px] text-white/40 italic">Scope: {link.scope}</span>
                                      <span className="text-[6px] text-white/20">Exp: {link.expires_at}</span>
                                   </div>
                                </div>
                              ))}
                           </div>
                        </div>
                      )}
                   </div>

                   {status.recommendation.telemetry_basis.includes("District") && (
                     <div className="flex gap-2 mb-2 p-1.5 rounded bg-blue-500/10 border border-blue-500/20">
                        <div className="flex items-center gap-1">
                           <Zap size={8} className="text-blue-400" />
                           <span className="text-[7px] text-white/60 font-black uppercase tracking-widest">District Pulse Active</span>
                        </div>
                     </div>
                   )}
                   <div className="flex items-center justify-between">
                        <span className="text-[8px] text-white/10 uppercase font-bold">Confidence: {status.recommendation.confidence.toFixed(2)}</span>
                        {status.coordination_status && status.coordination_status !== "Idle" ? (
                           <div className="flex items-center gap-1.5 px-2 py-0.5 rounded bg-blue-500/20 border border-blue-500/30">
                              <Loader2 size={8} className="animate-spin text-blue-400" />
                              <span className="text-[7px] text-blue-400 font-black uppercase tracking-widest">District Pulse: {status.coordination_status}</span>
                           </div>
                        ) : (
                           <button className="text-[8px] font-black uppercase tracking-widest text-blue-400 hover:text-blue-300 transition-colors">Approve Intent</button>
                        )}
                   </div>
                </div>
              )}

              <div className="flex items-center justify-between pt-4 border-t border-white/5">
                <div className="flex items-center gap-3">
                   {status?.state === "Downloading" ? (
                     <div className="flex items-center gap-2">
                        <div className="w-16 h-1 bg-white/5 rounded-full overflow-hidden">
                           <div className="h-full bg-blue-500 transition-all duration-300" style={{ width: `${status.progress}%` }} />
                        </div>
                        <span className="text-[8px] font-black font-mono text-blue-400">{status.progress}%</span>
                     </div>
                   ) : (
                     <div className="flex items-center gap-2">
                        {isDownloaded ? (
                          <div className={`flex items-center gap-1.5 text-[9px] font-bold uppercase tracking-wider ${status?.state === 'Warm' ? 'text-emerald-400' : 'text-white/40'}`}>
                            {status?.state === "Warm" ? <CheckCircle size={10} /> : <div className="w-1 h-1 rounded-full bg-white/20" />}
                            {status?.state}
                          </div>
                        ) : (
                          <div className="flex items-center gap-1.5 text-[9px] font-bold uppercase tracking-wider text-white/20">
                            <AlertCircle size={10} /> Not Local
                          </div>
                        )}
                     </div>
                   )}
                </div>

                <div className="flex gap-2">
                  {!isDownloaded && status?.state !== "Downloading" && (
                    <button onClick={() => handleDownload(model)} className="flex items-center gap-2 px-3 py-1.5 rounded-md bg-white/5 hover:bg-white/10 text-[9px] font-bold uppercase tracking-widest transition-all">
                      <Download size={12} /> Download
                    </button>
                  )}
                  {isDownloaded && status?.state !== "Warm" && status?.state !== "Loading" && (
                    <button onClick={() => handleLoad(model.model_id)} className="flex items-center gap-2 px-3 py-1.5 rounded-md bg-blue-600/20 text-blue-400 hover:bg-blue-600/30 text-[9px] font-bold uppercase tracking-widest transition-all border border-blue-500/20">
                      <Play size={12} fill="currentColor" /> Load Model
                    </button>
                  )}
                   {status?.state === "Loading" && (
                     <div className="flex items-center gap-2 px-3 py-1.5 text-blue-400 animate-pulse">
                        <Loader2 className="animate-spin" size={12} />
                        <span className="text-[9px] font-bold uppercase tracking-widest">Loading...</span>
                     </div>
                   )}
                  {status?.state === "Warm" && (
                    <button onClick={() => handleUnload(model.model_id)} className="flex items-center gap-2 px-3 py-1.5 rounded-md bg-red-500/10 text-red-400 hover:bg-red-500/20 text-[9px] font-bold uppercase tracking-widest transition-all border border-red-500/20">
                      <Square size={12} fill="currentColor" /> Unload
                    </button>
                  )}
                </div>
              </div>
            </div>
          );
        })}
      </div>
      <section className="glass p-5 border-white/5 bg-blue-500/[0.01]">
         <div className="flex items-start gap-4">
            <div className="p-2 bg-blue-500/10 rounded-lg text-blue-400">
               <Zap size={16} />
            </div>
            <div>
               <h4 className="text-[10px] font-black uppercase tracking-[0.2em] text-blue-400/80 mb-1">Activation Layer Note</h4>
               <p className="text-[11px] text-white/40 leading-relaxed max-w-xl">
                 Recommendations are derived from regional telemetry and hardware fit. Approving an intent transforms a recommendation into a scheduled activation. 
                 Zero-touch activation is currently disabled for M1.5 nodes.
               </p>
            </div>
         </div>
      </section>

      {proposals.length > 0 && (
        <section className="glass p-5 border-blue-500/20 bg-blue-500/[0.02]">
          <div className="flex justify-between items-center mb-4">
            <h3 className="text-[10px] text-white/20 font-black uppercase tracking-widest">Agent Optimization Proposals</h3>
            <span className="text-[8px] px-2 py-0.5 rounded-full bg-blue-500/20 text-blue-400 font-bold uppercase">{proposals.length} PENDING</span>
          </div>
          <div className="space-y-3">
            {proposals.map(prop => (
              <div key={prop.proposal_id} className="p-4 rounded border border-white/5 bg-white/[0.01] flex items-start justify-between">
                <div className="space-y-2">
                  <div className="flex items-center gap-3">
                    <span className="text-[9px] font-black text-white/80 uppercase tracking-tighter">{prop.agent_id}</span>
                    <span className="text-white/10">/</span>
                    <span className="text-[9px] font-bold text-blue-400 uppercase">{prop.action}</span>
                    <span className="text-[8px] px-1.5 py-0.5 rounded bg-white/5 text-white/40 font-black uppercase tracking-widest">{prop.scope}</span>
                  </div>
                  <p className="text-[10px] text-white/40 italic">"{prop.reason_summary}"</p>
                  <div className="flex items-center gap-4 text-[8px] font-bold uppercase tracking-widest text-white/20">
                    <span>Trust: {prop.trust_scope}</span>
                    <span>Confidence: {(prop.confidence * 100).toFixed(0)}%</span>
                  </div>
                </div>
                <div className="flex gap-2">
                  <button className="px-3 py-1.5 rounded-md bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 text-[8px] font-black uppercase tracking-widest hover:bg-emerald-500/20 transition-all">Evaluate Policy</button>
                  <button className="px-3 py-1.5 rounded-md bg-white/5 text-white/30 border border-white/5 text-[8px] font-black uppercase tracking-widest hover:bg-white/10 transition-all">Dismiss</button>
                </div>
              </div>
            ))}
          </div>
        </section>
      )}

      {activePlans.length > 0 && (
        <section className="glass p-5 border-emerald-500/20 bg-emerald-500/[0.02]">
          <div className="flex justify-between items-center mb-4">
            <h3 className="text-[10px] text-white/20 font-black uppercase tracking-widest">Active Optimization Plans</h3>
            <span className="text-[8px] px-2 py-0.5 rounded-full bg-emerald-500/20 text-emerald-400 font-bold uppercase">{activePlans.length} MONITORING</span>
          </div>
          <div className="space-y-3">
            {activePlans.map(plan => (
              <div key={plan.plan_id} className="p-4 rounded border border-white/5 bg-white/[0.01]">
                <div className="flex justify-between items-start mb-2">
                  <div className="flex items-center gap-3">
                    <span className="text-[9px] font-black text-white/80 uppercase tracking-tighter">{plan.plan_type}</span>
                    <span className="text-white/10">/</span>
                    <span className="text-[8px] px-1.5 py-0.5 rounded bg-emerald-500/20 text-emerald-400 font-bold uppercase">{plan.priority}</span>
                  </div>
                  <div className="text-[8px] text-white/20 font-bold uppercase tracking-widest">Confidence: {(plan.confidence * 100).toFixed(0)}%</div>
                </div>
                <p className="text-[10px] text-white/60 mb-3">"{plan.telemetry_basis}"</p>
                <div className="flex justify-between items-center">
                   <div className="text-[8px] text-white/10 uppercase font-black tracking-widest">Scope: {plan.target_scope}</div>
                   <button className="text-[8px] font-black uppercase tracking-widest text-emerald-400 hover:text-emerald-300 transition-colors">Review Actions</button>
                </div>
              </div>
            ))}
          </div>
        </section>
      )}

      {proposals.length > 0 && (
        <section className="glass p-5 border-amber-500/20 bg-amber-500/[0.02]">
          <div className="flex justify-between items-center mb-6">
            <div className="flex items-center gap-3">
              <ShieldCheck size={16} className="text-amber-400" />
              <h3 className="text-[10px] text-white/20 font-black uppercase tracking-[0.2em]">Multi-Agent Governance Pulse</h3>
            </div>
            <span className="text-[8px] px-2 py-0.5 rounded-full bg-amber-500/10 text-amber-400 font-bold uppercase border border-amber-500/20">Quorum Active</span>
          </div>

          <div className="space-y-6">
            {proposals.map(prop => {
              const reviews = governanceReviews[prop.proposal_id] || [];
              const isVetoed = reviews.some(r => r.decision === 'Veto');
              const isEscalated = reviews.some(r => r.decision === 'Escalate' || r.decision === 'NeedsHumanReview');

              return (
                <div key={prop.proposal_id} className="relative">
                  <div className="p-4 rounded-xl border border-white/5 bg-white/[0.01] mb-3">
                    <div className="flex justify-between items-start mb-4">
                      <div>
                        <div className="flex items-center gap-2 mb-1">
                          <span className="text-[9px] font-black text-white/80 uppercase tracking-tighter">{prop.action}</span>
                          <span className="text-white/10">|</span>
                          <span className="text-[9px] font-bold text-amber-400/80 uppercase">{prop.target_model_id}</span>
                        </div>
                        <p className="text-[10px] text-white/40 italic">"{prop.reason_summary}"</p>
                      </div>
                      <div className="text-right">
                         <div className={`text-[9px] font-black uppercase tracking-widest ${
                           isVetoed ? 'text-red-400' : isEscalated ? 'text-amber-400' : 'text-blue-400'
                         }`}>
                           {isVetoed ? 'VETOED' : isEscalated ? 'ESCALATED' : 'REVIEW_IN_PROGRESS'}
                         </div>
                      </div>
                    </div>

                    <div className="grid grid-cols-5 gap-2">
                       {["Architecture", "Security", "Operations", "Placement", "Compliance"].map(role => {
                         const review = reviews.find(r => r.governance_role === role);
                         return (
                           <div key={role} className={`p-2 rounded border ${
                             review?.decision === 'Approve' ? 'bg-emerald-500/10 border-emerald-500/20 text-emerald-400' :
                             review?.decision === 'Veto' ? 'bg-red-500/10 border-red-500/20 text-red-400' :
                             review?.decision === 'Reject' ? 'bg-orange-500/10 border-orange-500/20 text-orange-400' :
                             review ? 'bg-amber-500/10 border-amber-500/20 text-amber-400' :
                             'bg-white/5 border-white/5 text-white/20'
                           } transition-all duration-300`}>
                             <span className="text-[7px] font-black uppercase block tracking-tighter mb-1">{role}</span>
                             <div className="text-[8px] font-bold uppercase tracking-widest truncate">
                                {review?.decision || 'WAITING'}
                             </div>
                           </div>
                         );
                       })}
                    </div>
                  </div>
                  
                  {(isVetoed || isEscalated) && (
                    <div className="flex gap-2 justify-end mb-4">
                       <button className="px-4 py-2 rounded-lg bg-emerald-600 text-white text-[9px] font-black uppercase tracking-widest shadow-lg shadow-emerald-900/20">Human Override</button>
                       <button className="px-4 py-2 rounded-lg bg-white/5 text-white/40 text-[9px] font-black uppercase tracking-widest border border-white/5">Details</button>
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        </section>
      )}

      {districtRecommendation && (
        <section className="glass p-6 border-indigo-500/20 bg-indigo-500/[0.02]">
           <div className="flex justify-between items-center mb-6">
              <div className="flex items-center gap-3">
                 <Globe size={18} className="text-indigo-400" />
                 <h3 className="text-[11px] text-white/20 font-black uppercase tracking-[0.25em]">District Specialization Pulse</h3>
              </div>
              <div className="flex gap-2">
                 <span className="text-[8px] px-2 py-0.5 rounded-full bg-indigo-500/10 text-indigo-400 font-bold uppercase border border-indigo-500/20">Strategy Engine Active</span>
              </div>
           </div>

           <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
              <div className="lg:col-span-8 space-y-4">
                 <div className="p-4 rounded-xl border border-white/5 bg-white/[0.01]">
                    <div className="flex justify-between items-start mb-4">
                       <div>
                          <span className="text-[9px] text-white/20 uppercase font-black block mb-1">Recommended Strategy</span>
                          <div className="flex items-baseline gap-2">
                             <h4 className="text-xl font-bold text-white tracking-tighter uppercase">{districtRecommendation.recommended_class}</h4>
                             <span className="text-[10px] text-indigo-400 font-mono font-bold">CONFIDENCE: {(districtRecommendation.confidence * 100).toFixed(0)}%</span>
                          </div>
                       </div>
                    </div>
                    <p className="text-[11px] text-white/60 leading-relaxed italic mb-4">
                       "{districtRecommendation.primary_reason}"
                    </p>
                    <div className="grid grid-cols-2 gap-4">
                       <div className="p-3 rounded-lg bg-white/5 border border-white/5">
                          <span className="text-[8px] text-white/20 uppercase font-black block mb-1">Hardware Fitness</span>
                          <div className="h-1 bg-white/5 rounded-full overflow-hidden mb-2">
                             <div className="h-full bg-indigo-500 transition-all" style={{ width: `${districtRecommendation.hardware_fit_score * 100}%` }} />
                          </div>
                          <span className="text-[10px] font-mono text-white/40">SCORE: {districtRecommendation.hardware_fit_score.toFixed(2)}</span>
                       </div>
                       <div className="p-3 rounded-lg bg-white/5 border border-white/5">
                          <span className="text-[8px] text-white/20 uppercase font-black block mb-1">Demand Alignment</span>
                          <div className="h-1 bg-white/5 rounded-full overflow-hidden mb-2">
                             <div className="h-full bg-emerald-500 transition-all" style={{ width: `${districtRecommendation.demand_alignment * 100}%` }} />
                          </div>
                          <span className="text-[10px] font-mono text-white/40">SCORE: {districtRecommendation.demand_alignment.toFixed(2)}</span>
                       </div>
                    </div>
                 </div>
              </div>

               <div className="lg:col-span-4 space-y-4">
                  <div className="flex flex-col h-full">
                     <span className="text-[9px] text-white/20 uppercase font-black block mb-2">Capabilities Gap Analysis</span>
                     <div className="flex-1 space-y-2">
                        {specializationGaps.length > 0 ? specializationGaps.map((gap, i) => (
                          <div key={i} className="p-3 rounded-lg border border-orange-500/20 bg-orange-500/5 flex items-start gap-3">
                             <AlertCircle size={14} className="text-orange-400 mt-0.5" />
                             <div>
                                <span className="text-[9px] font-black text-orange-400 uppercase tracking-tighter">Gap: {gap.class}</span>
                                <p className="text-[10px] text-white/40 leading-tight mt-0.5">{gap.recommendation}</p>
                             </div>
                          </div>
                        )) : (
                          <div className="h-full flex items-center justify-center border border-white/5 rounded-xl border-dashed">
                             <span className="text-[9px] text-white/10 uppercase font-black tracking-widest">No Critical Gaps</span>
                          </div>
                        )}
                     </div>

                     {activeTransition ? (
                       <div className="mt-4 p-4 rounded-xl border border-indigo-500/30 bg-indigo-500/5">
                          <div className="flex items-center justify-between mb-3">
                             <div className="flex items-center gap-2">
                                <History size={12} className="text-indigo-400 animate-spin-slow" />
                                <span className="text-[9px] font-black text-white/80 uppercase tracking-widest">Transition: {activeTransition.intent.rollout_state}</span>
                             </div>
                             <span className="text-[9px] font-mono text-indigo-400 font-bold">{activeTransition.current_completion_percentage}%</span>
                          </div>
                          <div className="h-1.5 w-full bg-white/5 rounded-full overflow-hidden mb-3">
                             <div 
                                className="h-full bg-indigo-500 transition-all duration-1000 ease-in-out" 
                                style={{ width: `${activeTransition.current_completion_percentage}%` }} 
                             />
                          </div>
                          <div className="flex items-center justify-between">
                            <span className="text-[7px] text-white/20 uppercase font-bold">{activeTransition.intent.current_specialization}</span>
                            <ArrowUpRight size={10} className="text-white/10" />
                            <span className="text-[7px] text-indigo-400 uppercase font-black">{activeTransition.intent.proposed_specialization}</span>
                          </div>
                       </div>
                     ) : (
                       <button className="mt-4 w-full py-2 rounded-lg bg-indigo-600/20 text-indigo-400 border border-indigo-500/20 text-[9px] font-black uppercase tracking-[0.2em] hover:bg-indigo-600/30 transition-all">
                          Propose Strategy Shift
                       </button>
                     )}
                  </div>
               </div>
           </div>
        </section>
      )}

      {marketRecommendations.length > 0 && (
        <section className="glass p-6 border-emerald-500/20 bg-emerald-500/[0.02]">
           <div className="flex justify-between items-center mb-6">
              <div className="flex items-center gap-3">
                 <Activity size={18} className="text-emerald-400" />
                 <h3 className="text-[11px] text-white/20 font-black uppercase tracking-[0.25em]">Global Resource Economy</h3>
              </div>
              <div className="flex gap-2">
                 <span className="text-[8px] px-2 py-0.5 rounded-full bg-emerald-500/10 text-emerald-400 font-bold uppercase border border-emerald-500/20">Market Coordination Peer Active</span>
              </div>
           </div>

            <div className="space-y-4">
               {marketRecommendations.map(mkt => (
                 <div key={mkt.recommendation_id} className="p-5 rounded-2xl border border-white/5 bg-white/[0.01] flex flex-col md:flex-row gap-6 items-start md:items-center">
                    <div className="flex-1">
                       <div className="flex items-center gap-3 mb-2">
                          <div className="p-2 rounded-lg bg-emerald-500/10 text-emerald-400">
                             <TrendingUp size={16} />
                          </div>
                          <div>
                             <div className="flex items-center gap-2">
                                <span className="text-[10px] font-black text-white/90 uppercase tracking-tighter">{mkt.action}</span>
                                <span className="text-white/10">/</span>
                                <span className="text-[10px] font-bold text-emerald-400 uppercase">{mkt.model_id}</span>
                             </div>
                             <span className="text-[8px] text-white/20 font-bold uppercase tracking-widest">{mkt.district_id}</span>
                          </div>
                       </div>
                       <p className="text-[11px] text-white/40 leading-relaxed italic pr-4">
                          "{mkt.reason}"
                       </p>
                    </div>
                    
                    <div className="w-full md:w-auto flex flex-row md:flex-col gap-4 justify-between items-center md:items-end md:text-right border-t md:border-t-0 md:border-l border-white/5 pt-4 md:pt-0 md:pl-6">
                       <div>
                          <span className="text-[8px] text-white/20 uppercase font-black block mb-0.5">Confidence</span>
                          <span className="text-[11px] font-mono font-bold text-white">{(mkt.confidence * 100).toFixed(0)}%</span>
                       </div>
                       <div>
                          <span className="text-[8px] text-white/20 uppercase font-black block mb-0.5">Gravity Sync</span>
                          <span className="text-[11px] font-mono font-bold text-emerald-400">{(mkt.gravity_alignment * 100).toFixed(0)}%</span>
                       </div>
                    </div>
                 </div>
                ))}
            </div>
         </section>
      )}

      {/* Honey Pulse / Value Circulation */}
      <div className="mt-8 border-t border-white/5 pt-8">
        <div className="flex items-center justify-between mb-6">
          <div>
            <h3 className="text-xl font-black text-white italic tracking-tighter uppercase">Honey Pulse</h3>
            <p className="text-[10px] text-white/30 font-medium uppercase tracking-widest">Value Circulation & Economic Utility</p>
          </div>
          <div className="flex items-center gap-4">
            <div className="flex flex-col items-end">
              <span className="text-[8px] text-white/20 uppercase font-bold">Network Honey Density</span>
              <span className="text-sm font-black text-teal-400 tabular-nums">0.88</span>
            </div>
            <div className="w-px h-8 bg-white/5" />
            <div className="flex flex-col items-end">
              <span className="text-[8px] text-white/20 uppercase font-bold">Economic Confidence</span>
              <span className="text-sm font-black text-emerald-400 tabular-nums">94%</span>
            </div>
            <div className="w-px h-8 bg-white/5" />
            <div className="flex flex-col items-end">
              <span className="text-[8px] text-white/20 uppercase font-bold">Circulation Velocity</span>
              <span className="text-sm font-black text-amber-500 tabular-nums">1.2x</span>
            </div>
          </div>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-8">
          {[
            { id: "V-9901", target: "Mistral-7B-D1", usefulness: 0.92, scarcity: 0.1, status: "Flowing" },
            { id: "V-9902", target: "Llama-3-Spec-D4", usefulness: 0.88, scarcity: 0.85, status: "Liquid" },
            { id: "V-9903", target: "Codestral-Audit-D2", usefulness: 0.95, scarcity: 0.92, status: "Crystallized" },
          ].map((signal) => (
            <div key={signal.id} className="p-4 rounded-xl border border-teal-500/10 bg-teal-500/[0.02] hover:bg-teal-500/[0.05] transition-all group overflow-hidden relative">
              <div className="absolute -right-2 -top-2 w-12 h-12 bg-teal-500/5 rounded-full blur-xl group-hover:bg-teal-500/10 transition-all" />
              <div className="flex justify-between items-center mb-3">
                <span className="text-[8px] font-black text-teal-500/60 uppercase">{signal.id}</span>
                <span className={`text-[7px] font-bold px-1.5 py-0.5 rounded-sm uppercase ${
                  signal.status === 'Flowing' ? 'bg-teal-500/20 text-teal-400' :
                  signal.status === 'Liquid' ? 'bg-amber-500/20 text-amber-400' : 'bg-white/10 text-white/60'
                }`}>{signal.status}</span>
              </div>
              <div className="text-[10px] font-bold text-white/90 mb-4">{signal.target}</div>
              
              <div className="space-y-3">
                <div>
                  <div className="flex justify-between text-[7px] mb-1">
                    <span className="text-white/30 uppercase font-bold">Usefulness</span>
                    <span className="text-teal-400 font-black">{(signal.usefulness * 100).toFixed(0)}%</span>
                  </div>
                  <div className="h-1 bg-white/5 rounded-full overflow-hidden">
                    <div className="h-full bg-teal-500 transition-all duration-1000" style={{ width: `${signal.usefulness * 100}%` }} />
                  </div>
                </div>
                <div>
                  <div className="flex justify-between text-[7px] mb-1">
                    <span className="text-white/30 uppercase font-bold">Scarcity Value</span>
                    <span className="text-amber-500 font-black">{(signal.scarcity * 100).toFixed(0)}%</span>
                  </div>
                  <div className="h-1 bg-white/5 rounded-full overflow-hidden">
                    <div className="h-full bg-amber-500 transition-all duration-1000" style={{ width: `${signal.scarcity * 100}%` }} />
                  </div>
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>



      <section className="glass p-6 border-white/10 bg-white/[0.02]">
        <div className="flex justify-between items-center mb-6">
          <div className="flex items-center gap-3">
            <ShieldCheck size={18} className="text-white/40" />
            <h3 className="text-[11px] text-white/20 font-black uppercase tracking-[0.25em]">Institutional Authority Pulse (v3)</h3>
          </div>
          <span className="text-[8px] px-2 py-0.5 rounded-full bg-teal-500/10 text-teal-400 font-bold uppercase border border-teal-500/20">6-Layer Governance Active</span>
        </div>

        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-6 gap-4">
          {[
            { id: "SOFIIA", name: "Architecture Authority", color: "indigo", level: "Tier 0", status: "Active", reason: "Architectural integrity confirmed." },
            { id: "DAARWIZZ", name: "Orchestration Authority", color: "amber", level: "Tier 0", status: "Active", reason: "Strategic routing active." },
            { id: "DAIS", name: "Identity Authority", color: "blue", level: "Tier 1", status: "Active", reason: "Trust bindings verified." },
            { id: "AISTALK", name: "Security Authority", color: "red", level: "Tier 1", status: "Active", reason: "Threat surface stable." },
            { id: "SENTINEL", name: "Observability Authority", color: "emerald", level: "Tier 1", status: "Active", reason: "Telemetry normalization nominal." },
            { id: "MELISSA", name: "Value Authority", color: "teal", level: "Tier 2", status: "Active", reason: "Honey circulation optimized." },
          ].map((auth) => (
            <div key={auth.id} className={`p-4 rounded-xl border border-${auth.color}-500/20 bg-${auth.color}-500/[0.03] group hover:bg-${auth.color}-500/[0.05] transition-all relative overflow-hidden`}>
              <div className="absolute top-0 right-0 p-1.5 opacity-20 group-hover:opacity-100 transition-opacity">
                <div className="text-[6px] font-mono text-white/40 uppercase tracking-tighter">
                  {auth.id === 'SOFIIA' ? 'Hybrid' : 
                   auth.id === 'DAARWIZZ' ? 'Fabric' : 
                   auth.id === 'AISTALK' ? 'Team' : 
                   auth.id === 'MELISSA' ? 'Hybrid' : 'Fabric'}
                </div>
              </div>
              <div className="flex justify-between items-start mb-2">
                <span className={`text-[8px] font-black uppercase text-${auth.color}-400/80`}>{auth.id}</span>
                <span className="text-[7px] text-white/10 font-bold uppercase">{auth.level}</span>
              </div>
              <h4 className="text-[10px] font-bold text-white/90 mb-1">{auth.name}</h4>
              <p className="text-[8px] text-white/30 leading-tight mb-3 line-clamp-2">"{auth.reason}"</p>
              
              {/* Institutional Subroles Visualization */}
              <div className="flex flex-wrap gap-1 mb-3">
                 {auth.id === 'AISTALK' ? (
                   <>
                     <span className="text-[5px] px-1 py-0.5 rounded-sm bg-white/5 text-white/40 uppercase font-black">Tracer</span>
                     <span className="text-[5px] px-1 py-0.5 rounded-sm bg-white/5 text-white/40 uppercase font-black">Risk</span>
                     <span className="text-[5px] px-1 py-0.5 rounded-sm bg-white/5 text-white/40 uppercase font-black">Audit</span>
                   </>
                 ) : auth.id === 'SOFIIA' ? (
                   <>
                     <span className="text-[5px] px-1 py-0.5 rounded-sm bg-indigo-500/10 text-indigo-400/60 uppercase font-black">Planner</span>
                     <span className="text-[5px] px-1 py-0.5 rounded-sm bg-indigo-500/10 text-indigo-400/60 uppercase font-black">Strategy</span>
                   </>
                 ) : auth.id === 'MELISSA' ? (
                   <>
                     <span className="text-[5px] px-1 py-0.5 rounded-sm bg-teal-500/10 text-teal-400/60 uppercase font-black">Rewarded</span>
                     <span className="text-[5px] px-1 py-0.5 rounded-sm bg-teal-500/10 text-teal-400/60 uppercase font-black">Circulating</span>
                   </>
                 ) : (
                   <span className="text-[5px] px-1 py-0.5 rounded-sm bg-white/5 text-white/20 uppercase font-black">Distributed Fabric</span>
                 )}
              </div>

              <div className="flex items-center gap-2">
                <div className={`w-1.5 h-1.5 rounded-full bg-${auth.color}-500 group-hover:animate-ping`} />
                <span className={`text-[8px] font-black uppercase tracking-widest text-${auth.color}-500/80`}>{auth.status}</span>
              </div>
            </div>
          ))}
        </div>
      </section>

      <section className="glass p-6 border-indigo-500/20 bg-indigo-500/[0.02] mt-8">
        <div className="flex justify-between items-center mb-6">
          <div className="flex items-center gap-3">
            <RefreshCcw size={18} className="text-indigo-400 animate-spin-slow" />
            <h3 className="text-[11px] text-white/20 font-black uppercase tracking-[0.25em]">Evolution Dynamics Pulse (v1)</h3>
          </div>
          <div className="flex items-center gap-4">
            <div className="flex flex-col items-end">
              <span className="text-[8px] text-white/20 uppercase font-bold">Stability Index</span>
              <span className="text-sm font-black text-emerald-400 tabular-nums">0.96</span>
            </div>
            <div className="w-px h-8 bg-white/5" />
            <div className="flex flex-col items-end">
              <span className="text-[8px] text-white/20 uppercase font-bold">Adaptation Rate</span>
              <span className="text-sm font-black text-indigo-400 tabular-nums">12/hr</span>
            </div>
          </div>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
          {[
            { id: "EVO-01", type: "Specialization", target: "District 4", change: "Vision -> Audio", status: "Active", stability: 0.98 },
            { id: "EVO-02", type: "Placement", target: "Llama-3-Spec", change: "Cold -> Warm", status: "Staged", stability: 0.94 },
            { id: "EVO-03", type: "Routing", target: "Shard 7", change: "Weight +15%", status: "Approved", stability: 0.99 },
            { id: "EVO-04", type: "Value", target: "Melisss-Reward", change: "Recalculating", status: "Evaluating", stability: 0.92 },
          ].map((evo) => (
            <div key={evo.id} className="p-4 rounded-xl border border-white/5 bg-white/[0.01] hover:bg-white/[0.03] transition-all group">
              <div className="flex justify-between items-center mb-3">
                <div className="flex items-center gap-2">
                  <GitBranch size={12} className="text-white/20 group-hover:text-indigo-400 transition-colors" />
                  <span className="text-[8px] font-black text-white/40 uppercase">{evo.id}</span>
                </div>
                <span className={`text-[7px] font-bold px-1.5 py-0.5 rounded-sm uppercase ${
                  evo.status === 'Active' ? 'bg-emerald-500/20 text-emerald-400' :
                  evo.status === 'Staged' ? 'bg-amber-500/20 text-amber-400' : 'bg-white/10 text-white/60'
                }`}>{evo.status}</span>
              </div>
              <div className="text-[9px] font-bold text-white/80 mb-1">{evo.type}</div>
              <div className="text-[8px] text-white/40 mb-3">{evo.target} : {evo.change}</div>
              
              <div className="flex items-center justify-between">
                <span className="text-[7px] text-white/20 uppercase font-bold">Stability</span>
                <span className="text-[8px] text-emerald-400 font-mono font-black">{(evo.stability * 100).toFixed(0)}%</span>
              </div>
            </div>
          ))}
        </div>
      </section>

      <section className="glass p-6 border-cyan-500/20 bg-cyan-500/[0.02] mt-8">
        <div className="flex justify-between items-center mb-6">
          <div className="flex items-center gap-3">
            <Compass size={18} className="text-cyan-400 animate-pulse" />
            <h3 className="text-[11px] text-white/20 font-black uppercase tracking-[0.25em]">Conscious Coordination Pulse (v1)</h3>
          </div>
          <div className="flex items-center gap-4">
            <div className="flex flex-col items-end">
              <span className="text-[8px] text-white/20 uppercase font-bold">Alignment Index</span>
              <span className="text-sm font-black text-emerald-400 tabular-nums">0.98</span>
            </div>
            <div className="w-px h-8 bg-white/5" />
            <div className="flex flex-col items-end">
              <span className="text-[8px] text-white/20 uppercase font-bold">Global Direction</span>
              <span className="text-[10px] font-black text-cyan-400 uppercase tracking-tighter">Steady Expansion</span>
            </div>
          </div>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          <div className="space-y-4">
            <h4 className="text-[9px] font-black text-white/40 uppercase tracking-widest pl-1">Divergence Detectors</h4>
            {[
              { label: "Spec Drift", value: 0.12, status: "Normal" },
              { label: "Value Gap", value: 0.05, status: "Aligned" },
              { label: "Resource Sync", value: 0.08, status: "Normal" },
            ].map((det) => (
              <div key={det.label} className="p-3 rounded-lg border border-white/5 bg-white/[0.01]">
                <div className="flex justify-between items-center mb-2">
                  <span className="text-[9px] font-bold text-white/60">{det.label}</span>
                  <span className="text-[7px] font-black text-emerald-400 uppercase">{det.status}</span>
                </div>
                <div className="h-1 w-full bg-white/5 rounded-full overflow-hidden">
                  <div className="h-full bg-cyan-500/40" style={{ width: `${det.value * 100}%` }} />
                </div>
              </div>
            ))}
          </div>

          <div className="md:col-span-2 space-y-4">
            <h4 className="text-[9px] font-black text-white/40 uppercase tracking-widest pl-1">Alignment Vectors</h4>
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
              {[
                { area: "District Alignment", vector: "Synergistic", pressure: "Low" },
                { area: "Evolution Harmony", vector: "Coherent", pressure: "Minimum" },
                { area: "Market Equilibrium", vector: "Balanced", pressure: "Optimal" },
                { area: "Network Resonance", vector: "High", pressure: "Nominal" },
              ].map((vec) => (
                <div key={vec.area} className="p-3 rounded-lg border border-cyan-500/10 bg-cyan-500/[0.01] flex items-center justify-between group">
                  <div>
                    <div className="text-[9px] font-bold text-white/80">{vec.area}</div>
                    <div className="text-[8px] text-cyan-400/60 font-mono italic">{vec.vector}</div>
                  </div>
                  <div className="text-right">
                    <div className="text-[7px] text-white/20 uppercase font-black mb-1">Pressure</div>
                    <div className="text-[9px] font-black text-white/60">{vec.pressure}</div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>
      </section>
    </div>
  );
}
