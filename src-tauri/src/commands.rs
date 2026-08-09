use crate::config::{self, AppConfig};
use crate::credentials;
use crate::pairing::{self, BrowserPageCopy};
use crate::simulator;
use crate::sync::{self, SyncReport};
use tauri::menu::MenuItem;
use tauri::{AppHandle, State, Wry};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;

/// Handles to the tray menu entries, kept so the UI can relabel them once it
/// knows the user's language — the tray is built before the webview loads, so
/// it starts out in English.
pub struct TrayItems {
    pub open: MenuItem<Wry>,
    pub sync: MenuItem<Wry>,
    pub quit: MenuItem<Wry>,
}

#[tauri::command]
pub fn set_tray_labels(
    tray: State<'_, TrayItems>,
    open: String,
    sync: String,
    quit: String,
) -> Result<(), String> {
    tray.open.set_text(open).map_err(|e| e.to_string())?;
    tray.sync.set_text(sync).map_err(|e| e.to_string())?;
    tray.quit.set_text(quit).map_err(|e| e.to_string())?;
    Ok(())
}

/// Opens the browser so the user can authorize this machine, then stores the
/// device token it hands back. See pairing.rs for why linking happens in a
/// browser rather than in the app.
#[tauri::command]
pub async fn link_account(app: AppHandle, copy: BrowserPageCopy) -> Result<(), String> {
    let cfg = config::load(&app);
    let opener = app.clone();

    let token = pairing::link_account(&cfg.site_url, &cfg.backend_api_url, copy, move |url| {
        opener
            .opener()
            .open_url(url, None::<&str>)
            .map_err(|e| format!("browser_open_failed:{e}"))
    })
    .await?;

    credentials::store(&token)?;

    let mut cfg = config::load(&app);
    cfg.linked = true;
    config::save(&app, &cfg)
}

#[tauri::command]
pub fn is_linked() -> bool {
    credentials::load().is_some()
}

#[tauri::command]
pub fn unlink(app: AppHandle) -> Result<(), String> {
    credentials::clear();
    let mut cfg = config::load(&app);
    cfg.linked = false;
    // Forget what we mirrored too: the next pairing may be a different account,
    // and those filenames must not be treated as ours to delete.
    cfg.synced_decks.clear();
    config::save(&app, &cfg)
}

#[tauri::command]
pub fn get_settings(app: AppHandle) -> AppConfig {
    config::load(&app)
}

#[derive(Debug, serde::Deserialize)]
pub struct SettingsInput {
    pub simulator_decks_folder: Option<String>,
    pub backend_api_url: Option<String>,
    pub site_url: Option<String>,
    pub sync_on_simulator_launch: Option<bool>,
}

#[tauri::command]
pub fn save_settings(app: AppHandle, input: SettingsInput) -> Result<AppConfig, String> {
    let mut cfg = config::load(&app);
    if let Some(folder) = input.simulator_decks_folder {
        cfg.simulator_decks_folder = Some(folder);
    }
    if let Some(url) = input.backend_api_url {
        cfg.backend_api_url = url;
    }
    if let Some(url) = input.site_url {
        cfg.site_url = url;
    }
    if let Some(on_launch) = input.sync_on_simulator_launch {
        cfg.sync_on_simulator_launch = on_launch;
    }
    config::save(&app, &cfg)?;
    Ok(cfg)
}

/// Tries to locate the simulator's Decks folder automatically, saving it when
/// found so first-time setup usually needs no manual browsing at all.
#[tauri::command]
pub fn autodetect_folder(app: AppHandle) -> Option<String> {
    let found = simulator::autodetect_decks_folder()?;
    let mut cfg = config::load(&app);
    cfg.simulator_decks_folder = Some(found.clone());
    let _ = config::save(&app, &cfg);
    Some(found)
}

#[tauri::command]
pub fn is_simulator_running() -> bool {
    simulator::is_running()
}

/// Opens the native folder picker and resolves whatever the user chose into
/// the simulator's actual `Decks` folder (see simulator::resolve_decks_folder
/// for what "resolves" covers — install root, `Builds_Windows`, etc.).
///
/// Returns `Ok(None)` if the dialog was cancelled, `Err("decks_folder_not_found")`
/// if the chosen folder has no `Decks` folder in or under it — this is what
/// catches picking the simulator's install root by mistake instead of the
/// `Decks` folder itself, which used to silently write files somewhere the
/// simulator never reads.
#[tauri::command]
pub async fn pick_decks_folder(app: AppHandle) -> Result<Option<String>, String> {
    let (tx, rx) = std::sync::mpsc::channel();
    app.dialog().file().pick_folder(move |path| {
        let _ = tx.send(path);
    });
    let picked = match rx.recv() {
        Ok(Some(path)) => path.into_path().ok().map(|p| p.display().to_string()),
        _ => None,
    };
    let Some(picked) = picked else {
        return Ok(None);
    };
    match simulator::resolve_decks_folder(&picked) {
        Some(resolved) => Ok(Some(resolved)),
        None => Err("decks_folder_not_found".to_string()),
    }
}

#[tauri::command]
pub async fn set_autostart(app: AppHandle, enabled: bool) -> Result<(), String> {
    use tauri_plugin_autostart::ManagerExt;
    let manager = app.autolaunch();
    let result = if enabled {
        manager.enable()
    } else {
        manager.disable()
    };
    result.map_err(|e| e.to_string())?;

    let mut cfg = config::load(&app);
    cfg.autostart_enabled = enabled;
    config::save(&app, &cfg)
}

#[tauri::command]
pub async fn sync_now(app: AppHandle) -> Result<SyncReport, String> {
    let token = credentials::load().ok_or("not_linked")?;
    let mut cfg = config::load(&app);
    let report = sync::run(&mut cfg, &token).await?;
    config::save(&app, &cfg)?;
    Ok(report)
}
