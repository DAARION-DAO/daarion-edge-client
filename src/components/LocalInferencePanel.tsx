import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Send, Loader2, Sparkles, Clock, Cpu, MessageSquare, AlertCircle } from "lucide-react";

interface LocalInferenceRequest {
  request_id: string;
  model_id: string;
  task_type: string;
  prompt: string;
  max_tokens: number;
  temperature: number;
  stream: boolean;
}

interface StreamEvent {
  session_id: string;
  event_type: "Token" | "Metadata" | "StateTransition" | "Error";
  payload: string;
  timestamp: number;
}

interface LocalInferenceResponse {
  request_id: string;
  status: string;
  model_id: string;
  runtime: string;
  latency_ms: number;
  output_text: string;
}

interface ModelRegistryEntry {
  model_id: string;
  name: string;
}

export function LocalInferencePanel() {
  const [models, setModels] = useState<ModelRegistryEntry[]>([]);
  const [selectedModel, setSelectedModel] = useState("");
  const [prompt, setPrompt] = useState("");
  const [sessionState, setSessionState] = useState<"Idle" | "Queued" | "LoadingModel" | "Running" | "RoutingToNetwork" | "Done" | "Failed">("Idle");
  const [arbitration, setArbitration] = useState<{ decision: string; reason: string; estimated_local_latency_ms: number; estimated_remote_latency_ms: number } | null>(null);
  const [response, setResponse] = useState<LocalInferenceResponse | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [streamingText, setStreamingText] = useState("");
  const [streamMetrics, setStreamMetrics] = useState<{ tps: number; latency: number } | null>(null);

  useEffect(() => {
    async function fetchModels() {
      try {
        const res = await invoke<ModelRegistryEntry[]>("get_supported_models");
        setModels(res);
        if (res.length > 0) setSelectedModel(res[0].model_id);
      } catch (e) {
        console.error(e);
      }
    }
    fetchModels();

    const unlistenStatus = listen<{ request_id: string; state: string; result?: any; reason?: string }>(
      "inference-session-update",
      (event) => {
        setSessionState(event.payload.state as any);
        if (event.payload.state === "Done" && event.payload.result) {
          setResponse(event.payload.result);
          setIsLoading(false);
        } else if (event.payload.state === "Failed") {
            setError("Inference failed");
            setIsLoading(false);
        }
      }
    );

    const unlistenArbitration = listen<any>(
      "inference-arbitration-result",
      (event) => {
        setArbitration(event.payload);
      }
    );

    const unlistenStream = listen<StreamEvent>(
        "inference-stream-event",
        (event) => {
            const { event_type, payload } = event.payload;
            if (event_type === "Token") {
                setStreamingText(prev => prev + payload);
            } else if (event_type === "Metadata") {
                try {
                    const meta = JSON.parse(payload);
                    setStreamMetrics(meta);
                } catch (e) {}
            } else if (event_type === "StateTransition") {
                setSessionState(payload as any);
            } else if (event_type === "Error") {
                setError(payload);
                setIsLoading(false);
            }
        }
    );

    return () => {
      unlistenStatus.then(f => f());
      unlistenArbitration.then(f => f());
      unlistenStream.then(f => f());
    };
  }, []);

  async function handleSend() {
    if (!prompt.trim() || !selectedModel) return;

    setError(null);
    setResponse(null);
    setArbitration(null);
    setStreamingText("");
    setStreamMetrics(null);
    setSessionState("Queued");
    setIsLoading(true);

    const request: LocalInferenceRequest = {
      request_id: crypto.randomUUID(),
      model_id: selectedModel,
      task_type: "chat",
      prompt: prompt,
      max_tokens: 128,
      temperature: 0.3,
      stream: true
    };

    try {
      await invoke("run_local_inference", { request });
    } catch (e) {
      setError(String(e));
      setIsLoading(false);
      setSessionState("Idle");
    }
  }

  return (
    <div className="space-y-6 max-w-4xl mx-auto">
      <header className="flex justify-between items-center mb-2">
        <div>
          <h2 className="text-sm font-bold tracking-tight text-white/80">Local Inference Playground</h2>
          <p className="text-[10px] text-white/30 uppercase font-bold tracking-widest mt-1">Direct Runtime Access via Local Model Manager</p>
        </div>
      </header>

      <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
        {/* Left: Input (8 cols) */}
        <div className="lg:col-span-8 space-y-4">
          <div className="glass p-6 border-white/5 space-y-4">
            <div className="flex gap-4">
               <div className="flex-1">
                  <label className="text-[9px] text-white/20 uppercase font-black mb-2 block tracking-widest">Select Runtime Model</label>
                  <select 
                    value={selectedModel}
                    onChange={(e) => setSelectedModel(e.target.value)}
                    className="w-full bg-black/40 border border-white/10 rounded-lg px-3 py-2 text-xs focus:border-blue-500/50 outline-none text-white/70"
                  >
                    {models.map(m => <option key={m.model_id} value={m.model_id}>{m.name}</option>)}
                  </select>
               </div>
               <div className="w-1/3">
                  <label className="text-[9px] text-white/20 uppercase font-black mb-2 block tracking-widest">Task Type</label>
                  <div className="px-3 py-2 bg-white/5 border border-white/5 rounded-lg text-xs text-white/40 font-bold uppercase">Chat Completion</div>
               </div>
            </div>

            <div className="relative">
              <textarea 
                value={prompt}
                onChange={(e) => setPrompt(e.target.value)}
                placeholder="Enter inference prompt..."
                className="w-full bg-black/40 border border-white/10 rounded-xl px-4 py-4 text-sm focus:border-blue-500/50 outline-none min-h-[160px] resize-none placeholder:text-white/10"
              />
              <button 
                onClick={handleSend}
                disabled={isLoading || !prompt.trim()}
                className={`absolute bottom-4 right-4 p-3 rounded-xl transition-all ${
                  isLoading ? 'bg-white/5 text-white/20' : 'bg-blue-600 text-white hover:bg-blue-500 shadow-xl shadow-blue-900/20'
                }`}
              >
                {isLoading ? <Loader2 size={18} className="animate-spin" /> : <Send size={18} />}
              </button>
            </div>
          </div>

          {streamingText && (
            <div className={`glass p-6 border-opacity-20 animate-in slide-in-from-bottom-2 duration-500 border-blue-500 bg-blue-500/[0.02]`}>
              <div className="flex items-center justify-between mb-4">
                <div className="flex items-center gap-2 font-black uppercase text-[10px] tracking-widest text-blue-400">
                    <Sparkles size={14} /> Streaming Result
                </div>
                {streamMetrics && (
                    <div className="flex gap-4 text-[9px] font-mono text-white/40 uppercase">
                        <span>{streamMetrics.tps.toFixed(1)} tokens/s</span>
                        <span>{streamMetrics.latency}ms</span>
                    </div>
                )}
              </div>
              <p className="text-sm text-white/80 leading-relaxed font-medium whitespace-pre-wrap">
                {streamingText}
              </p>
            </div>
          )}

          {response && !streamingText && (
            <div className={`glass p-6 border-opacity-20 animate-in slide-in-from-bottom-2 duration-500 ${
                response.runtime.includes("Remote") ? "border-blue-500 bg-blue-500/[0.02]" : "border-emerald-500 bg-emerald-500/[0.02]"
            }`}>
              <div className={`flex items-center gap-2 mb-4 font-black uppercase text-[10px] tracking-widest ${
                  response.runtime.includes("Remote") ? "text-blue-400" : "text-emerald-400"
              }`}>
                <Sparkles size={14} /> {response.runtime.includes("Remote") ? "Network Inference" : "Local Inference"} Result
              </div>
              <p className="text-sm text-white/80 leading-relaxed font-medium">
                {response.output_text}
              </p>
            </div>
          )}

          {error && (
            <div className="glass p-4 border-red-500/20 bg-red-500/[0.02] flex items-center gap-3 text-red-400">
               <AlertCircle size={16} />
               <span className="text-xs font-bold">{error}</span>
            </div>
          )}
        </div>

        {/* Right: Telemetry (4 cols) */}
        <div className="lg:col-span-4 space-y-4">
          <div className="glass p-6 border-white/5">
             <h3 className="text-[10px] text-white/20 font-black uppercase tracking-widest mb-6">Execution Telemetry</h3>
             
             <div className="space-y-6">
                <div className="flex justify-between items-center group/item">
                  <span className="text-[11px] font-bold text-white/40 uppercase">Session State</span>
                  <div className="flex items-center gap-2">
                    {sessionState !== "Idle" && <div className={`w-1.5 h-1.5 rounded-full ${sessionState === 'Done' ? 'bg-emerald-500' : 'bg-blue-500 animate-pulse'}`} />}
                    <span className={`text-[10px] font-black uppercase tracking-wider ${sessionState === 'Done' ? 'text-emerald-400' : 'text-blue-400'}`}>{sessionState}</span>
                  </div>
                </div>

                <div className="space-y-1">
                   <div className="flex justify-between text-[11px] font-bold text-white/40 uppercase mb-2">
                      <span>Progress</span>
                      <span className="text-white/20 tracking-tighter">
                        {sessionState === "Idle" ? "0%" : sessionState === "Queued" ? "10%" : (sessionState === "LoadingModel" || sessionState === "RoutingToNetwork") ? "40%" : sessionState === "Running" ? "75%" : "100%"}
                      </span>
                   </div>
                   <div className="h-1 bg-white/5 rounded-full overflow-hidden">
                      <div className={`h-full bg-blue-500 transition-all duration-700 ${
                        sessionState === "Idle" ? "w-0" : 
                        sessionState === "Queued" ? "w-[10%]" : 
                        (sessionState === "LoadingModel" || sessionState === "RoutingToNetwork") ? "w-[40%]" : 
                        sessionState === "Running" ? "w-[75%]" : "w-full"
                      }`} />
                   </div>
                </div>

                {arbitration && (
                    <div className="space-y-3 pt-4 border-t border-white/5">
                        <div className="flex flex-col gap-1">
                            <span className="text-[9px] text-white/20 uppercase font-black tracking-widest">Arbitration Decision</span>
                            <span className={`text-[10px] font-bold ${arbitration.decision === "LocalExecution" ? "text-emerald-400" : "text-blue-400"}`}>
                                {arbitration.decision === "LocalExecution" ? "LOCAL" : "REMOTE FALLBACK"}
                            </span>
                        </div>
                        <p className="text-[10px] text-white/40 italic leading-tight">
                            "{arbitration.reason}"
                        </p>
                        <div className="grid grid-cols-2 gap-2 text-[9px] font-medium text-white/20">
                            <div className="px-2 py-1 bg-white/[0.02] rounded">L: {arbitration.estimated_local_latency_ms}ms</div>
                            <div className="px-2 py-1 bg-white/[0.02] rounded">R: {arbitration.estimated_remote_latency_ms}ms</div>
                        </div>
                    </div>
                )}

                <div className="grid grid-cols-1 gap-3 pt-4 border-t border-white/5">
                   <div className="flex items-center gap-3 p-2 rounded-lg bg-white/[0.02] border border-white/5">
                      <div className="p-1.5 bg-blue-500/10 rounded text-blue-400"><Clock size={12} /></div>
                      <div className="flex flex-col">
                         <span className="text-[8px] text-white/20 uppercase font-black">Latency</span>
                         <span className="text-[10px] font-mono text-white/60">{response?.latency_ms || '---'} ms</span>
                      </div>
                   </div>
                   <div className="flex items-center gap-3 p-2 rounded-lg bg-white/[0.02] border border-white/5">
                      <div className="p-1.5 bg-blue-500/10 rounded text-blue-400"><Cpu size={12} /></div>
                      <div className="flex flex-col">
                         <span className="text-[8px] text-white/20 uppercase font-black">Runtime</span>
                         <span className="text-[10px] font-mono text-white/60 line-clamp-1">{response?.runtime || '---'}</span>
                      </div>
                   </div>
                </div>
             </div>
          </div>

          <div className="p-6 bg-blue-500/[0.03] border border-blue-500/10 rounded-2xl flex items-start gap-4">
             <div className="p-2 bg-blue-500/10 rounded-xl text-blue-400">
                <MessageSquare size={16} />
             </div>
             <div>
                <h4 className="text-[10px] font-black uppercase text-blue-400 tracking-widest mb-1">Arbitration Mode</h4>
                <p className="text-[10px] text-white/40 leading-relaxed">
                  Node acts as a Decision Gateway. If local warm-start cost &gt; remote latency, request is offloaded.
                </p>
             </div>
          </div>
        </div>
      </div>
    </div>
  );
}
