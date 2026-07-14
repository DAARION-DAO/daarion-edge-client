import { useEffect, useRef, useState } from "react";
import {
  AlertCircle,
  CheckCircle,
  Clock,
  Cpu,
  Loader2,
  MessageSquare,
  RefreshCw,
  Send,
  Sparkles,
  Square,
  TerminalSquare,
} from "lucide-react";
import {
  cancelLocalInference,
  cancelLocalModelPreparation,
  getLocalInferenceStatus,
  listInferenceModels,
  listenForInferenceEvents,
  prepareLocalModel,
  runLocalInference,
  type ChatMessage,
  InferenceClientError,
  type InferenceEvent,
  type InferenceModelSummary,
  type InferenceResponse,
  type InferenceStatus,
} from "../lib/inferenceClient";

type RuntimeState =
  | "checking"
  | "unavailable"
  | "model_missing"
  | "ready"
  | "preparing"
  | "running"
  | "cancelling"
  | "cancelled"
  | "timed_out"
  | "failed"
  | "completed_locally";

const stateLabels: Record<RuntimeState, string> = {
  checking: "Checking local provider",
  unavailable: "Local provider unavailable",
  model_missing: "Model not installed",
  ready: "Ready for local inference",
  preparing: "Preparing local model",
  running: "Running locally",
  cancelling: "Cancelling",
  cancelled: "Cancelled",
  timed_out: "Timed out",
  failed: "Failed",
  completed_locally: "Completed locally",
};

