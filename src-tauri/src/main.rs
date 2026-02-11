// Motrix - A full-featured download manager
// Built with Tauri + Vue 3

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use motrix_lib::{aria2, cli, commands, tray};
use tauri::{Emitter, Manager};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "motrix=debug,tauri=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Motrix...");

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            tracing::info!("Second instance detected with argv: {:?}", argv);

            // Focus existing window
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
                let _ = window.unminimize();
            }

            // Pass argv URLs/files to frontend
            let urls = cli::parse_args(&argv);
            if !urls.is_empty() {
                let _ = app.emit("open-urls", &urls);
            }
        }))
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, None))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            // Initialize tray
            tray::create_tray(app)?;

            // Hide window on startup if configured
            {
                use tauri_plugin_store::StoreExt;
                let start_hidden = app.handle()
                    .store("config.json")
                    .ok()
                    .and_then(|store| store.get("config"))
                    .and_then(|v| v.get("startHidden").and_then(|v| v.as_bool()))
                    .unwrap_or(false);

                if start_hidden {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.hide();
                        tracing::info!("Window hidden on startup (startHidden=true)");
                    }
                }
            }

            // Parse command line arguments from first launch
            let args: Vec<String> = std::env::args().collect();
            let urls = cli::parse_args(&args);
            if !urls.is_empty() {
                let app_handle_cli = app.handle().clone();
                let urls_clone = urls.clone();
                tauri::async_runtime::spawn(async move {
                    // Delay to wait for frontend to be ready
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    let _ = app_handle_cli.emit("open-urls", &urls_clone);
                });
            }

            // Initialize aria2 engine
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = aria2::init_engine(&app_handle).await {
                    tracing::error!("Failed to initialize aria2 engine: {}", e);
                    return;
                }

                // Auto-resume all tasks on launch if configured
                {
                    use tauri_plugin_store::StoreExt;
                    let should_resume = app_handle
                        .store("config.json")
                        .ok()
                        .and_then(|store| store.get("config"))
                        .and_then(|v| v.get("resumeAllWhenAppLaunched").and_then(|v| v.as_bool()))
                        .unwrap_or(true);

                    if should_resume {
                        if let Ok(client) = aria2::get_client().await {
                            match client.unpause_all().await {
                                Ok(_) => tracing::info!("Auto-resumed all tasks on launch"),
                                Err(e) => tracing::warn!("Failed to auto-resume tasks: {}", e),
                            }
                        }
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_app_config,
            commands::save_app_config,
            commands::add_uri,
            commands::add_torrent,
            commands::add_torrent_file,
            commands::add_metalink_file,
            commands::add_metalink_file_base64,
            commands::pause_task,
            commands::resume_task,
            commands::remove_task,
            commands::get_task_list,
            commands::get_task_info,
            commands::get_global_stat,
            commands::change_global_option,
            commands::shutdown_engine,
            commands::pause_all_tasks,
            commands::resume_all_tasks,
            commands::remove_task_record,
            commands::purge_task_records,
            commands::open_file,
            commands::show_in_folder,
            commands::save_session,
            commands::force_pause_task,
            commands::force_remove_task,
            commands::get_engine_version,
            commands::get_task_peers,
            commands::change_task_option,
            commands::fetch_tracker_list,
            commands::update_tray_menu,
            commands::delete_task_files,
            commands::parse_torrent_file,
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                use tauri_plugin_store::StoreExt;

                // Read hideOnClose config
                let hide_on_close = window.app_handle()
                    .store("config.json")
                    .ok()
                    .and_then(|store| store.get("config"))
                    .and_then(|config_val| config_val.get("hideOnClose").and_then(|v| v.as_bool()))
                    .unwrap_or(true);

                if hide_on_close {
                    api.prevent_close();
                    let _ = window.hide();
                    tracing::info!("Window hidden to tray");
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
