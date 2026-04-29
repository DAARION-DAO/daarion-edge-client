import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  CheckCircle2, Zap, Monitor
} from "lucide-react";

interface StepStatus {
  step: string;
  status: string;
  message: string;
}

interface WorkerModeState {
  opted_in: boolean;
  runtime_status: { state: string; reason?: string };
}

export function EdgeActivation() {
  // workerModeEnabled removed to fix TS error
  const [runtimeStatus, setRuntimeStatus] = useState<string>("Unknown");
  const [currentStep, setCurrentStep] = useState<number>(0);
  const [onboardingLog, setOnboardingLog] = useState<string[]>([]);
  const [isProcessing, setIsProcessing] = useState(false);
  const [stepFailures, setStepFailures] = useState<Record<number, string>>({});

  useEffect(() => {
    async function checkWorkerMode() {
      try {
        const status = await invoke<WorkerModeState>("get_worker_mode");
        // setWorkerModeEnabled(status.opted_in);
        setRuntimeStatus(status.runtime_status?.state || "Unknown");
        if (status.opted_in) setCurrentStep(4); // already enabled
      } catch (e) {
        console.error("Failed to fetch worker mode", e);
      }
    }
    checkWorkerMode();
  }, []);

  const addLog = (msg: string) => setOnboardingLog(prev => [...prev, msg]);

  async function handleOnboardingStart() {
    setCurrentStep(1);
    addLog("Initializing bounded Worker Mode setup...");
  }

  async function handleEnvironmentCheck() {
    setIsProcessing(true);
    setStepFailures(prev => ({ ...prev, 1: "" }));
    addLog("Checking environment readiness natively...");
    
    try {
      const res = await invoke<StepStatus>("check_environment");
      addLog(res.message);
      if (res.status === "passed") {
        setCurrentStep(2);
      } else if (res.status === "failed_wsl_missing") {
        setStepFailures(prev => ({ ...prev, 1: "wsl_missing" }));
        addLog("[ACTION REQUIRED] WSL2 is not enabled. Please enable it to proceed.");
      } else {
        setStepFailures(prev => ({ ...prev, 1: res.message }));
        addLog("[FAIL] Environment check aborted.");
      }
    } catch (e) {
      addLog(`[ERROR] Command failed: ${e}`);
    } finally {
      setIsProcessing(false);
    }
  }

  async function handleEnableWSL() {
    setIsProcessing(true);
    addLog("Requesting elevated privileges to enable WSL2...");
    try {
      const msg = await invoke<string>("enable_wsl_windows");
      addLog(msg);
      addLog("Please reboot your machine if prompted by Windows, then restart this application.");
    } catch (e) {
      addLog(`[ERROR] Failed to enable WSL2: ${e}`);
    } finally {
      setIsProcessing(false);
    }
  }

  async function handleOperatorApproval() {
    setIsProcessing(true);
    setStepFailures(prev => ({ ...prev, 2: "" }));
    addLog("Verifying worker identity for eligibility...");
    
    try {
      const res = await invoke<StepStatus>("check_operator_approval");
      addLog(res.message);
      if (res.status === "passed") {
        setCurrentStep(3);
      } else {
        setStepFailures(prev => ({ ...prev, 2: res.message }));
        addLog("[FAIL] Eligibility check failed. Worker Mode requires approval.");
      }
    } catch (e) {
      addLog(`[ERROR] Command failed: ${e}`);
    } finally {
      setIsProcessing(false);
    }
  }

  async function handleConnectAndEnable() {
    setIsProcessing(true);
    setStepFailures(prev => ({ ...prev, 3: "" }));
    addLog("Establishing secure network connection...");
    
    try {
      const octRes = await invoke<StepStatus>("activate_octelium_tunnel");
      addLog(octRes.message);
      
      if (octRes.status === "passed") {
        await invoke("toggle_worker_mode", { enabled: true });
        // setWorkerModeEnabled(true);
        addLog("Worker daemon successfully activated in Advisory Mode.");
        setCurrentStep(4);
      } else {
        setStepFailures(prev => ({ ...prev, 3: octRes.message }));
      }
    } catch (e) {
      addLog(`[ERROR] Failed to enable worker: ${e}`);
      setStepFailures(prev => ({ ...prev, 3: "Rust execution error." }));
    } finally {
      setIsProcessing(false);
    }
  }

  async function handleDisableWorker() {
    try {
      await invoke("toggle_worker_mode", { enabled: false });
      // setWorkerModeEnabled(false);
      setCurrentStep(0);
      setOnboardingLog([]);
    } catch (e) {
      console.error(e);
    }
  }

  return (
    <div className="h-full bg-[#050505] p-6 lg:p-10 font-sans text-white overflow-y-auto">
      <div className="max-w-3xl mx-auto space-y-8">
        
        <header className="space-y-4 border-b border-white/5 pb-8">
          <div className="inline-flex items-center gap-2 px-3 py-1 bg-emerald-500/10 border border-emerald-500/20 rounded-full text-[10px] font-black uppercase tracking-widest text-emerald-400">
            <Monitor size={12} />
            Worker Mode Setup
          </div>
          <h1 className="text-3xl font-black tracking-tight text-white/90">
            Advisory Worker Onboarding
          </h1>
          <p className="text-sm text-white/50 leading-relaxed max-w-2xl">
            Welcome. By enabling Worker Mode, your device may contribute to the DAARION network in a strictly bounded, advisory capacity. Contributions are optional and do not grant consensus authority or network sovereignty.
          </p>
        </header>

        {currentStep === 0 && (
          <div className="glass p-8 rounded-3xl border border-white/5 flex flex-col items-center justify-center text-center space-y-6 bg-white/[0.02]">
            <div className="w-20 h-20 bg-emerald-500/10 rounded-full flex items-center justify-center border border-emerald-500/20">
              <Zap size={32} className="text-emerald-400" />
            </div>
            <div className="space-y-2">
              <h2 className="text-xl font-bold text-white/90">Enable bounded contribution?</h2>
              <p className="text-xs text-white/50 max-w-md mx-auto leading-relaxed">
                Follow the in-app wizard to verify eligibility and enable optional advisory tasks. This mode is bounded and can be disabled at any time.
              </p>
            </div>
            <button 
              onClick={handleOnboardingStart}
              className="px-8 py-4 bg-emerald-600 hover:bg-emerald-500 rounded-xl text-xs font-black uppercase tracking-widest text-white transition-all shadow-[0_0_20px_rgba(16,185,129,0.3)]"
            >
              Start Onboarding
            </button>
          </div>
        )}

        {currentStep > 0 && currentStep < 4 && (
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            {/* Steps Sidebar */}
            <div className="space-y-4 col-span-1">
              <div className={`p-4 rounded-xl border ${currentStep >= 1 ? 'border-emerald-500/30 bg-emerald-500/10' : 'border-white/5 opacity-50'} ${stepFailures[1] ? 'border-red-500/50 bg-red-500/10' : ''}`}>
                <p className="text-xs font-bold text-white/80">Step 1: Environment Check</p>
                {stepFailures[1] && stepFailures[1] !== "wsl_missing" && <p className="text-[10px] text-red-400 mt-1">{stepFailures[1]}</p>}
                {stepFailures[1] === "wsl_missing" && <p className="text-[10px] text-amber-400 mt-1">WSL2 is required on Windows.</p>}
              </div>
              <div className={`p-4 rounded-xl border ${currentStep >= 2 ? 'border-emerald-500/30 bg-emerald-500/10' : 'border-white/5 opacity-50'} ${stepFailures[2] ? 'border-red-500/50 bg-red-500/10' : ''}`}>
                <p className="text-xs font-bold text-white/80">Step 2: Eligibility Check</p>
                {stepFailures[2] && <p className="text-[10px] text-red-400 mt-1">{stepFailures[2]}</p>}
              </div>
              <div className={`p-4 rounded-xl border ${currentStep >= 3 ? 'border-emerald-500/30 bg-emerald-500/10' : 'border-white/5 opacity-50'} ${stepFailures[3] ? 'border-red-500/50 bg-red-500/10' : ''}`}>
                <p className="text-xs font-bold text-white/80">Step 3: Secure Connection</p>
                {stepFailures[3] && <p className="text-[10px] text-red-400 mt-1">{stepFailures[3]}</p>}
              </div>
            </div>

            {/* Action Area */}
            <div className="col-span-1 md:col-span-2 glass p-6 rounded-3xl border border-white/5 space-y-6 bg-white/[0.02]">
              <div className="h-48 bg-[#0a0a0a] rounded-xl p-4 font-mono text-[10px] text-emerald-400/80 overflow-y-auto border border-white/5">
                {onboardingLog.map((log, idx) => (
                  <div key={idx} className="mb-1">{`> ${log}`}</div>
                ))}
                {isProcessing && <div className="animate-pulse text-white/40 mt-2">Processing...</div>}
              </div>

              <div className="flex justify-end pt-4 border-t border-white/5 gap-3">
                {currentStep === 1 && stepFailures[1] === "wsl_missing" && (
                  <button onClick={handleEnableWSL} disabled={isProcessing} className="px-6 py-3 bg-blue-600 hover:bg-blue-500 rounded-lg text-xs font-bold transition-colors">
                    Enable WSL2 (Admin Required)
                  </button>
                )}
                {currentStep === 1 && (
                  <button onClick={handleEnvironmentCheck} disabled={isProcessing} className="px-6 py-3 bg-white/10 hover:bg-white/20 rounded-lg text-xs font-bold transition-colors">
                    Verify Environment
                  </button>
                )}
                {currentStep === 2 && (
                  <button onClick={handleOperatorApproval} disabled={isProcessing} className="px-6 py-3 bg-white/10 hover:bg-white/20 rounded-lg text-xs font-bold transition-colors">
                    Check Operator Approval
                  </button>
                )}
                {currentStep === 3 && (
                  <button onClick={handleConnectAndEnable} disabled={isProcessing} className="px-6 py-3 bg-emerald-600 hover:bg-emerald-500 rounded-lg text-xs font-bold shadow-[0_0_15px_rgba(16,185,129,0.3)] transition-colors">
                    Activate Secure Connection
                  </button>
                )}
              </div>
            </div>
          </div>
        )}

        {currentStep === 4 && (
          <div className="space-y-6">
            <div className={`p-10 rounded-3xl ${runtimeStatus === "Active" ? "bg-emerald-500/10 border-emerald-500/20" : "bg-amber-500/10 border-amber-500/20"} border flex flex-col items-center justify-center text-center space-y-4 relative overflow-hidden`}>
              <div className={`absolute top-0 right-0 p-32 ${runtimeStatus === "Active" ? "bg-emerald-500/10" : "bg-amber-500/10"} blur-[100px] rounded-full pointer-events-none`} />
              <CheckCircle2 size={48} className={runtimeStatus === "Active" ? "text-emerald-400" : "text-amber-400"} />
              <div>
                <h2 className={`text-2xl font-black ${runtimeStatus === "Active" ? "text-emerald-400" : "text-amber-400"}`}>
                  {runtimeStatus === "Active" ? "Worker Active" : "Worker Opted In"}
                </h2>
                <p className={`text-sm mt-2 max-w-md mx-auto ${runtimeStatus === "Active" ? "text-emerald-400/70" : "text-amber-400/70"}`}>
                  {runtimeStatus === "Active" 
                    ? "Bounded advisory worker is active. Tasks are processed locally in a sandboxed environment."
                    : runtimeStatus === "Unavailable" || runtimeStatus === "Blocked"
                      ? "Worker Mode is enabled but cannot connect to the network yet. This is expected during early access."
                      : "Worker Mode is enabled. Connection status is being determined."}
                </p>
              </div>
              <button 
                onClick={handleDisableWorker}
                className="mt-8 px-6 py-3 bg-red-500/10 hover:bg-red-500/20 border border-red-500/30 rounded-lg text-xs font-bold text-red-400 transition-colors"
              >
                Disconnect & Halt
              </button>
            </div>
          </div>
        )}

      </div>
    </div>
  );
}
