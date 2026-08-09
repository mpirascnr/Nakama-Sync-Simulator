mod backend;
mod commands;
mod config;
mod credentials;
mod deckfile;
mod pairing;
mod simulator;
mod sync;

use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{Emitter, Manager, WindowEvent};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .invoke_handler(tauri::generate_handler![
            commands::link_account,
            commands::is_linked,
            commands::unlink,
            commands::get_settings,
            commands::save_settings,
            commands::pick_decks_folder,
            commands::set_autostart,
            commands::sync_now,
            commands::autodetect_folder,
            commands::is_simulator_running,
            commands::set_tray_labels,
        ])
        .setup(|app| {
            let handle = app.handle().clone();

            // Tray icon: open window / sync now / quit.
            let open_item = MenuItem::with_id(app, "open", "Open", true, None::<&str>)?;
            let sync_item = MenuItem::with_id(app, "sync", "Sync now", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let tray_menu = Menu::with_items(app, &[&open_item, &sync_item, &quit_item])?;
            app.manage(commands::TrayItems {
                open: open_item.clone(),
                sync: sync_item.clone(),
                quit: quit_item.clone(),
            });

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&tray_menu)
                .show_menu_on_left_click(false)
                .on_menu_event(move |app, event| match event.id().as_ref() {
                    "open" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    "sync" => {
                        let app = app.clone();
                        tauri::async_runtime::spawn(async move {
                            let _ = commands::sync_now(app.clone()).await;
                        });
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click {
                        button: tauri::tray::MouseButton::Left,
                        button_state: tauri::tray::MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                })
                .build(app)?;

            // Closing the window minimizes to tray instead of quitting, like OneDrive.
            if let Some(window) = app.get_webview_window("main") {
                let window_for_close = window.clone();
                window.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = window_for_close.hide();
                    }
                });
            }

            // Sync once right away on startup, if there's an account and a
            // folder already configured — covers "I just opened the app" and
            // "the app just auto-started with Windows" without waiting on the
            // simulator or a manual click.
            {
                let handle = handle.clone();
                tauri::async_runtime::spawn(async move {
                    let cfg = config::load(&handle);
                    if cfg.simulator_decks_folder.is_some() && commands::is_linked() {
                        if let Ok(report) = commands::sync_now(handle.clone()).await {
                            let _ = handle.emit("sync-completed", report);
                        }
                    }
                });
            }

            // Background loop, ticking every 5s, watching only for OP TCG
            // Simulator starting up — the other two triggers are the startup
            // sync above and the manual "Sync now" button/tray item.
            tauri::async_runtime::spawn(async move {
                let mut simulator_was_running = simulator::is_running();

                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

                    let simulator_running = simulator::is_running();
                    let just_launched = simulator_running && !simulator_was_running;
                    simulator_was_running = simulator_running;

                    let _ = handle.emit("simulator-status", simulator_running);

                    if !just_launched {
                        continue;
                    }

                    let cfg = config::load(&handle);
                    if cfg.simulator_decks_folder.is_none() || !cfg.sync_on_simulator_launch {
                        continue;
                    }
                    if !commands::is_linked() {
                        continue;
                    }

                    if let Ok(report) = commands::sync_now(handle.clone()).await {
                        let _ = handle.emit("sync-completed", report);
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
