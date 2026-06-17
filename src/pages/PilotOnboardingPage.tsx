import { Shield, Monitor, HardDrive, Info, Download } from "lucide-react";

export function PilotOnboardingPage() {
  return (
    <div className="min-h-screen bg-[#020202] text-white font-sans p-6 md:p-12 overflow-x-hidden">
      <div className="max-w-3xl mx-auto space-y-12">
        
        {/* Header */}
        <header className="space-y-4 border-b border-white/10 pb-8">
          <div className="inline-flex items-center gap-2 px-3 py-1 bg-emerald-500/10 border border-emerald-500/20 rounded-full text-[10px] font-black uppercase tracking-widest text-emerald-400">
            <Shield size={12} />
            Internal Pilot Only
          </div>
          <h1 className="text-3xl md:text-4xl font-black tracking-tight text-white/90">
            Desktop Worker Onboarding
          </h1>
          <p className="text-sm md:text-base text-white/50 leading-relaxed max-w-xl">
            Welcome to the DAARION Constrained Internal Pilot. This flow is for approved Pilot Participants to join the network in Advisory Mode.
          </p>
        </header>

        {/* What this is */}
        <section className="space-y-4">
          <h2 className="text-lg font-bold text-white/80 flex items-center gap-2">
            <Info size={16} className="text-blue-500" /> What this is
          </h2>
          <div className="glass p-6 md:p-8 border-white/5 text-sm text-white/60 leading-relaxed space-y-4 rounded-2xl bg-white/[0.02]">
            <p>
              This is a <strong>Bounded Environment</strong> for executing deterministic edge tasks (`ping_math` and `text_hash`). 
            </p>
            <p>
              By installing the Desktop Worker, your machine acts in an <strong>advisory-only</strong> capacity. All cryptographic validation and final state acceptance remains strictly on the canonical backend.
            </p>
            <div className="mt-4 p-4 bg-red-500/10 border border-red-500/20 rounded-xl">
               <p className="text-red-400/90 italic font-medium text-xs">
                 This is NOT a public release. This software does NOT provide write access or sovereign control over the network.
               </p>
            </div>
          </div>
        </section>

        {/* Platform Support */}
        <section className="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div className="glass p-6 border-white/5 space-y-3 rounded-2xl bg-white/[0.02]">
            <div className="flex items-center gap-2 text-white/80 font-bold mb-4">
              <Monitor size={16} className="text-emerald-500" />
              Supported Platform
            </div>
            <div className="text-sm text-white/60">
              <strong>macOS (Apple Silicon or Intel)</strong><br />
              Native app with in-app Worker activation. No terminal required.
            </div>
            <div className="mt-6 px-3 py-2 bg-emerald-500/10 border border-emerald-500/20 rounded-lg text-emerald-400 text-xs font-bold text-center uppercase tracking-wider">
              Active for current pilot wave
            </div>
          </div>

          <div className="glass p-6 border-white/5 space-y-3 rounded-2xl bg-white/[0.02] opacity-70">
            <div className="flex items-center gap-2 text-white/80 font-bold mb-4">
              <HardDrive size={16} className="text-blue-500" />
              Windows Platform
            </div>
            <div className="text-sm text-white/60">
              <strong>Windows 11 (WSL2)</strong><br />
              Desktop Worker artifact builds successfully in CI. Native install and execution path remain unvalidated.
            </div>
            <div className="mt-6 px-3 py-2 bg-blue-500/10 border border-blue-500/20 rounded-lg text-blue-400 text-xs font-bold text-center uppercase tracking-wider">
              Build verified / Execution unvalidated
            </div>
          </div>
        </section>

        {/* Join Steps */}
        <section className="space-y-4">
          <h2 className="text-lg font-bold text-white/80 flex items-center gap-2">
            <Download size={16} className="text-emerald-500" /> How to Join (macOS)
          </h2>
          <div className="glass p-6 md:p-8 border-white/5 space-y-6 rounded-2xl bg-white/[0.02]">
            <p className="text-sm text-white/50 leading-relaxed">
              Download the latest macOS installer from GitHub Releases. The Worker Daemon is bundled entirely within the application — no terminal setup required.
            </p>
            
            <a 
              href="https://github.com/DAARION-DAO/daarion-edge-client/releases" 
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex items-center justify-center gap-2 w-full md:w-auto px-6 py-4 rounded-xl bg-emerald-600 hover:bg-emerald-500 text-white font-black uppercase tracking-[0.1em] text-[12px] transition-all shadow-[0_0_20px_rgba(16,185,129,0.2)] hover:shadow-[0_0_30px_rgba(16,185,129,0.4)]"
            >
              <Download size={14} /> Download from GitHub Releases
            </a>
            
            {/* Consumer-friendly steps — NOT a terminal block */}
            <div className="space-y-4 pt-4 border-t border-white/5">
              {[
                { n: 1, title: "Download the .dmg file", desc: "Select the latest macOS release artifact for your architecture (Apple Silicon or Intel)." },
                { n: 2, title: "Install the app", desc: "Open the .dmg and drag DAARION Edge into your Applications folder." },
                { n: 3, title: "Launch", desc: "Open DAARION Edge from Applications. If macOS Gatekeeper blocks, right-click → Open." },
                { n: 4, title: "Complete Genesis", desc: "Follow the in-app Sovereign Genesis wizard to create your agent identity." },
                { n: 5, title: "Worker Mode", desc: "Worker activation is blocked until cryptographic operator-token validation is available." },
              ].map((s) => (
                <div key={s.n} className="flex gap-3 items-start">
                  <div className="flex-shrink-0 w-7 h-7 rounded-full bg-emerald-600/20 border border-emerald-500/30 flex items-center justify-center text-[10px] font-black text-emerald-400">
                    {s.n}
                  </div>
                  <div>
                    <p className="text-sm font-bold text-white/80">{s.title}</p>
                    <p className="text-[11px] text-white/40 leading-relaxed mt-0.5">{s.desc}</p>
                  </div>
                </div>
              ))}
            </div>
            
            <div className="flex flex-col gap-3 mt-4 pt-4 border-t border-white/5">
              <div className="flex items-start gap-3">
                <Shield size={14} className="text-white/30 flex-shrink-0 mt-0.5" />
                <p className="text-xs text-white/40 italic">
                  Operator Approval Required: Ensure your identity is whitelisted by the pilot operator before joining the network.
                </p>
              </div>
              <div className="flex items-start gap-3">
                <Info size={14} className="text-blue-500/70 flex-shrink-0 mt-0.5" />
                <p className="text-xs text-white/40 italic">
                  <strong className="text-white/60">Manual Updates Only:</strong> Auto-update is not supported during the pilot. To update, manually download the latest release and replace the existing application.
                </p>
              </div>
            </div>
          </div>
        </section>

      </div>
    </div>
  );
}
