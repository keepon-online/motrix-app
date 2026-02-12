// Motrix - A full-featured download manager
// Built with Tauri + Vue 3

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use motrix_lib::{aria2, cli, commands, menu, tray};
use tauri::{Emitter, Manager};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() {
    // Check for portable mode (marker file next to executable)
    let portable_data_dir = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|p| p.to_path_buf()))
        .and_then(|dir| {
            if dir.join("portable").exists() || dir.join(".portable").exists() {
                Some(dir.join("data"))
            } else {
                None
            }
        });

    // Determine log directory early (before Tauri app setup)
    let log_dir = if let Some(ref portable) = portable_data_dir {
        portable.join("logs")
    } else {
        dirs::data_dir()
            .map(|d| d.join("app.motrix.native").join("logs"))
            .unwrap_or_else(|| std::path::PathBuf::from("logs"))
    };
    let _ = std::fs::create_dir_all(&log_dir);

    // Create rolling daily file appender
    let file_appender = tracing_appender::rolling::daily(&log_dir, "motrix.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // Initialize logging with both console and file output
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "motrix=debug,tauri=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false),
        )
        .init();

    tracing::info!("Starting Motrix...");
    tracing::info!("Log directory: {}", log_dir.display());

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
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .setup(move |app| {
            // Store log dir path for the get_log_path command
            let app_log_dir = app.path().app_log_dir().unwrap_or_else(|_| log_dir.clone());
            app.manage(commands::LogDir(app_log_dir));

            // Store portable data dir for aria2 and other commands
            app.manage(commands::PortableDataDir(portable_data_dir.clone()));

            // Initialize tray
            tray::create_tray(app)?;

            // Build application menu
            menu::build_menu(app)?;

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
            commands::prevent_sleep,
            commands::allow_sleep,
            commands::change_task_position,
            commands::get_log_path,
            commands::get_app_data_paths,
            commands::clear_session,
            commands::factory_reset,
            commands::set_window_progress,
            commands::set_dock_badge,
            commands::bounce_dock,
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                use tauri_plugin_store::StoreExt;

                // Read runMode config (standard / tray / hide_tray)
                let run_mode = window.app_handle()
                    .store("config.json")
                    .ok()
                    .and_then(|store| store.get("config"))
                    .and_then(|config_val| config_val.get("runMode").and_then(|v| v.as_str().map(String::from)))
                    .unwrap_or_else(|| "tray".to_string());

                match run_mode.as_str() {
                    "standard" => {
                        // Standard mode: just close (default behavior)
                    }
                    "hide_tray" => {
                        // Hide to tray without tray icon visible
                        api.prevent_close();
                        let _ = window.hide();
                        tracing::info!("Window hidden (hide_tray mode)");
                    }
                    _ => {
                        // "tray" mode (default): hide to tray
                        api.prevent_close();
                        let _ = window.hide();
                        tracing::info!("Window hidden to tray");
                    }
                }
            }
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| {
            if let tauri::RunEvent::Exit = event {
                // Ensure aria2c is cleaned up on any exit path
                tauri::async_runtime::block_on(async {
                    aria2::shutdown_and_cleanup().await;
                });
            }
        });
}
