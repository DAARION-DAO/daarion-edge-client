import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Send, Loader2, Sparkles, Clock, Cpu, MessageSquare, AlertCircle, TerminalSquare } from "lucide-react";

interface ChatMessage {
  role: string;
  content: string;
}

interface LocalInferenceRequest {
  request_id: string;
  model_id: string;
  messages: ChatMessage[];
  max_tokens: number;
  temperature: number;
  stream: boolean;
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
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [prompt, setPrompt] = useState("");
  const [sessionState, setSessionState] = useState<"Idle" | "Queued" | "LoadingModel" | "Running" | "RoutingToNetwork" | "Done" | "Failed">("Idle");
  const [response, setResponse] = useState<LocalInferenceResponse | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [streamingText, setStreamingText] = useState("");
  
  const chatEndRef = useRef<HTMLDivElement>(null);

  // Auto-scroll chat
  useEffect(() => {
    chatEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, streamingText]);

  useEffect(() => {
    async function fetchModels() {
      try {
        const res = await invoke<string[]>("list_local_models");
        setModels(res.map(t => ({ model_id: t, name: t })));
        if (res.length > 0) setSelectedModel(res[0]);
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
          const finalResult = event.payload.result as LocalInferenceResponse;
          setResponse(finalResult);
          setIsLoading(false);
          setMessages(prev => [...prev, { role: "assistant", content: finalResult.output_text }]);
          setStreamingText(""); // Clear streaming state
        } else if (event.payload.state === "Failed") {
            setError(event.payload.reason || "Inference failed");
            setIsLoading(false);
            setStreamingText("");
        }
      }
    );

    // Used if returning tokens explicitly via another event.
    // In our backend we emit direct tokens as "inference-token-stream"
    const unlistenTokenStream = listen<{ request_id: string, token: string }>(
        "inference-token-stream",
        (event) => {
            setStreamingText(prev => prev + event.payload.token);
        }
    );

    return () => {
      unlistenStatus.then(f => f());
      unlistenTokenStream.then(f => f());
    };
  }, []);

  async function handleSend() {
    if (!prompt.trim() || !selectedModel) return;

    const userMsg = { role: "user", content: prompt };
    const newHistory = [...messages, userMsg];
    setMessages(newHistory);
    setPrompt("");

    setError(null);
    setResponse(null);
    setStreamingText("");
    setSessionState("Queued");
    setIsLoading(true);

    // Context budget: Last 10 messages max
    const trimmedMessages = newHistory.slice(-10);

    const request: LocalInferenceRequest = {
      request_id: crypto.randomUUID(),
      model_id: selectedModel,
      messages: trimmedMessages,
      max_tokens: 2048,
      temperature: 0.7,
      stream: true
    };

    try {
      await invoke("run_chat", { request });
    } catch (e) {
      setError(String(e));
      setIsLoading(false);
      setSessionState("Idle");
    }
  }

  return (
    <div className="space-y-6 max-w-5xl mx-auto h-[85vh] flex flex-col">
      <header className="flex justify-between items-center shrink-0">
        <div>
          <h2 className="text-xl font-black tracking-tight text-white/90">Agent Orchestration Shell</h2>
          <p className="text-[10px] text-white/30 uppercase font-black tracking-widest mt-1 text-emerald-400">Local Tier 2 Runtime Active</p>
        </div>
        {models.length > 0 && (
            <div className="flex items-center gap-3">
              <span className="text-[9px] uppercase font-black text-white/20 tracking-widest">Active Model</span>
              <select 
                  value={selectedModel}
                  onChange={(e) => setSelectedModel(e.target.value)}
                  className="bg-emerald-500/10 border border-emerald-500/30 text-emerald-400 rounded-lg px-3 py-1.5 text-[10px] font-bold focus:border-emerald-500 outline-none uppercase"
               >
                  {models.map(m => <option key={m.model_id} value={m.model_id}>{m.name}</option>)}
               </select>
            </div>
        )}
      </header>

      <div className="grid grid-cols-1 lg:grid-cols-12 gap-6 flex-1 min-h-0">
        
        {/* Main Chat Interface (8 Cols) */}
        <div className="lg:col-span-8 flex flex-col glass rounded-2xl border-white/5 overflow-hidden">
          {/* Chat History */}
          <div className="flex-1 overflow-y-auto p-6 space-y-6">
             {messages.length === 0 && !streamingText && (
                <div className="h-full flex flex-col items-center justify-center text-white/10">
                   <TerminalSquare size={48} className="mb-4 opacity-20" />
                   <p className="text-sm font-bold uppercase tracking-widest mb-1">Local Inference Shell</p>
                   <p className="text-[10px] uppercase font-black tracking-widest text-emerald-400/30 text-center max-w-xs">Zero-latency private execution via {selectedModel || 'local engine'}</p>
                </div>
             )}
             
             {messages.map((m, i) => (
                <div key={i} className={`flex ${m.role === 'user' ? 'justify-end' : 'justify-start'}`}>
                   <div className={`max-w-[85%] p-4 rounded-2xl ${
                      m.role === 'user' 
                      ? 'bg-blue-600/20 border border-blue-500/20 text-blue-100 rounded-br-sm' 
                      : 'bg-white/5 border border-white/10 text-white/90 rounded-bl-sm font-light'
                   }`}>
                      {m.role === 'assistant' && (
                         <div className="flex items-center gap-2 mb-2 text-emerald-400">
                            <Sparkles size={12} />
                            <span className="text-[8px] font-black uppercase tracking-widest">DAARION Edge</span>
                         </div>
                      )}
                      <div className="whitespace-pre-wrap text-sm leading-relaxed">{m.content}</div>
                   </div>
                </div>
             ))}

             {streamingText && (
                <div className="flex justify-start">
                   <div className="max-w-[85%] p-4 rounded-2xl bg-white/5 border border-white/10 text-white/90 rounded-bl-sm font-light">
                      <div className="flex items-center gap-2 mb-2 text-emerald-400 animate-pulse">
                         <Sparkles size={12} />
                         <span className="text-[8px] font-black uppercase tracking-widest">Generating Locally...</span>
                      </div>
                      <div className="whitespace-pre-wrap text-sm leading-relaxed">
                         {streamingText}<span className="inline-block w-1.5 h-3 ml-1 bg-emerald-400 animate-pulse" />
                      </div>
                   </div>
                </div>
             )}

             {error && (
                <div className="flex justify-center my-4">
                   <div className="px-4 py-2 bg-red-500/10 border border-red-500/20 text-red-400 rounded-full text-xs font-bold flex items-center gap-2">
                      <AlertCircle size={14} /> {error}
                   </div>
                </div>
             )}
             
             <div ref={chatEndRef} />
          </div>

          {/* Input Box */}
          <div className="p-4 bg-white/[0.01] border-t border-white/5 shrink-0">
             <div className="relative flex items-end gap-2 bg-black/40 border border-white/10 rounded-xl px-4 py-3 focus-within:border-emerald-500/40 focus-within:ring-1 focus-within:ring-emerald-500/20 transition-all">
               <textarea 
                 value={prompt}
                 onChange={(e) => setPrompt(e.target.value)}
                 onKeyDown={(e) => {
                    if (e.key === 'Enter' && !e.shiftKey) {
                       e.preventDefault();
                       handleSend();
                    }
                 }}
                 placeholder="Communicate directly with the local DAARION edge runtime..."
                 className="w-full bg-transparent text-sm focus:outline-none min-h-[40px] max-h-[160px] resize-none placeholder:text-white/20 text-white leading-relaxed"
               />
               <button 
                 onClick={handleSend}
                 disabled={isLoading || !prompt.trim() || !selectedModel}
                 className={`p-2 rounded-lg transition-all shrink-0 ${
                   isLoading || !prompt.trim() || !selectedModel
                   ? 'bg-white/5 text-white/20' 
                   : 'bg-emerald-500 text-black hover:bg-emerald-400 shadow-lg shadow-emerald-900/20'
                 }`}
               >
                 {isLoading ? <Loader2 size={18} className="animate-spin text-emerald-500" /> : <Send size={18} />}
               </button>
             </div>
             <p className="text-center mt-2 text-[9px] font-bold text-white/20 uppercase tracking-widest hidden md:block">
                Shift + Enter for new line • Local execution ensures complete data sovereignty
             </p>
          </div>
        </div>

        {/* Right: Telemetry (4 cols) */}
        <div className="lg:col-span-4 space-y-4 overflow-y-auto">
          <div className="glass p-5 border-white/5 rounded-2xl">
             <h3 className="text-[10px] text-white/20 font-black uppercase tracking-widest mb-5">Inference Telemetry</h3>
             
             <div className="space-y-5">
                <div className="flex justify-between items-center">
                  <span className="text-[10px] font-bold text-white/40 uppercase">State</span>
                  <div className="flex items-center gap-2">
                    {sessionState !== "Idle" && <div className={`w-1.5 h-1.5 rounded-full ${sessionState === 'Done' ? 'bg-emerald-500' : 'bg-emerald-400 animate-pulse'}`} />}
                    <span className={`text-[9px] font-black uppercase tracking-widest ${sessionState === 'Done' ? 'text-emerald-400' : sessionState === 'Idle' ? 'text-white/20' : 'text-emerald-300'}`}>{sessionState}</span>
                  </div>
                </div>

                <div className="space-y-1.5">
                   <div className="flex justify-between text-[9px] font-bold text-white/40 uppercase mb-1 tracking-widest">
                      <span>Pipeline Progress</span>
                      <span className="text-white/20">
                         {sessionState === "Idle" ? "0%" : sessionState === "Queued" ? "10%" : (sessionState === "LoadingModel") ? "40%" : sessionState === "Running" ? "80%" : "100%"}
                      </span>
                   </div>
                   <div className="h-1 bg-white/5 rounded-full overflow-hidden">
                      <div className={`h-full bg-emerald-500 transition-all duration-700 ${
                        sessionState === "Idle" ? "w-0" : 
                        sessionState === "Queued" ? "w-[10%]" : 
                        (sessionState === "LoadingModel") ? "w-[40%]" : 
                        sessionState === "Running" ? "w-[80%]" : "w-full"
                      }`} />
                   </div>
                </div>

                <div className="grid grid-cols-2 gap-2 pt-4 border-t border-white/5">
                   <div className="p-3 rounded-xl bg-white/[0.02] border border-white/5 flex flex-col justify-center">
                      <div className="flex items-center gap-1 mb-1">
                         <Clock size={10} className="text-white/30" />
                         <span className="text-[8px] text-white/30 uppercase font-black tracking-widest">Latency</span>
                      </div>
                      <span className="text-xs font-mono font-bold text-white/70">{response?.latency_ms || '---'} <span className="text-[9px] text-white/30">ms</span></span>
                   </div>
                   <div className="p-3 rounded-xl bg-white/[0.02] border border-white/5 flex flex-col justify-center">
                      <div className="flex items-center gap-1 mb-1">
                         <Cpu size={10} className="text-white/30" />
                         <span className="text-[8px] text-white/30 uppercase font-black tracking-widest">Runtime</span>
                      </div>
                      <span className="text-[10px] font-mono font-bold text-white/70 line-clamp-1">{response?.runtime || '---'}</span>
                   </div>
                </div>
             </div>
          </div>

          <div className="p-5 bg-emerald-500/[0.03] border border-emerald-500/10 rounded-2xl flex flex-col gap-3">
             <div className="flex items-center gap-3">
                <div className="p-2 bg-emerald-500/10 rounded-lg text-emerald-400">
                   <MessageSquare size={16} />
                </div>
                <h4 className="text-[10px] font-black uppercase text-emerald-400 tracking-widest">Privacy Guarantee</h4>
             </div>
             <p className="text-[11px] text-white/40 leading-relaxed font-medium">
               Active interaction is executed 100% locally through the Host APU/GPU. Zero network transmission. Native context window trimming limits absolute history to recent turns.
             </p>
          </div>
        </div>
      </div>
    </div>
  );
}