export function LocalInferencePanel() {
  const [models, setModels] = useState<InferenceModelSummary[]>([]);
  const [selectedModel, setSelectedModel] = useState("");
  const [providerStatus, setProviderStatus] = useState<InferenceStatus | null>(null);
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [prompt, setPrompt] = useState("");
  const [runtimeState, setRuntimeState] = useState<RuntimeState>("checking");
  const [response, setResponse] = useState<InferenceResponse | null>(null);
  const [streamingText, setStreamingText] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [isPreparing, setIsPreparing] = useState(false);
  const activeChatRequestId = useRef<string | null>(null);
  const activePreparationRequestId = useRef<string | null>(null);
  const runtimeStateRef = useRef<RuntimeState>("checking");
  const chatEndRef = useRef<HTMLDivElement>(null);

  function transition(next: RuntimeState) {
    runtimeStateRef.current = next;
    setRuntimeState(next);
  }

  async function refresh() {
    transition("checking");
    setError(null);
    try {
      const [status, registryModels] = await Promise.all([
        getLocalInferenceStatus(),
        listInferenceModels(),
      ]);
      setProviderStatus(status);
      setModels(registryModels);
      const preferred =
        registryModels.find((model) => model.installed)?.canonical_model_id ??
        registryModels[0]?.canonical_model_id ??
        "";
      setSelectedModel((current) =>
        registryModels.some((model) => model.canonical_model_id === current)
          ? current
          : preferred,
      );
      if (!status.available) {
        transition("unavailable");
      } else if (!registryModels.some((model) => model.installed)) {
        transition("model_missing");
      } else {
        transition("ready");
      }
    } catch (reason) {
      setError(String(reason));
      transition("failed");
    }
  }

  useEffect(() => {
    void refresh();
    const unlisten = listenForInferenceEvents(handleInferenceEvent);
    return () => {
      unlisten.then((stop) => stop());
    };
  }, []);

  useEffect(() => {
    chatEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, streamingText]);

  function handleInferenceEvent(event: InferenceEvent) {
    if (event.request_id !== activeChatRequestId.current) return;
    switch (event.kind) {
      case "running":
        transition("running");
        break;
      case "token":
        setStreamingText((current) => current + event.content);
        break;
      case "cancelled":
        transition("cancelled");
        setStreamingText("");
        break;
      case "timed_out":
        transition("timed_out");
        setStreamingText("");
        setError("The bounded local inference deadline was reached.");
        break;
      case "failed":
        transition("failed");
        setStreamingText("");
        setError(event.error);
        break;
      case "completed":
        transition("completed_locally");
        break;
      case "started":
        break;
    }
  }

  const selected = models.find((model) => model.canonical_model_id === selectedModel);
  const busy = isPreparing || runtimeState === "running" || runtimeState === "cancelling";

  async function handlePrepare() {
    if (!selectedModel || !providerStatus?.available) return;
    const requestId = crypto.randomUUID();
    activePreparationRequestId.current = requestId;
    setIsPreparing(true);
    transition("preparing");
    setError(null);
    try {
      await prepareLocalModel({
        request_id: requestId,
        canonical_model_id: selectedModel,
      });
      if (activePreparationRequestId.current === requestId) {
        await refresh();
        if (activePreparationRequestId.current === requestId) {
          transition("completed_locally");
        }
      }
    } catch (reason) {
      if (activePreparationRequestId.current === requestId) {
        if (reason instanceof InferenceClientError && reason.code === "cancelled") {
          setError("The local preparation request was cancelled. Ollama may retain resumable download progress.");
          transition("cancelled");
        } else if (reason instanceof InferenceClientError && reason.code === "timed_out") {
          setError("The bounded local model preparation deadline was reached.");
          transition("timed_out");
        } else {
          setError(reason instanceof Error ? reason.message : "Local model preparation failed.");
          transition("failed");
        }
      }
    } finally {
      if (activePreparationRequestId.current === requestId) {
        activePreparationRequestId.current = null;
        setIsPreparing(false);
      }
    }
  }

  async function handleSend() {
    if (!prompt.trim() || !selected?.installed || busy) return;
    const userMessage: ChatMessage = { role: "user", content: prompt.trim() };
    const history = [...messages, userMessage];
    const requestId = crypto.randomUUID();
    setMessages(history);
    setPrompt("");
    setError(null);
    setResponse(null);
    setStreamingText("");
    activeChatRequestId.current = requestId;
    transition("running");

    try {
      const result = await runLocalInference({
        request_id: requestId,
        canonical_model_id: selected.canonical_model_id,
        messages: history.slice(-10),
        max_tokens: 2048,
        temperature: 0.7,
        stream: true,
      });
      if (activeChatRequestId.current === requestId) {
        setResponse(result);
        setMessages((current) => [
          ...current,
          { role: "assistant", content: result.output_text },
        ]);
        setStreamingText("");
        transition("completed_locally");
      }
    } catch (reason) {
      if (
        activeChatRequestId.current === requestId &&
        runtimeStateRef.current !== "cancelled" &&
        runtimeStateRef.current !== "timed_out"
      ) {
        setError(String(reason));
        setStreamingText("");
        transition("failed");
      }
    } finally {
      if (activeChatRequestId.current === requestId) activeChatRequestId.current = null;
    }
  }

  async function handleCancelInference() {
    const requestId = activeChatRequestId.current;
    if (!requestId || runtimeState !== "running") return;
    transition("cancelling");
    try {
      const accepted = await cancelLocalInference(requestId);
      if (!accepted && activeChatRequestId.current === requestId) {
        setError("The request was no longer active.");
        transition("failed");
      }
    } catch (reason) {
      setError(String(reason));
      transition("failed");
    }
  }

  async function handleCancelPreparation() {
    const requestId = activePreparationRequestId.current;
    if (!requestId || !isPreparing || runtimeState === "cancelling") return;
    transition("cancelling");
    try {
      const accepted = await cancelLocalModelPreparation(requestId);
      if (!accepted && activePreparationRequestId.current === requestId) {
        setError("The preparation request was no longer registered; waiting for its terminal result.");
        transition("preparing");
      }
    } catch (reason) {
      if (activePreparationRequestId.current === requestId) {
        setError(reason instanceof Error ? reason.message : "Cancellation could not be requested; preparation may still be active.");
        transition("preparing");
      }
    }
  }

  return (
    <div className="mx-auto flex h-[85vh] max-w-5xl flex-col gap-6">
      <header className="flex shrink-0 items-center justify-between gap-4">
        <div>
          <h2 className="text-xl font-black tracking-tight text-white/90">Local Inference</h2>
          <p className="mt-1 text-[10px] font-black uppercase tracking-widest text-emerald-400">
            Local-only policy · no remote fallback
          </p>
        </div>
        <button onClick={() => void refresh()} disabled={busy} className="rounded-lg border border-white/10 p-2 text-white/50 hover:text-white disabled:opacity-30" aria-label="Refresh local inference status">
          <RefreshCw size={16} className={runtimeState === "checking" ? "animate-spin" : ""} />
        </button>
      </header>

      <div className="grid min-h-0 flex-1 grid-cols-1 gap-6 lg:grid-cols-12">
        <div className="glass flex flex-col overflow-hidden rounded-2xl border-white/5 lg:col-span-8">
          <div className="flex-1 space-y-6 overflow-y-auto p-6">
            {messages.length === 0 && !streamingText && (
              <div className="flex h-full flex-col items-center justify-center text-white/10">
                <TerminalSquare size={48} className="mb-4 opacity-20" />
                <p className="text-sm font-bold uppercase tracking-widest">Bounded local chat</p>
                <p className="mt-2 max-w-sm text-center text-[10px] uppercase tracking-widest text-white/25">
                  Requests are accepted only for canonical models mapped to the loopback Ollama provider.
                </p>
              </div>
            )}
            {messages.map((message, index) => (
              <div key={`${message.role}-${index}`} className={`flex ${message.role === "user" ? "justify-end" : "justify-start"}`}>
                <div className={`max-w-[85%] rounded-2xl border p-4 text-sm leading-relaxed ${message.role === "user" ? "rounded-br-sm border-blue-500/20 bg-blue-600/20 text-blue-100" : "rounded-bl-sm border-white/10 bg-white/5 text-white/90"}`}>
                  {message.role === "assistant" && <Sparkles size={12} className="mb-2 text-emerald-400" />}
                  <div className="whitespace-pre-wrap">{message.content}</div>
                </div>
              </div>
            ))}
            {streamingText && (
              <div className="flex justify-start">
                <div className="max-w-[85%] rounded-2xl rounded-bl-sm border border-white/10 bg-white/5 p-4 text-sm text-white/90">
                  <div className="whitespace-pre-wrap">{streamingText}<span className="ml-1 inline-block h-3 w-1.5 animate-pulse bg-emerald-400" /></div>
                </div>
              </div>
            )}
            {error && <div className="flex items-center justify-center gap-2 rounded-xl border border-red-500/20 bg-red-500/10 px-4 py-3 text-xs text-red-300"><AlertCircle size={14} />{error}</div>}
            <div ref={chatEndRef} />
          </div>

          <div className="shrink-0 border-t border-white/5 bg-white/[0.01] p-4">
            {providerStatus?.available && selected && !selected.installed && (
              <button onClick={() => void handlePrepare()} disabled={busy} className="mb-3 w-full rounded-xl border border-amber-500/20 bg-amber-500/10 px-4 py-3 text-xs font-bold text-amber-300 disabled:opacity-40">
                Install {selected.canonical_model_id} through local Ollama
              </button>
            )}
            <div className="flex items-end gap-2 rounded-xl border border-white/10 bg-black/40 px-4 py-3 focus-within:border-emerald-500/40">
              <textarea value={prompt} onChange={(event) => setPrompt(event.target.value)} onKeyDown={(event) => { if (event.key === "Enter" && !event.shiftKey) { event.preventDefault(); void handleSend(); } }} disabled={!selected?.installed || busy} placeholder={selected?.installed ? "Ask the local model…" : "Select and install a local model first"} className="max-h-[160px] min-h-[40px] w-full resize-none bg-transparent text-sm text-white outline-none placeholder:text-white/20 disabled:opacity-40" />
              {isPreparing ? (
                <button onClick={() => void handleCancelPreparation()} disabled={runtimeState === "cancelling"} className="shrink-0 rounded-lg bg-red-500/20 p-2 text-red-300 disabled:opacity-40" aria-label="Cancel local model preparation">
                  {runtimeState === "cancelling" ? <Loader2 size={18} className="animate-spin" /> : <Square size={18} />}
                </button>
              ) : busy ? (
                <button onClick={() => void handleCancelInference()} disabled={runtimeState === "cancelling"} className="shrink-0 rounded-lg bg-red-500/20 p-2 text-red-300 disabled:opacity-40" aria-label="Cancel local inference">
                  {runtimeState === "cancelling" ? <Loader2 size={18} className="animate-spin" /> : <Square size={18} />}
                </button>
              ) : (
                <button onClick={() => void handleSend()} disabled={!prompt.trim() || !selected?.installed} className="shrink-0 rounded-lg bg-emerald-500 p-2 text-black disabled:bg-white/5 disabled:text-white/20" aria-label="Run local inference"><Send size={18} /></button>
              )}
            </div>
          </div>
        </div>

        <aside className="space-y-4 overflow-y-auto lg:col-span-4">
          <div className="glass rounded-2xl border-white/5 p-5">
            <h3 className="mb-5 text-[10px] font-black uppercase tracking-widest text-white/25">Verified runtime state</h3>
            <div className="space-y-4 text-xs">
              <div className="flex items-center justify-between gap-3"><span className="text-white/40">State</span><span className="text-right font-bold text-emerald-300">{stateLabels[runtimeState]}</span></div>
              <div className="flex items-center justify-between gap-3"><span className="text-white/40">Provider</span><span className="font-mono text-white/70">{providerStatus?.provider_id ?? "—"}</span></div>
              <div className="flex items-center justify-between gap-3"><span className="text-white/40">Policy</span><span className="font-mono text-white/70">local_only</span></div>
              <div className="border-t border-white/5 pt-4">
                <select value={selectedModel} onChange={(event) => { setSelectedModel(event.target.value); const next = models.find((model) => model.canonical_model_id === event.target.value); if (providerStatus?.available) transition(next?.installed ? "ready" : "model_missing"); }} disabled={busy || models.length === 0} className="w-full rounded-lg border border-white/10 bg-black/30 px-3 py-2 text-[10px] text-white/70">
                  {models.map((model) => <option key={model.canonical_model_id} value={model.canonical_model_id}>{model.canonical_model_id}{model.installed ? " · installed" : " · missing"}</option>)}
                </select>
              </div>
              {response && <div className="grid grid-cols-2 gap-2 border-t border-white/5 pt-4"><div className="rounded-xl border border-white/5 p-3"><Clock size={11} className="mb-1 text-white/30" /><span className="font-mono text-white/70">{response.latency_ms} ms</span></div><div className="rounded-xl border border-white/5 p-3"><Cpu size={11} className="mb-1 text-white/30" /><span className="font-mono text-white/70">{response.provider_id}</span></div></div>}
            </div>
          </div>
          <div className="rounded-2xl border border-emerald-500/10 bg-emerald-500/[0.03] p-5">
            <div className="mb-3 flex items-center gap-3 text-emerald-400"><MessageSquare size={16} /><h4 className="text-[10px] font-black uppercase tracking-widest">Local-only boundary</h4></div>
            <p className="text-[11px] leading-relaxed text-white/45">Inference traffic is restricted to an HTTP loopback origin. Redirects, system proxies, remote endpoints, and silent fallback are disabled.</p>
            {runtimeState === "completed_locally" && <div className="mt-3 flex items-center gap-2 text-[10px] text-emerald-300"><CheckCircle size={13} />Completed by the local provider</div>}
          </div>
        </aside>
      </div>
    </div>
  );
}
