// Motrix - A full-featured download manager
// Built with Tauri + Vue 3

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use motrix_lib::{aria2, commands, tray};
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
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            // Initialize tray
            tray::create_tray(app)?;

            // Initialize aria2 engine
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = aria2::init_engine(&app_handle).await {
                    tracing::error!("Failed to initialize aria2 engine: {}", e);
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_app_config,
            commands::save_app_config,
            commands::add_uri,
            commands::add_torrent,
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
