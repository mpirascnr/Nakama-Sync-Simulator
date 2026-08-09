import "@fontsource/manrope/400.css";
import "@fontsource/manrope/500.css";
import "@fontsource/manrope/700.css";
import "@fontsource/manrope/800.css";

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

import {
  applyTranslations,
  getLocale,
  linkPageCopy,
  LOCALES,
  setLocale,
  t,
  translateError,
  type Locale,
} from "./i18n";

type AppConfig = {
  simulator_decks_folder: string | null;
  site_url: string;
  backend_api_url: string;
  autostart_enabled: boolean;
  sync_on_simulator_launch: boolean;
  synced_decks: Record<string, { filename: string; updated_at: number }>;
  linked: boolean;
};

type SyncReport = {
  written: string[];
  deleted: string[];
  skipped_unchanged: number;
  errors: string[];
  total_decks: number;
};

const $ = <T extends HTMLElement = HTMLElement>(id: string) =>
  document.getElementById(id) as T;

const views = ["link", "main", "settings"] as const;
type ViewName = (typeof views)[number];

let lastReport: SyncReport | null = null;

function showView(name: ViewName) {
  for (const v of views) {
    $(`view-${v}`).hidden = v !== name;
  }
  $("settings-btn").hidden = name === "link";
}

function setStatus(
  tone: "idle" | "ok" | "warn" | "err" | "busy",
  title: string,
  detail: string,
) {
  $("status-dot").className = `dot${tone === "idle" ? "" : ` ${tone}`}`;
  $("status-title").textContent = title;
  $("status-detail").textContent = detail;
}

function formatTime(date = new Date()) {
  return date.toLocaleTimeString(getLocale(), { hour: "2-digit", minute: "2-digit" });
}

function describeReport(report: SyncReport): string {
  const parts: string[] = [];
  if (report.written.length)
    parts.push(t("sync.summaryUpdated", { count: report.written.length }));
  if (report.deleted.length)
    parts.push(t("sync.summaryRemoved", { count: report.deleted.length }));
  if (report.skipped_unchanged)
    parts.push(t("sync.summaryUnchanged", { count: report.skipped_unchanged }));
  return parts.length ? parts.join(" · ") : t("sync.summaryNothing");
}

function renderReport(report: SyncReport) {
  lastReport = report;
  const log = $("sync-log");
  log.innerHTML = "";

  const rows: Array<{ text: string; isError?: boolean }> = [
    ...report.written.map((file) => ({ text: t("sync.updated", { file }) })),
    ...report.deleted.map((file) => ({ text: t("sync.removed", { file }) })),
    ...report.errors.map((e) => ({ text: translateError(e), isError: true })),
  ];
  if (!rows.length) {
    rows.push({
      text: report.total_decks === 0 ? t("sync.noDecksFound") : t("sync.allUpToDate"),
    });
  }

  for (const row of rows) {
    const li = document.createElement("li");
    li.textContent = row.text;
    if (row.isError) li.classList.add("err");
    log.appendChild(li);
  }

  if (report.errors.length) {
    setStatus(
      "warn",
      t("status.syncedWithWarnings"),
      t("status.warningsDetail", {
        summary: describeReport(report),
        count: report.errors.length,
      }),
    );
  } else {
    setStatus("ok", t("status.synced"), `${describeReport(report)} · ${formatTime()}`);
  }
}

async function refreshMain(): Promise<AppConfig> {
  const config = await invoke<AppConfig>("get_settings");
  const configured = Boolean(config.simulator_decks_folder);

  $("setup-panel").hidden = configured;
  $("sync-panel").hidden = !configured;
  $("deck-count").textContent = String(Object.keys(config.synced_decks).length);

  if (configured) {
    setStatus("idle", t("status.ready"), config.simulator_decks_folder!);
  } else {
    setStatus("warn", t("status.setupNeeded"), t("status.setupDetail"));
  }

  void updateSimulatorState();
  return config;
}

async function updateSimulatorState(running?: boolean) {
  const isRunning = running ?? (await invoke<boolean>("is_simulator_running"));
  $("sim-state").textContent = isRunning
    ? t("sync.simulatorRunning")
    : t("sync.simulatorClosed");
}

async function refreshCounts() {
  const config = await invoke<AppConfig>("get_settings");
  $("deck-count").textContent = String(Object.keys(config.synced_decks).length);
}

async function runSync() {
  const btn = $<HTMLButtonElement>("sync-now-btn");
  btn.disabled = true;
  btn.classList.add("is-syncing");
  $("sync-btn-label").textContent = t("sync.running");
  setStatus("busy", t("status.syncing"), t("status.syncingDetail"));

  try {
    renderReport(await invoke<SyncReport>("sync_now"));
    $("log-details").setAttribute("open", "");
    await refreshCounts();
  } catch (err) {
    setStatus("err", t("status.syncFailed"), translateError(err));
  } finally {
    btn.disabled = false;
    btn.classList.remove("is-syncing");
    $("sync-btn-label").textContent = t("sync.now");
  }
}

/** Pushes the current language onto the tray menu, built in Rust before the
 *  webview loaded and therefore still in English. */
async function syncTrayLabels() {
  try {
    await invoke("set_tray_labels", {
      open: t("tray.open"),
      sync: t("tray.sync"),
      quit: t("tray.quit"),
    });
  } catch {
    // A tray we can't relabel is cosmetic; never block startup on it.
  }
}

