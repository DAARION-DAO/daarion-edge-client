/**
 * InstallPage.tsx
 * PWA installation guide page — shown at /install
 * Works on iOS (Safari), Android (Chrome), and Desktop (Chromium).
 */
import { useState, useEffect } from "react";
import {
  Smartphone, Monitor, Share, PlusSquare, Chrome,
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

// ── Step card ─────────────────────────────────────────────────────────────────
function Step({ num, icon: Icon, title, desc }: {
  num: number; icon: any; title: string; desc: string;
}) {
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

// ── Feature pill ──────────────────────────────────────────────────────────────
function Feature({ icon: Icon, label }: { icon: any; label: string }) {
  return (
    <div className="flex items-center gap-2 px-3 py-2 rounded-xl bg-white/[0.03] border border-white/5">
      <Icon size={12} className="text-blue-400/70" />
      <span className="text-[10px] text-white/40 font-medium">{label}</span>
    </div>
  );
}

export function InstallPage() {
  const [platform, setPlatform] = useState<Platform>("unknown");
  const isInstalled = useIsInstalled();

  useEffect(() => {
    setPlatform(detectPlatform());
  }, []);

  return (
    <div className="min-h-screen bg-[#020202] text-white font-sans flex flex-col items-center justify-center p-6 relative overflow-hidden">

      {/* Background glow */}
      <div className="absolute inset-0 pointer-events-none">
        <div className="absolute top-1/3 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[500px] h-[500px] rounded-full bg-blue-900/20 blur-[120px] animate-pulse" />
      </div>

      <div className="z-10 w-full max-w-sm space-y-8">

        {/* Header */}
        <div className="text-center space-y-3">
          <img src="/icon-512.png" alt="DAARION Edge" className="w-20 h-20 rounded-3xl mx-auto shadow-[0_0_40px_rgba(59,130,246,0.25)]" />
          <div>
            <p className="text-[9px] uppercase tracking-[0.4em] text-white/20 mb-1">DAARION Protocol</p>
            <h1 className="text-3xl font-black tracking-tighter bg-gradient-to-br from-white via-white/90 to-white/30 bg-clip-text text-transparent">
              EDGE<span className="text-blue-500 font-light ml-1">Client</span>
            </h1>
          </div>
          <p className="text-[11px] text-white/30 leading-relaxed max-w-xs mx-auto">
            Встановіть суверенний вузол мережі прямо на свій пристрій. Без App Store. Без посередників.
          </p>
        </div>

        {/* Features */}
        <div className="grid grid-cols-2 gap-2">
          <Feature icon={Wifi} label="Offline-ready" />
          <Feature icon={Shield} label="Sovereign Identity" />
          <Feature icon={Zap} label="Native-like UX" />
          <Feature icon={Globe} label="No App Store" />
        </div>

        {/* Already installed banner */}
        {isInstalled && (
          <div className="flex items-center gap-3 px-4 py-3 rounded-xl bg-emerald-500/10 border border-emerald-500/20">
            <CheckCircle size={16} className="text-emerald-400 flex-shrink-0" />
            <p className="text-[11px] text-emerald-400 font-bold">DAARION Edge вже встановлено! ✓</p>
          </div>
        )}

        {/* Platform-specific install steps */}
        {!isInstalled && (
          <div className="glass p-6 border-white/5 space-y-6">
            {/* Platform switcher */}
            <div className="flex gap-1 p-1 bg-white/[0.03] border border-white/5 rounded-xl">
              {(["ios", "android", "desktop"] as Platform[]).map((p) => (
                <button
                  key={p}
                  onClick={() => setPlatform(p)}
                  className={`flex-1 flex items-center justify-center gap-1.5 py-2 rounded-lg text-[9px] font-black uppercase tracking-wider transition-all ${
                    platform === p ? "bg-white/10 text-white" : "text-white/30 hover:text-white/60"
                  }`}
                >
                  {p === "ios" ? <Smartphone size={11} /> : p === "android" ? <Download size={11} /> : <Monitor size={11} />}
                  {p === "ios" ? "iOS" : p === "android" ? "Android" : "Desktop"}
                </button>
              ))}
            </div>

            {/* iOS steps */}
            {platform === "ios" && (
              <div className="space-y-5">
                <Step num={1} icon={Globe} title="Відкрийте в Safari"
                  desc="Переконайтесь, що використовуєте браузер Safari (Chrome не підтримує iOS PWA install)." />
                <Step num={2} icon={Share} title="Натисніть Share (поділитися)"
                  desc="Знайдіть іконку 'Share' внизу екрану Safari — квадрат зі стрілкою вгору." />
                <Step num={3} icon={PlusSquare} title="'Add to Home Screen'"
                  desc="Прокрутіть меню вниз і оберіть 'Add to Home Screen'. Натисніть 'Add'." />
                <p className="text-[9px] text-white/20 text-center italic">Потрібен iOS 16.4+ для повної підтримки PWA</p>
              </div>
            )}

            {/* Android steps */}
            {platform === "android" && (
              <div className="space-y-5">
                <Step num={1} icon={Chrome} title="Відкрийте в Chrome"
                  desc="Переконайтесь, що використовуєте Chrome або будь-який Chromium-браузер." />
                <Step num={2} icon={Download} title="'Install App' у браузері"
                  desc="Chrome автоматично покаже банер 'Add DAARION Edge to Home Screen' або кнопку ⊕ в адресному рядку." />
                <Step num={3} icon={PlusSquare} title="Підтвердіть встановлення"
                  desc="Натисніть 'Install' у діалозі. Іконка з'явиться на домашньому екрані." />
              </div>
            )}

            {/* Desktop steps */}
            {platform === "desktop" && (
              <div className="space-y-5">
                <Step num={1} icon={Chrome} title="Chrome / Edge / Arc"
                  desc="Відкрийте цю сторінку в Chromium-браузері (Chrome, Edge, Arc, Brave)." />
                <Step num={2} icon={Download} title="Кнопка install в адресному рядку"
                  desc="Знайдіть іконку ⊕ або 💻 в правій частині адресного рядка та натисніть 'Install DAARION Edge'." />
                <Step num={3} icon={Monitor} title="Запустіть як додаток"
                  desc="DAARION Edge відкриється у власному вікні без браузерного інтерфейсу — як нативний застосунок." />
                <p className="text-[9px] text-white/20 text-center italic">Firefox не підтримує install PWA — використовуйте Chrome або Edge</p>
              </div>
            )}
          </div>
        )}

        {/* CTA back to app */}
        <a
          href="/"
          className="flex items-center justify-center gap-2 w-full py-3.5 rounded-xl bg-blue-600 hover:bg-blue-500 text-white font-black uppercase tracking-[0.15em] text-[11px] transition-all duration-200 shadow-[0_0_20px_rgba(37,99,235,0.25)] hover:shadow-[0_0_30px_rgba(37,99,235,0.4)]"
        >
          <Zap size={14} />
          {isInstalled ? "Відкрити DAARION Edge" : "Продовжити в браузері"}
        </a>

        <p className="text-center text-[9px] text-white/15">
          DAARION Protocol · Edge Client v0.1 · PWA
        </p>
      </div>
    </div>
  );
}
