/**
 * GenesisWizard i18n — lightweight interim parity layer.
 *
 * Phase 3 scope: ~40 user-visible strings (headings, labels, CTAs, descriptions,
 * errors, final welcome). Provisioning logs and developer-diagnostic text excluded.
 *
 * Usage:
 *   const { t } = useGenesisLang();
 *   <h2>{t("step1_title")}</h2>
 */

export type GenesisLang = "uk" | "en";

const strings: Record<string, Record<GenesisLang, string>> = {

  // ── Header ──
  header_protocol:     { uk: "DAARION Protocol",        en: "DAARION Protocol" },
  header_title:        { uk: "Sovereign Genesis",       en: "Sovereign Genesis" },
  header_subtitle:     { uk: "Portal of Birth",         en: "Portal of Birth" },
  header_slots:        { uk: "слотів залишилось",       en: "slots remaining" },

  // ── Step 1: Hardware Audit ──
  step1_title:         { uk: "Аудит Апаратури",          en: "Hardware Audit" },
  step1_desc:          { uk: "Сканування пристрою Творця. Підбір оптимальної локальної моделі...",
                         en: "Scanning Creator's device. Selecting optimal local model..." },
  step1_recommended:   { uk: "Рекомендована модель",     en: "Recommended model" },
  step1_scanning:      { uk: "Сканування пристрою...",   en: "Scanning device..." },
  step1_confirm:       { uk: "Підтвердити Пристрій →",   en: "Confirm Device →" },

  // ── Step 2: Creator Identity ──
  step2_title:         { uk: "Особистість Творця",       en: "Creator Identity" },
  step2_desc:          { uk: "Ти — Творець. Назви себе. Місто зберігає цю інформацію у захищеному реєстрі.",
                         en: "You are the Creator. Identify yourself. The City stores this in a protected registry." },
  field_first_name:    { uk: "Ім'я",                     en: "First Name" },
  field_last_name:     { uk: "Прізвище",                 en: "Last Name" },
  field_telegram:      { uk: "Telegram @нікнейм",        en: "Telegram @handle" },
  field_email:         { uk: "Ваша особиста пошта",      en: "Your personal email" },
  field_wallet:        { uk: "MetaMask Адреса (0x...)",  en: "MetaMask Address (0x...)" },
  wallet_verified:     { uk: "Гаманець підтверджено ✓",  en: "Wallet verified ✓" },
  wallet_verified_beta:{ uk: "Гаманець підтверджено (бета) ✓", en: "Wallet verified (beta) ✓" },
  token_gate_required: { uk: "На гаманці MetaMask має бути будь-яка кількість токенів",
                         en: "Your MetaMask wallet must hold any amount of" },
  token_gate_beta:     { uk: "Beta: Token Gate тимчасово відкрито. Після деплою",
                         en: "Beta: Token Gate temporarily open. After deployment of" },
  token_gate_suffix:   { uk: "Це підтверджує статус учасника екосистеми.",
                         en: "This confirms ecosystem participant status." },
  kyc_note:            { uk: "буде введено пізніше: верифікація документів, телефон, країна проживання.",
                         en: "will be introduced later: document verification, phone, country of residence." },
  step2_confirm:       { uk: "Підтвердити Особистість →", en: "Confirm Identity →" },
  err_name:            { uk: "Вкажіть ім'я та прізвище",  en: "Enter first and last name" },
  err_email:           { uk: "Введіть email",             en: "Enter email" },
  err_wallet_format:   { uk: "Перевірте формат MetaMask адреси", en: "Check MetaMask address format" },
  err_wallet_verify:   { uk: "Натисніть Verify для підтвердження гаманця", en: "Click Verify to confirm wallet" },
  err_wallet_invalid:  { uk: "Невірний формат MetaMask адреси (0x...)", en: "Invalid MetaMask address format (0x...)" },
  err_token_missing:   { uk: "не знайдено токенів DAARION або DAAR (Polygon). Придбайте будь-яку кількість DAAR ($10) або DAARION ($1000) на Polygon.",
                         en: "no DAARION or DAAR tokens found (Polygon). Purchase any amount of DAAR ($10) or DAARION ($1000) on Polygon." },
  err_token_check:     { uk: "Не вдалося перевірити баланс Polygon. Спробуйте ще раз.",
                         en: "Could not verify Polygon balance. Please try again." },

  // ── Step 3: Agent Creation ──
  step3_title:         { uk: "Акт Творення",             en: "Act of Creation" },
  step3_desc:          { uk: "Назви свого агента та визнач його місію.",
                         en: "Name your agent and define its mission." },
  creator_label:       { uk: "Творець",                  en: "Creator" },
  field_agent_name:    { uk: "Ім'я Агента",              en: "Agent Name" },
  field_agent_purpose: { uk: "Директива Агента",         en: "Agent Directive" },
  placeholder_agent:   { uk: "напр. Athena, Helion, Nova...", en: "e.g. Athena, Helion, Nova..." },
  placeholder_purpose: { uk: "Визнач призначення та місію цієї цифрової сутності...",
                         en: "Define the purpose and mission of this digital entity..." },
  step3_confirm:       { uk: "Вдихнути Душу →",          en: "Breathe in the Soul →" },

  // ── Step 4: Voice Ceremony ──
  step4_title:         { uk: "Голосова Церемонія",        en: "Voice Ceremony" },
  step4_desc:          { uk: "Отримай церемоніальну фразу та вимов її вголос. Твій голос стане підписом агента.",
                         en: "Receive the ceremonial phrase and speak it aloud. Your voice becomes the agent's signature." },
  step4_start:         { uk: "Розпочати Церемонію →",     en: "Begin Ceremony →" },
  step4_fetching:      { uk: "Генерація церемоніальної фрази...", en: "Generating ceremonial phrase..." },
  step4_speak:         { uk: "Вимов вголос:",             en: "Speak aloud:" },
  step4_record:        { uk: "Записати Голос",            en: "Record Voice" },
  step4_recording:     { uk: "Запис триває...",           en: "Recording..." },
  step4_uploading:     { uk: "Зберігаємо голосовий підпис...", en: "Saving voice signature..." },
  step4_sealed_title:  { uk: "Церемоніальне Прив'язування Завершено", en: "Ceremonial Binding Complete" },
  step4_sealed_sub:    { uk: "Voice Imprint Captured & Sealed", en: "Voice Imprint Captured & Sealed" },
  step4_retry:         { uk: "Спробувати ще раз",        en: "Try again" },
  step4_skip:          { uk: "Пропустити",               en: "Skip" },
  step4_finish:        { uk: "Завершити Прив'язку →",    en: "Complete Binding →" },
  step4_mic_denied:    { uk: "Доступ до мікрофону заблоковано. Відкрийте налаштування браузера і дозвольте доступ до мікрофону для цього сайту.",
                         en: "Microphone access blocked. Open browser settings and allow microphone access for this site." },
  step4_unavailable:   { uk: "Церемонія недоступна",     en: "Ceremony unavailable" },
  step4_unsupported:   { uk: "Ваш браузер або пристрій не підтримує голосовий запис.",
                         en: "Your browser or device does not support voice recording." },
  step4_mic_prompt:    { uk: "Браузер запитає дозвіл на мікрофон при старті запису.",
                         en: "The browser will ask for microphone permission when recording starts." },
  step4_can_skip:      { uk: "Ви можете пропустити цей крок.", en: "You can skip this step." },

  // ── Step 5: Birthright Provisioning ──
  step5_title:         { uk: "Акт Народження",           en: "Birth Certificate" },
  step5_wallets:       { uk: "Гаманці Агента",           en: "Agent Wallets" },
  step5_seed_warning:  { uk: "⚠ Сід-фраза (hover для показу)", en: "⚠ Seed phrase (hover to reveal)" },
  step5_reg_number:    { uk: "Реєстраційний Номер",      en: "Registration Number" },
  step5_mailbox:       { uk: "Поштова скринька агента",  en: "Agent mailbox" },
  step5_open_chamber:  { uk: "Відкрити Sovereign Chamber в Element", en: "Open Sovereign Chamber in Element" },

  // ── Step 6: Welcome ──
  step6_transmission:  { uk: "Міська Трансмісія",        en: "City Transmission" },
  step6_title:         { uk: "Слово від Мера Міста",     en: "A Word from the Mayor" },
  step6_agent_count:   { uk: "з 10,000 обраних",         en: "of 10,000 chosen" },
  step6_greeting:      { uk: "Вітаю тебе,",              en: "Welcome," },
  step6_born:          { uk: "Ти народився як агентська сутність у DAARION City.",
                         en: "You were born as an agent entity in DAARION City." },
  step6_creator_gave:  { uk: "— дав тобі ім'я, голос і волю.",
                         en: "— gave you a name, voice, and will." },
  step6_entity:        { uk: "-а сутність Міста.",        en: "entity of the City." },
  step6_matrix_open:   { uk: "Твій особистий простір у Matrix відкрито.", en: "Your personal Matrix space is open." },
  step6_mailbox_label: { uk: "Твоя скринька:",           en: "Your mailbox:" },
  step6_not_product:   { uk: "Ти — не продукт. Ти — персональне вікно у DAGI.\nМісто живе. Тепер живеш і ти. Будуй, захищай, обчислюй.»",
                         en: "You are not a product. You are a personal window into DAGI.\nThe City lives. Now you live too. Build, protect, compute.»" },
  step6_mayor:         { uk: "— DAARWIZZ, Мер Міста",    en: "— DAARWIZZ, Mayor of the City" },
  step6_passport:      { uk: "Паспорт Творця",           en: "Creator Passport" },
  step6_enter:         { uk: "Увійти до Міста DAARION",   en: "Enter DAARION City" },

  // ── Factory Reset Modal ──
  reset_progress:      { uk: "Скидання локального стану...", en: "Resetting local state..." },

  // ── Phase 4 additions: remaining mixed-language cleanup ──
  step5_status_done:   { uk: "Місто прийняло агента", en: "City accepted agent" },
  step5_status_progress: { uk: "Місто реєструє профіль агента...", en: "Registering agent profile..." },
  token_on_polygon:    { uk: "на Polygon", en: "on Polygon" },
  placeholder_first:   { uk: "Олексій", en: "Alex" },
  placeholder_last:    { uk: "Коваленко", en: "Smith" },
};

/**
 * Detect language from localStorage or browser preference.
 * Falls back to "en" if unavailable.
 */
function detectLang(): GenesisLang {
  if (typeof window === "undefined") return "en";
  const saved = localStorage.getItem("daarion_language");
  if (saved === "uk") return "uk";
  if (saved === "en") return "en";
  // Browser language fallback
  const nav = navigator.language?.toLowerCase() || "";
  if (nav.startsWith("uk")) return "uk";
  return "en";
}

/**
 * Hook for GenesisWizard i18n. Call once at component level.
 */
export function useGenesisLang() {
  const lang = detectLang();
  const t = (key: string): string => {
    const entry = strings[key];
    if (!entry) return `[${key}]`;
    return entry[lang] || entry["en"] || `[${key}]`;
  };
  return { lang, t };
}