function relocalize() {
  applyTranslations();
  $("setup-description").textContent = t("setup.description", { folder: "Decks" });
  if (lastReport) renderReport(lastReport);
  void updateSimulatorState();
  void syncTrayLabels();
}

async function bootstrap() {
  applyTranslations();
  $("setup-description").textContent = t("setup.description", { folder: "Decks" });
  void syncTrayLabels();

  const select = $<HTMLSelectElement>("settings-language");
  for (const { code, label } of LOCALES) {
    const option = document.createElement("option");
    option.value = code;
    option.textContent = label;
    select.appendChild(option);
  }
  select.value = getLocale();
  select.addEventListener("change", () => {
    setLocale(select.value as Locale);
    relocalize();
  });

  if (await invoke<boolean>("is_linked")) {
    await refreshMain();
    showView("main");
  } else {
    showView("link");
  }
}

// ---------- Link account ----------
$("link-btn").addEventListener("click", async () => {
  const btn = $<HTMLButtonElement>("link-btn");
  const errorEl = $("link-error");
  btn.disabled = true;
  $("link-btn-label").textContent = t("link.waiting");
  errorEl.hidden = true;

  try {
    await invoke("link_account", { copy: linkPageCopy() });
    await refreshMain();
    showView("main");
  } catch (err) {
    errorEl.textContent = translateError(err);
    errorEl.hidden = false;
  } finally {
    btn.disabled = false;
    $("link-btn-label").textContent = t("link.cta");
  }
});

// ---------- First-run setup ----------
$("autodetect-btn").addEventListener("click", async () => {
  const btn = $<HTMLButtonElement>("autodetect-btn");
  const errorEl = $("setup-error");
  btn.disabled = true;
  btn.textContent = t("setup.searching");
  errorEl.hidden = true;

  try {
    const found = await invoke<string | null>("autodetect_folder");
    if (found) {
      await refreshMain();
      await runSync();
    } else {
      errorEl.textContent = t("setup.notFound");
      errorEl.hidden = false;
    }
  } catch (err) {
    errorEl.textContent = translateError(err);
    errorEl.hidden = false;
  } finally {
    btn.disabled = false;
    btn.textContent = t("setup.autodetect");
  }
});

$("pick-folder-btn").addEventListener("click", async () => {
  const errorEl = $("setup-error");
  errorEl.hidden = true;
  try {
    const folder = await invoke<string | null>("pick_decks_folder");
    if (!folder) return;
    await invoke("save_settings", { input: { simulator_decks_folder: folder } });
    await refreshMain();
    await runSync();
  } catch (err) {
    errorEl.textContent = translateError(err);
    errorEl.hidden = false;
  }
});

// ---------- Sync ----------
$("sync-now-btn").addEventListener("click", () => void runSync());

// ---------- Settings ----------
$("settings-btn").addEventListener("click", async () => {
  const config = await invoke<AppConfig>("get_settings");
  $<HTMLInputElement>("settings-folder").value = config.simulator_decks_folder ?? "";
  $<HTMLInputElement>("settings-backend-url").value = config.backend_api_url;
  $<HTMLInputElement>("settings-autostart").checked = config.autostart_enabled;
  $<HTMLInputElement>("settings-on-launch").checked = config.sync_on_simulator_launch;
  $<HTMLSelectElement>("settings-language").value = getLocale();
  $("settings-status").hidden = true;
  showView("settings");
});

$("settings-pick-folder-btn").addEventListener("click", async () => {
  const statusEl = $("settings-status");
  statusEl.hidden = true;
  try {
    const folder = await invoke<string | null>("pick_decks_folder");
    if (folder) $<HTMLInputElement>("settings-folder").value = folder;
  } catch (err) {
    statusEl.textContent = translateError(err);
    statusEl.className = "alert alert-error";
    statusEl.hidden = false;
  }
});

$("settings-save-btn").addEventListener("click", async () => {
  const statusEl = $("settings-status");
  statusEl.hidden = true;

  try {
    await invoke("save_settings", {
      input: {
        simulator_decks_folder: $<HTMLInputElement>("settings-folder").value || null,
        backend_api_url: $<HTMLInputElement>("settings-backend-url").value.trim(),
        sync_on_simulator_launch: $<HTMLInputElement>("settings-on-launch").checked,
      },
    });
    await invoke("set_autostart", {
      enabled: $<HTMLInputElement>("settings-autostart").checked,
    });
    statusEl.textContent = t("settings.saved");
    statusEl.className = "alert alert-ok";
    statusEl.hidden = false;
  } catch (err) {
    statusEl.textContent = translateError(err);
    statusEl.className = "alert alert-error";
    statusEl.hidden = false;
  }
});

$("settings-back-btn").addEventListener("click", async () => {
  await refreshMain();
  showView("main");
});

$("unlink-btn").addEventListener("click", async () => {
  await invoke("unlink");
  lastReport = null;
  showView("link");
});

// ---------- Live updates from the background sync loop ----------
void listen<boolean>("simulator-status", (event) => {
  void updateSimulatorState(event.payload);
});

void listen<SyncReport>("sync-completed", (event) => {
  renderReport(event.payload);
  void refreshCounts();
});

void bootstrap();
