// Localization for the desktop app. English is the source language and the
// fallback; every other locale may omit keys and will fall back per-key.
//
// Rust never returns user-facing prose: it returns machine-readable codes like
// `invalid_credentials` or `write_failed:<detail>`, which `translateError`
// resolves here. That keeps the backend language-agnostic.

export type Locale = "en" | "it";

export const LOCALES: Array<{ code: Locale; label: string }> = [
  { code: "en", label: "English" },
  { code: "it", label: "Italiano" },
];

type Dict = Record<string, string>;

const en: Dict = {
  "app.subtitle": "Deck Sync",
  "app.settings": "Settings",

  "link.title": "Connect your account",
  "link.lead": "Authorize this computer from your browser, then your decklists appear straight inside OP TCG Simulator.",
  "link.cta": "Connect account",
  "link.waiting": "Waiting for your browser…",
  "link.why": "Sign-in happens on nakamacompanion.com, so your password never passes through this app.",
  "link.unlink": "Disconnect this computer",

  "linkPage.doneTitle": "All set",
  "linkPage.doneBody": "You can go back to Nakama Companion Deck Sync.",
  "linkPage.failedTitle": "Connection failed",
  "linkPage.failedBody": "Go back to the app and try again.",



  "status.waiting": "Waiting",
  "status.ready": "Ready",
  "status.setupNeeded": "Setup needed",
  "status.setupDetail": "Tell the app where the simulator lives.",
  "status.syncing": "Syncing",
  "status.syncingDetail": "Fetching the decks from your account…",
  "status.synced": "Everything in sync",
  "status.syncedWithWarnings": "Synced with warnings",
  "status.syncFailed": "Sync failed",
  "status.warningsDetail": "{summary} — {count} error(s), see the details.",

  "setup.title": "Where is the simulator?",
  "setup.description": "The app needs the folder literally named \"{folder}\" inside your OP TCG Simulator install (not the main install folder). Let's try to find it automatically.",
  "setup.autodetect": "Find it automatically",
  "setup.searching": "Searching…",
  "setup.manual": "Choose manually…",
  "setup.notFound": "Couldn't find the simulator. Please pick the Decks folder manually.",

  "sync.now": "Sync now",
  "sync.running": "Syncing…",
  "sync.deckCount": "Decks synced",
  "sync.simulator": "Simulator",
  "sync.simulatorRunning": "Running",
  "sync.simulatorClosed": "Closed",
  "sync.logTitle": "Last sync details",
  "sync.updated": "Updated — {file}",
  "sync.removed": "Removed — {file}",
  "sync.allUpToDate": "All decks were already up to date.",
  "sync.noDecksFound": "No decks found on your account. Save a deck in the builder on nakamacompanion.com first — decks only sync once you press Save/Update there.",
  "sync.summaryUpdated": "{count} updated",
  "sync.summaryRemoved": "{count} removed",
  "sync.summaryUnchanged": "{count} already in sync",
  "sync.summaryNothing": "No changes",

  "settings.title": "Settings",
  "settings.folder": "Simulator Decks folder",
  "settings.folderEmpty": "Not set",
  "settings.browse": "Browse",
  "settings.onLaunch": "Sync when the simulator starts",
  "settings.onLaunchHint": "Refreshes your decks as soon as you open OP TCG Simulator.",
  "settings.autostart": "Start with Windows",
  "settings.autostartHint": "Stays in the system tray, quietly.",
  "settings.language": "Language",
  "settings.advanced": "Advanced",
  "settings.backendUrl": "Backend URL",
  "settings.save": "Save",
  "settings.saved": "Settings saved.",

  "error.not_linked": "This computer isn't connected yet.",
  "error.device_revoked": "This computer's access was revoked. Connect it again.",
  "error.claim_failed": "Couldn't complete the connection. ({detail})",
  "error.pairing_timeout": "Connection timed out. Try again.",
  "error.state_mismatch": "The connection failed a security check. Try again.",
  "error.no_claim_code": "The browser didn't send back a connection code.",
  "error.loopback_failed": "Couldn't open the local listener.",
  "error.network_error": "Can't reach the server. Check your connection.",
  "error.backend_error": "The server returned an error.",
  "error.invalid_response": "Unexpected response from the server.",
  "error.invalid_cards": "Deck \"{detail}\" has unreadable card data.",
  "error.unreleased_leader": "Deck \"{detail}\": the leader isn't officially released yet, so this deck can't be synced yet.",
  "error.unresolved_cards": "Deck \"{detail}\": {count} card(s) aren't officially released yet and were skipped from the file.",
  "error.no_folder": "No simulator folder selected.",
  "error.decks_folder_not_found": "That folder doesn't look right — pick the folder named \"Decks\" inside your OP TCG Simulator install, or its parent folder and we'll find it for you.",
  "error.folder_missing": "The folder no longer exists: {detail}",
  "error.write_failed": "Couldn't write {detail}",
  "error.delete_failed": "Couldn't delete {detail}",
  "error.keyring_unavailable": "Windows Credential Manager is unavailable.",
  "error.keyring_write_failed": "Couldn't store your session securely.",
  "error.config_dir_unavailable": "Couldn't open the app data folder.",
  "error.config_dir_failed": "Couldn't create the app data folder.",
  "error.config_write_failed": "Couldn't save the settings.",
  "error.browser_open_failed": "Couldn't open your browser.",
  "error.unknown": "Something went wrong.",

  "tray.open": "Open",
  "tray.sync": "Sync now",
  "tray.quit": "Quit",
};

