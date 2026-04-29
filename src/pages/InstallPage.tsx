import { useState, useEffect } from "react";
import {
  Smartphone, Share, PlusSquare,
  CheckCircle, Download, Wifi, Shield, Zap, Globe
} from "lucide-react";

type Platform = "ios" | "android" | "desktop" | "unknown";

function detectPlatform(): Platform {
  const ua = navigator.userAgent;
  const isIOS = /iPad|iPhone|iPod/.test(ua) && !(window as any).MSStream;
  if (isIOS) return "ios";
  if (/android/i.test(ua)) return "android";
  if (typeof window !== "undefined") return "desktop";
  return "unknown";
}

function useIsInstalled(): boolean {
  const [installed, setInstalled] = useState(false);
  useEffect(() => {
    if (window.matchMedia("(display-mode: standalone)").matches) {
      setInstalled(true);
    }
  }, []);
  return installed;
}

function Step({ num, icon: Icon, title, desc }: { num: number; icon: any; title: string; desc: string }) {
  return (
    <div className="flex gap-4 items-start">
      <div className="flex-shrink-0 w-8 h-8 rounded-full bg-blue-600/20 border border-blue-500/30 flex items-center justify-center text-[10px] font-black text-blue-400">
        {num}
      </div>
      <div className="flex gap-3 items-start">
        <Icon size={18} className="text-white/30 flex-shrink-0 mt-0.5" />
        <div>
          <p className="text-sm font-bold text-white/80">{title}</p>
          <p className="text-[11px] text-white/40 leading-relaxed mt-0.5">{desc}</p>
        </div>
      </div>
    </div>
  );
}

function Feature({ icon: Icon, label }: { icon: any; label: string }) {
  return (
    <div className="flex items-center gap-2 px-3 py-2 rounded-xl bg-white/[0.03] border border-white/5">
      <Icon size={12} className="text-blue-400/70" />
      <span className="text-[10px] text-white/40 font-medium">{label}</span>
    </div>
  );
}

