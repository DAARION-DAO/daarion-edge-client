import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Bot, Cpu, FolderOpen, Link2, ShieldCheck, CheckCircle2, Loader2, X } from "lucide-react";
import "./LocalAgentPilotPage.css";

type Profile = { node_id: string; agent_id: string; name: string; purpose: string };
type DeviceReadiness = { os: string; arch: string; cpu: string; cores: number; ram_total_gb: number; ram_available_gb: number; ollama_found: boolean };
type Entry = { name: string; kind: string; bytes: number; excerpt: string | null };
type Task = { id: string; created_at: string; status: string; model: string | null; report: { folder: string; entries: Entry[]; skipped: number; truncated: boolean } | null; answer: string | null; error: string | null; city?: { thread_id: string; run_id: string; request_hash: string } | null };
type Snapshot = { profile: Profile | null; folder: string | null; active: string | null; tasks: Task[]; models: { canonical_model_id: string; installed: boolean }[]; model_ready: boolean; city?: { listener_ready: boolean; connected: boolean; profile_id: string | null; detail: string } };
const statuses: Record<string, string> = { completed: "Збережено", failed: "Не завершено", interrupted: "Перервано закриттям", cancelled: "Скасовано", running: "Виконується" };

export function LocalAgentPilotPage() {
  const [data, setData] = useState<Snapshot | null>(null);
  const [device, setDevice] = useState<DeviceReadiness | null>(null);
  const [name, setName] = useState("");
  const [purpose, setPurpose] = useState("");
  const [model, setModel] = useState("");
  const [busy, setBusy] = useState("");
  const [error, setError] = useState("");
  const [selected, setSelected] = useState<string | null>(null);
  const refresh = async () => {
    const next = await invoke<Snapshot>("pilot_snapshot");
    setData(next);
    setModel(current => next.models.some(m => m.canonical_model_id === current && m.installed) ? current : "");
  };
  useEffect(() => { refresh().catch(() => setError("Цей екран працює у встановленому Edge Local Pilot.")); }, []);
  useEffect(() => {
    if (!data?.city?.listener_ready) return;
    const timer = window.setInterval(() => { refresh().catch(() => {}); }, 4000);
    return () => window.clearInterval(timer);
  }, [data?.city?.listener_ready]);
  const act = async (label: string, command: string, args?: Record<string, unknown>) => {
    setBusy(label); setError("");
    try { await invoke(command, args); await refresh(); }
    catch (e) { setError(typeof e === "string" ? e : "Дію не завершено. Спробуйте ще раз."); await refresh().catch(() => {}); }
    finally { setBusy(""); }
  };
  const task = data?.tasks.find(t => t.id === selected) ?? data?.tasks[0];
  const scan = async () => {
    setBusy("Перевіряю пристрій…"); setError("");
    try { setDevice(await invoke<DeviceReadiness>("pilot_scan_device")); }
    catch { setError("Не вдалося перевірити пристрій. Спробуйте ще раз."); }
    finally { setBusy(""); }
  };
  const installed = data?.models.filter(m => m.installed) ?? [];
  return <main className="local-pilot">
    <header className="lp-header"><div className="lp-brand"><div className="lp-logo"><Bot size={25} /></div><div><strong>DAARION Edge</strong><span>Мій локальний агент · пілот</span></div></div><div className="lp-badge"><span /> На цьому комп’ютері</div></header>
    <section className="lp-intro"><p className="lp-eyebrow">ВАШ АГЕНТ. ВАША НОДА.</p><h1>Почнімо з вашого комп’ютера.</h1><p>Створіть агента, дайте йому доступ до однієї папки та отримайте перший збережений результат.</p></section>
    {error && <div role="alert" className="lp-error">{error}</div>}
    <div className="lp-grid">
      <aside>
        <section className="lp-card"><div className="lp-heading"><Cpu size={21} /><h2>Перевірка пристрою</h2></div>
          <p className="lp-note">Перевірка працює локально. Вона не читає ваші документи, не завантажує моделі й не підключає комп’ютер до міста.</p>
          <button disabled={!!busy || !data} onClick={() => void scan()}>Перевірити пристрій</button>
          {device && <><p role="status">Базову перевірку завершено</p><dl className="lp-refs"><dt>Система</dt><dd>{device.os} · {device.arch}</dd><dt>Процесор</dt><dd>{device.cpu} · {device.cores} ядер</dd><dt>Пам’ять</dt><dd>{device.ram_total_gb.toFixed(1)} ГБ · доступно {device.ram_available_gb.toFixed(1)} ГБ</dd></dl>
            <p>{device.ollama_found ? "Знайдено Ollama. Після створення агента можна перевірити вже встановлені моделі в блоці завдання." : "Ollama у стандартному місці не знайдено. Зараз доступний огляд папки без моделі; автоматичне встановлення AI ще не підключене."}</p>
            <p className="lp-note">GPU, драйвери та швидкість моделі ще не перевірено. Повний підбір моделі й план її встановлення ще не підключені.</p>
          </>}
        </section>
        <section className="lp-card"><div className="lp-heading"><Bot size={21} /><h2>1. Власний агент</h2></div>
          {data?.profile ? <><div className="lp-agent"><div className="lp-avatar">{data.profile.name.slice(0, 1)}</div><div><h3>{data.profile.name}</h3><p>{data.profile.purpose || "Локальний помічник"}</p></div></div><dl className="lp-refs"><dt>Агент</dt><dd>{data.profile.agent_id.slice(0, 8)}</dd><dt>Нода</dt><dd>{data.profile.node_id.slice(0, 8)}</dd></dl><p className="lp-note">Профіль і посилання ноди збережені локально. Це ще не реєстрація у DAGI.</p></> : <form onSubmit={e => { e.preventDefault(); void act("Створюю агента…", "pilot_create_agent", { name, purpose }); }}>
            <label>Ім’я агента<input aria-label="Ім’я агента" maxLength={80} required value={name} onChange={e => setName(e.target.value)} placeholder="Як звати вашого помічника?" /></label>
            <label>Для чого він вам<textarea aria-label="Призначення агента" maxLength={500} value={purpose} onChange={e => setPurpose(e.target.value)} placeholder="Наприклад, допомагати з робочими матеріалами" rows={2} /></label>
            <button className="lp-primary" disabled={!!busy || !data}>Створити власного агента</button>
          </form>}
        </section>
        <section className="lp-card"><div className="lp-heading"><FolderOpen size={21} /><h2>2. Дозволена папка</h2></div>
          {data?.folder ? <><div className="lp-folder"><FolderOpen size={23} /><strong>{data.folder}</strong></div><p className="lp-note">Лише читання. Дозвіл діє до закриття застосунку.</p><div className="lp-actions"><button disabled={!!busy} onClick={() => void act("Виберіть папку…", "pilot_choose_folder")}>Змінити папку</button><button className="lp-text" disabled={!!busy} onClick={() => void act("Відкликаю доступ…", "pilot_revoke_folder")}>Відкликати доступ</button></div></> : <><p className="lp-note">Оберіть папку через вікно системи. Підпапки, приховані файли та посилання пропускаються.</p><button disabled={!!busy || !data?.profile} onClick={() => void act("Виберіть папку…", "pilot_choose_folder")}><FolderOpen size={16} /> Обрати робочу папку</button></>}
        </section>
        <section className="lp-card lp-network"><div className="lp-heading"><Link2 size={21} /><h2>Мережа міста</h2></div>
          <span className="lp-chip">{data?.city?.connected ? "Локально підключено" : data?.city?.listener_ready ? "Edge готовий · панель очікує" : "Ще не підключено"}</span>
          <p>{data?.city?.detail || "Підключіть цього агента до наявної міської панелі на цьому комп’ютері."}</p>
          <p className="lp-note">Пілот використовує локальну операторську панель. Вхід працівника через MicroDAO ще не налаштований.</p>
          <p className="lp-note">У чат надходять лише повідомлення цієї розмови. Доступ до вибраної папки місту не передається.</p>
          {!data?.city?.listener_ready ? <button disabled={!!busy || !model || !data?.model_ready} onClick={() => void act("З’єдную агента з міською панеллю…", "pilot_connect_city", { modelId: model })}>Підключити до міської панелі</button> : <button disabled={!!busy} onClick={() => void act("Від’єдную міський чат…", "pilot_disconnect_city")}>Від’єднати міський чат</button>}
          {!model && <p className="lp-note">Спочатку підключіть і виберіть локальну модель у блоці завдання.</p>}
          <button className="lp-text" onClick={() => void act("Відкриваю міську панель…", "pilot_open_city")}>Відкрити міську панель</button>
        </section>
      </aside>
      <div>
        <section className="lp-card"><div className="lp-heading"><Cpu size={21} /><h2>3. Перше локальне завдання</h2></div><h3>Розібратися, що є в робочій папці</h3><p className="lp-description">Агент складе перелік файлів і папок. За вашим вибором локальна модель додасть короткий огляд і запропонує наступну дію.</p>
          <div className="lp-model"><div><strong>Локальна модель</strong><p className="lp-note">Лише вже встановлена Ollama. Без хмарного виконання й автоматичного завантаження моделей. Тимчасовий запуск на CPU.</p></div>
          {!data?.model_ready ? <button disabled={!!busy || !data?.profile} onClick={() => void act("Перевіряю встановлені моделі…", "pilot_start_model")}>Підключити локальну модель</button> : <label className="lp-model-select">Спосіб виконання<select aria-label="Локальна модель" value={model} onChange={e => setModel(e.target.value)} disabled={!!busy}><option value="">Огляд без моделі</option>{installed.map(m => <option value={m.canonical_model_id} key={m.canonical_model_id}>{m.canonical_model_id.replace("pilot-installed:", "")}</option>)}</select></label>}
          </div>
          <p className="lp-note">{model ? "Модель отримає назви до 20 елементів і короткі уривки .txt та .md тільки з вибраної папки. Відповідь моделі може містити помилки." : "Без моделі: точний перелік до 256 елементів і короткі уривки до 8 текстових файлів. Це огляд інструментом, без AI-висновків."}</p>
          <button className="lp-primary" disabled={!!busy || !data?.folder} onClick={() => { setSelected(null); void act(model ? "Агент готує локальний AI-огляд…" : "Агент оглядає папку…", "pilot_run_task", { modelId: model || null }); }}>{model ? "Зробити AI-огляд папки" : "Виконати огляд папки"}</button>
          {busy && <div role="status" className="lp-progress"><Loader2 className="lp-spin" size={16} />{busy}{busy.includes("AI-огляд") && <button onClick={() => invoke("pilot_cancel").catch(() => setError("Не вдалося скасувати завдання."))}><X size={14} /> Скасувати</button>}</div>}
        </section>
        <section className="lp-card lp-result"><div className="lp-heading"><CheckCircle2 size={21} /><h2>Збережений результат</h2>{task && <span className="lp-chip">{statuses[task.status] ?? task.status}</span>}</div>
          {!task ? <div className="lp-empty"><ShieldCheck size={38} /><p>Тут з’явиться результат першого завдання.</p><span>Він залишиться після закриття застосунку.</span></div> : <>
            {data && data.tasks.length > 1 && <label>Історія завдань<select aria-label="Історія завдань" value={task.id} onChange={e => setSelected(e.target.value)}>{data.tasks.map(t => <option key={t.id} value={t.id}>{new Date(t.created_at).toLocaleString("uk-UA")} · {statuses[t.status]} · {t.model ? "AI" : "огляд"}</option>)}</select></label>}
            <p className="lp-note">{new Date(task.created_at).toLocaleString("uk-UA")} · {task.city ? "Міський чат" : task.report?.folder} · {task.model?.replace("pilot-installed:", "") ?? "Огляд інструментом, без моделі"}</p>
            {task.city && <p className="lp-note">Завдання з міської розмови збережено локально. Файли комп’ютера не передавалися.</p>}
            {task.error && <p className="lp-error">{task.error}</p>}
            {task.answer && <div className="lp-answer"><strong>Відповідь локальної моделі</strong><p>{task.answer}</p></div>}
            {task.report && <><p className="lp-summary"><strong>{task.report.entries.length}</strong> елементів у збереженому огляді · пропущено {task.report.skipped}{task.report.truncated ? " · досягнуто ліміт, перелік неповний" : " · без обходу підпапок"}</p><div className="lp-file-list">{task.report.entries.map((e, i) => <details key={i}><summary><span>{e.kind === "folder" ? "▸" : "·"} {e.name}</span><span>{e.kind === "folder" ? "папка" : `${e.bytes} Б`}</span></summary>{e.excerpt ? <pre>{e.excerpt}</pre> : <p className="lp-note">Вміст не читався.</p>}</details>)}</div></>}
            <p className="lp-note lp-footer-note">Збережено у сховищі цього пілота. Вихідні файли не змінювалися.</p>
          </>}
        </section>
      </div>
    </div>
  </main>;
}