const it: Dict = {
  "app.subtitle": "Deck Sync",
  "app.settings": "Impostazioni",

  "link.title": "Collega il tuo account",
  "link.lead": "Autorizza questo computer dal browser, poi le tue decklist compaiono direttamente dentro OP TCG Simulator.",
  "link.cta": "Collega account",
  "link.waiting": "In attesa del browser…",
  "link.why": "L'accesso avviene su nakamacompanion.com, così la tua password non passa mai da questa app.",
  "link.unlink": "Scollega questo computer",

  "linkPage.doneTitle": "Fatto",
  "linkPage.doneBody": "Puoi tornare a Nakama Companion Deck Sync.",
  "linkPage.failedTitle": "Collegamento non riuscito",
  "linkPage.failedBody": "Torna all'app e riprova.",



  "status.waiting": "In attesa",
  "status.ready": "Pronto",
  "status.setupNeeded": "Configurazione richiesta",
  "status.setupDetail": "Indica dove si trova il simulatore.",
  "status.syncing": "Sincronizzazione in corso",
  "status.syncingDetail": "Recupero i mazzi dal tuo account…",
  "status.synced": "Tutto sincronizzato",
  "status.syncedWithWarnings": "Sincronizzato con avvisi",
  "status.syncFailed": "Sincronizzazione fallita",
  "status.warningsDetail": "{summary} — {count} errore/i, vedi i dettagli.",

  "setup.title": "Dov'è il simulatore?",
  "setup.description": "Serve la cartella chiamata esattamente \"{folder}\" dentro l'installazione di OP TCG Simulator (non la cartella principale del simulatore). Proviamo a trovarla in automatico.",
  "setup.autodetect": "Trova automaticamente",
  "setup.searching": "Ricerca in corso…",
  "setup.manual": "Scegli manualmente…",
  "setup.notFound":
    "Non sono riuscito a trovare il simulatore. Seleziona la cartella Decks manualmente.",

  "sync.now": "Sincronizza ora",
  "sync.running": "Sincronizzazione…",
  "sync.deckCount": "Mazzi sincronizzati",
  "sync.simulator": "Simulatore",
  "sync.simulatorRunning": "In esecuzione",
  "sync.simulatorClosed": "Chiuso",
  "sync.logTitle": "Dettagli ultima sincronizzazione",
  "sync.updated": "Aggiornato — {file}",
  "sync.removed": "Rimosso — {file}",
  "sync.allUpToDate": "Tutti i mazzi erano già aggiornati.",
  "sync.noDecksFound": "Nessun mazzo trovato sul tuo account. Salva prima un mazzo nel builder su nakamacompanion.com — i mazzi si sincronizzano solo dopo aver premuto Save/Update lì.",
  "sync.summaryUpdated": "{count} aggiornati",
  "sync.summaryRemoved": "{count} rimossi",
  "sync.summaryUnchanged": "{count} già allineati",
  "sync.summaryNothing": "Nessuna modifica",

  "settings.title": "Impostazioni",
  "settings.folder": "Cartella Decks del simulatore",
  "settings.folderEmpty": "Non impostata",
  "settings.browse": "Sfoglia",
  "settings.onLaunch": "Sincronizza all'avvio del simulatore",
  "settings.onLaunchHint": "Aggiorna i mazzi appena apri OP TCG Simulator.",
  "settings.autostart": "Avvia con Windows",
  "settings.autostartHint": "Resta nella barra delle applicazioni, in silenzio.",
  "settings.language": "Lingua",
  "settings.advanced": "Avanzate",
  "settings.backendUrl": "URL del backend",
  "settings.save": "Salva",
  "settings.saved": "Impostazioni salvate.",

  "error.not_linked": "Questo computer non è ancora collegato.",
  "error.device_revoked": "L'accesso di questo computer è stato revocato. Ricollegalo.",
  "error.claim_failed": "Impossibile completare il collegamento. ({detail})",
  "error.pairing_timeout": "Tempo scaduto per il collegamento. Riprova.",
  "error.state_mismatch": "Controllo di sicurezza fallito. Riprova.",
  "error.no_claim_code": "Il browser non ha restituito un codice di collegamento.",
  "error.loopback_failed": "Impossibile avviare il ricevitore locale.",
  "error.network_error": "Server irraggiungibile. Controlla la connessione.",
  "error.backend_error": "Il server ha restituito un errore.",
  "error.invalid_response": "Risposta del server non valida.",
  "error.invalid_cards": "Il mazzo \"{detail}\" ha dati carta illeggibili.",
  "error.unreleased_leader": "Mazzo \"{detail}\": il leader non è ancora rilasciato ufficialmente, questo mazzo non può ancora essere sincronizzato.",
  "error.unresolved_cards": "Mazzo \"{detail}\": {count} carta/e non ancora rilasciate ufficialmente, saltate dal file.",
  "error.no_folder": "Nessuna cartella del simulatore selezionata.",
  "error.decks_folder_not_found": "Quella cartella non sembra giusta — scegli la cartella chiamata \"Decks\" dentro l'installazione di OP TCG Simulator, oppure la sua cartella superiore e la trovo io.",
  "error.folder_missing": "La cartella non esiste più: {detail}",
  "error.write_failed": "Impossibile scrivere {detail}",
  "error.delete_failed": "Impossibile cancellare {detail}",
  "error.keyring_unavailable": "Gestione credenziali di Windows non disponibile.",
  "error.keyring_write_failed": "Impossibile salvare la sessione in modo sicuro.",
  "error.config_dir_unavailable": "Impossibile aprire la cartella dati dell'app.",
  "error.config_dir_failed": "Impossibile creare la cartella dati dell'app.",
  "error.config_write_failed": "Impossibile salvare le impostazioni.",
  "error.browser_open_failed": "Impossibile aprire il browser.",
  "error.unknown": "Qualcosa è andato storto.",

  "tray.open": "Apri",
  "tray.sync": "Sincronizza ora",
  "tray.quit": "Esci",
};