export function InstallPage() {
  const isInstalled = useIsInstalled();

  useEffect(() => {
    detectPlatform(); // just to run it, but we don't need the state here anymore since UI unified
  }, []);

  return (
    <div className="min-h-screen bg-[#020202] text-white font-sans flex flex-col items-center justify-center p-6 lg:p-12 relative overflow-y-auto">
      
      {/* Background glow */}
      <div className="absolute inset-0 pointer-events-none fixed">
        <div className="absolute top-1/3 left-1/4 -translate-x-1/2 -translate-y-1/2 w-[400px] h-[400px] rounded-full bg-blue-900/10 blur-[120px] animate-pulse" />
        <div className="absolute top-2/3 right-1/4 translate-x-1/2 -translate-y-1/2 w-[400px] h-[400px] rounded-full bg-emerald-900/10 blur-[120px]" />
      </div>

      <div className="w-full max-w-5xl z-10 space-y-10">
        
        {/* Unified Header */}
        <div className="text-center space-y-4">
          <img src="/icon-512.png" alt="DAARION Edge" className="w-20 h-20 rounded-3xl mx-auto shadow-[0_0_40px_rgba(255,255,255,0.05)]" />
          <div>
            <p className="text-[10px] uppercase tracking-[0.4em] text-white/30 mb-2 font-bold">DAARION Protocol</p>
            <h1 className="text-4xl md:text-5xl font-black tracking-tighter bg-gradient-to-br from-white via-white/90 to-white/40 bg-clip-text text-transparent">
              Unified Access
            </h1>
          </div>
          <p className="text-sm text-white/40 max-w-lg mx-auto leading-relaxed">
            Choose how you want to interact with the DAARION network. Join as a standard client or contribute as an advisory worker.
          </p>
        </div>

        {/* Two-Column Layout */}
        <div className="grid grid-cols-1 md:grid-cols-2 gap-8 items-start">
          
          {/* LEFT: PWA Client App */}
          <div className="glass p-8 rounded-3xl border border-blue-500/10 bg-[#0a0a0a]/60 backdrop-blur-xl relative overflow-hidden space-y-8">
            <div className="absolute top-0 inset-x-0 h-1 bg-gradient-to-r from-blue-500 to-cyan-400 opacity-50" />
            
            <div className="text-center space-y-2">
              <h2 className="text-2xl font-black tracking-tighter">
                EDGE<span className="text-blue-500 font-light ml-1">Client</span>
              </h2>
              <p className="text-[10px] text-blue-400 uppercase tracking-widest font-bold">Standard User Access</p>
            </div>

            <div className="text-xs text-white/50 leading-relaxed text-center">
              Install the client directly to your device. No App Store. Sovereign chat, models, and network telemetry.
            </div>

            <div className="grid grid-cols-2 gap-2">
              <Feature icon={Wifi} label="Offline-ready" />
              <Feature icon={Shield} label="Sovereign UX" />
              <Feature icon={Zap} label="Native Feel" />
              <Feature icon={Globe} label="No App Store" />
            </div>

            <div className="space-y-6 pt-4 border-t border-white/5">
              <div className="flex items-center gap-3">
                <Smartphone className="text-blue-400" size={20} />
                <p className="text-sm font-bold text-white/90">Web App Installation</p>
              </div>
              
              {!isInstalled ? (
                <div className="space-y-5">
                  <Step num={1} icon={Globe} title="Open in Browser" desc="Use Safari (iOS), Chrome (Android), or Edge/Brave (Desktop)." />
                  <Step num={2} icon={Share} title="Share / Options" desc="Tap Share on iOS, or the browser menu (⋮) on Android/Desktop." />
                  <Step num={3} icon={PlusSquare} title="Add to Home Screen" desc="Select 'Add to Home Screen' or 'Install App'." />
                </div>
              ) : (
                <div className="p-4 rounded-xl bg-emerald-500/10 border border-emerald-500/20 flex items-center gap-3">
                  <CheckCircle className="text-emerald-400 flex-shrink-0" />
                  <div>
                    <p className="text-xs font-bold text-emerald-400">Successfully Installed</p>
                    <p className="text-[10px] text-emerald-400/60 mt-0.5">Launch DAARION from your home screen.</p>
                  </div>
                </div>
              )}
            </div>

            <a href="/" className="flex items-center justify-center gap-2 w-full py-4 rounded-xl bg-blue-600/10 hover:bg-blue-600/20 border border-blue-500/30 text-blue-400 font-black uppercase tracking-widest text-[11px] transition-colors">
              <Zap size={14} /> {isInstalled ? "Open App" : "Continue in Browser"}
            </a>
          </div>

          {/* RIGHT: Desktop Worker */}
          <div className="glass p-8 rounded-3xl border border-emerald-500/10 bg-[#0a0a0a]/60 backdrop-blur-xl relative overflow-hidden space-y-8">
            <div className="absolute top-0 inset-x-0 h-1 bg-gradient-to-r from-emerald-500 to-teal-400 opacity-50" />
            
            <div className="text-center space-y-2">
              <h2 className="text-2xl font-black tracking-tighter">
                EDGE<span className="text-emerald-500 font-light ml-1">Worker</span>
              </h2>
              <p className="text-[10px] text-emerald-400 uppercase tracking-widest font-bold">Advisory Mode</p>
            </div>

            <div className="text-xs text-emerald-100/50 leading-relaxed bg-emerald-500/5 p-4 rounded-xl border border-emerald-500/10">
              <p className="mb-2"><strong className="text-emerald-400">Join the DAARION Network</strong> by donating spare compute power.</p>
              <p className="text-[10px]">The Edge Worker runs in a strictly bounded sandbox. It acts in an advisory capacity only and does not control consensus state.</p>
            </div>

            <div className="space-y-4 pt-4 border-t border-white/5">
              
              {/* macOS Active */}
              <div className="p-5 rounded-2xl bg-white/[0.02] border border-white/10 relative overflow-hidden group hover:border-emerald-500/30 transition-colors">
                <div className="absolute inset-y-0 left-0 w-1 bg-emerald-500" />
                <div className="flex flex-col gap-4 ml-2">
                  <div>
                    <p className="text-sm font-bold text-white/90">macOS (Apple Silicon / Intel)</p>
                    <p className="text-[11px] text-white/40 mt-1">Native DMG with in-app setup. No terminal required.</p>
                  </div>
                  <a 
                    href="https://github.com/DAARION-DAO/daarion-edge-client/releases" 
                    target="_blank"
                    rel="noopener noreferrer"
                    className="inline-flex items-center justify-center gap-2 px-4 py-3 bg-emerald-600 hover:bg-emerald-500 rounded-xl text-[11px] font-black uppercase tracking-widest text-white shadow-[0_0_15px_rgba(16,185,129,0.3)] transition-all"
                  >
                    <Download size={14} /> View Releases
                  </a>
                  <p className="text-[10px] text-emerald-400/50 mt-1 italic">
                    Manual updates only via GitHub Releases
                  </p>
                </div>
              </div>

              {/* Windows Preview */}
              <div className="p-5 rounded-2xl bg-white/[0.02] border border-white/10 relative overflow-hidden group hover:border-blue-500/30 transition-colors">
                <div className="absolute inset-y-0 left-0 w-1 bg-blue-500" />
                <div className="flex flex-col gap-4 ml-2">
                  <div>
                    <div className="flex items-center gap-2 mb-1">
                      <p className="text-sm font-bold text-white/90">Windows 10 / 11</p>
                      <span className="px-1.5 py-0.5 rounded text-[8px] font-black uppercase bg-amber-500/20 text-amber-400 tracking-wider">Preview</span>
                    </div>
                    <p className="text-[11px] text-white/40 mt-1">Build verified in CI. Install &amp; execution paths are being validated by preview testers.</p>
                  </div>
                  <a 
                    href="https://github.com/DAARION-DAO/daarion-edge-client/releases" 
                    target="_blank"
                    rel="noopener noreferrer"
                    className="inline-flex items-center justify-center gap-2 px-4 py-3 bg-blue-600/20 hover:bg-blue-600/30 border border-blue-500/30 rounded-xl text-[11px] font-black uppercase tracking-widest text-blue-400 transition-all"
                  >
                    <Download size={14} /> Preview Download
                  </a>
                  <p className="text-[10px] text-blue-400/40 mt-1 italic">
                    Execution unvalidated · Manual updates only
                  </p>
                </div>
              </div>

            </div>

          </div>

        </div>
        
        <p className="text-center text-[10px] text-white/20 font-mono">
          DAARION Edge v0.2.0-beta
        </p>

      </div>
    </div>
  );
}