const DICTS: Record<Locale, Dict> = { en, it };

const STORAGE_KEY = "nakama.decksync.locale";

/** Falls back to the OS/webview language, then English. */
function detectLocale(): Locale {
  const stored = localStorage.getItem(STORAGE_KEY);
  if (stored && stored in DICTS) return stored as Locale;

  const preferred = navigator.languages ?? [navigator.language];
  for (const tag of preferred) {
    const base = tag.toLowerCase().split("-")[0];
    if (base in DICTS) return base as Locale;
  }
  return "en";
}

let current: Locale = detectLocale();

export function getLocale(): Locale {
  return current;
}

export function setLocale(locale: Locale) {
  current = locale;
  localStorage.setItem(STORAGE_KEY, locale);
  document.documentElement.lang = locale;
  applyTranslations();
}

/** Looks up `key`, substituting `{name}` placeholders from `params`. */
export function t(key: string, params: Record<string, string | number> = {}): string {
  const template = DICTS[current][key] ?? en[key] ?? key;
  return template.replace(/\{(\w+)\}/g, (match, name: string) =>
    name in params ? String(params[name]) : match,
  );
}

/**
 * Turns a Rust error string into localized prose. The format is
 * `code` or `code:detail`; unknown codes fall back to the raw string so
 * nothing is ever silently swallowed.
 */
export function translateError(raw: unknown): string {
  const text = String(raw ?? "").trim();
  if (!text) return t("error.unknown");

  const separator = text.indexOf(":");
  const code = separator === -1 ? text : text.slice(0, separator);
  let detail = separator === -1 ? "" : text.slice(separator + 1).trim();

  // A few codes (e.g. unresolved_cards) carry a second, purely-numeric field
  // after a trailing "|" — safe to pass straight through untranslated, since
  // Rust never sends prose, only counts/names.
  let count: string | undefined;
  const pipeIndex = detail.lastIndexOf("|");
  if (pipeIndex !== -1) {
    count = detail.slice(pipeIndex + 1);
    detail = detail.slice(0, pipeIndex);
  }

  const key = `error.${code}`;
  const hasKey = key in DICTS[current] || key in en;
  if (!hasKey) return text;

  return t(key, count !== undefined ? { detail, count } : { detail });
}

/**
 * Applies translations to every element carrying a `data-i18n*` attribute.
 * Called on boot and whenever the language changes.
 */
export function applyTranslations(root: ParentNode = document) {
  root.querySelectorAll<HTMLElement>("[data-i18n]").forEach((el) => {
    el.textContent = t(el.dataset.i18n!);
  });
  root.querySelectorAll<HTMLElement>("[data-i18n-placeholder]").forEach((el) => {
    (el as HTMLInputElement).placeholder = t(el.dataset.i18nPlaceholder!);
  });
  root.querySelectorAll<HTMLElement>("[data-i18n-title]").forEach((el) => {
    el.title = t(el.dataset.i18nTitle!);
  });
  document.documentElement.lang = current;
}

/** Copy handed to Rust for the page shown in the browser after linking. */
export function linkPageCopy() {
  return {
    done_title: t("linkPage.doneTitle"),
    done_body: t("linkPage.doneBody"),
    failed_title: t("linkPage.failedTitle"),
    failed_body: t("linkPage.failedBody"),
  };
}
