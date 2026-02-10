//! Tauri commands for frontend communication

use crate::aria2;
use crate::config::AppConfig;
use crate::error::Error;
use crate::tray::TrayLabels;
use crate::Result;
use serde_json::Value;
use tauri_plugin_store::StoreExt;

/// Get application configuration
#[tauri::command]
pub async fn get_app_config(app: tauri::AppHandle) -> Result<AppConfig> {
    let store = app.store("config.json")?;

    // Try to load from store, or use defaults
    let config: AppConfig = if let Some(data) = store.get("config") {
        serde_json::from_value(data.clone()).unwrap_or_default()
    } else {
        AppConfig::default()
    };

    Ok(config)
}

/// Save application configuration
#[tauri::command]
pub async fn save_app_config(app: tauri::AppHandle, config: AppConfig) -> Result<()> {
    let store = app.store("config.json")?;
    store.set("config", serde_json::to_value(&config)?);
    store.save()?;
    Ok(())
}

/// Add URI download task
#[tauri::command]
pub async fn add_uri(uris: Vec<String>, options: Option<Value>) -> Result<String> {
    let client = aria2::get_client().await?;
    client.add_uri(uris, options).await
}

/// Add torrent download task
#[tauri::command]
pub async fn add_torrent(torrent: String, options: Option<Value>) -> Result<String> {
    let client = aria2::get_client().await?;
    client.add_torrent(&torrent, options).await
}

/// Pause a task
#[tauri::command]
pub async fn pause_task(gid: String) -> Result<String> {
    let client = aria2::get_client().await?;
    client.pause(&gid).await
}

/// Resume a task
#[tauri::command]
pub async fn resume_task(gid: String) -> Result<String> {
    let client = aria2::get_client().await?;
    client.unpause(&gid).await
}

/// Remove a task
#[tauri::command]
pub async fn remove_task(gid: String) -> Result<String> {
    let client = aria2::get_client().await?;
    client.remove(&gid).await
}

/// Get task list by type
#[tauri::command]
pub async fn get_task_list(task_type: String) -> Result<Value> {
    let client = aria2::get_client().await?;

    match task_type.as_str() {
        "active" => {
            let active = client.tell_active().await?;
            let waiting = client.tell_waiting(0, 100).await?;

            // Merge active and waiting
            let mut tasks = Vec::new();
            if let Some(arr) = active.as_array() {
                tasks.extend(arr.clone());
            }
            if let Some(arr) = waiting.as_array() {
                tasks.extend(arr.clone());
            }
            Ok(serde_json::to_value(tasks)?)
        }
        "waiting" => client.tell_waiting(0, 100).await,
        "stopped" => client.tell_stopped(0, 1000).await,
        _ => client.tell_active().await,
    }
}

/// Get single task info
#[tauri::command]
pub async fn get_task_info(gid: String) -> Result<Value> {
    let client = aria2::get_client().await?;
    client.tell_status(&gid).await
}

/// Get global statistics
#[tauri::command]
pub async fn get_global_stat() -> Result<Value> {
    let client = aria2::get_client().await?;
    client.get_global_stat().await
}

/// Change global options
#[tauri::command]
pub async fn change_global_option(options: Value) -> Result<Value> {
    let client = aria2::get_client().await?;
    client.change_global_option(options).await
}

/// Shutdown aria2 engine
#[tauri::command]
pub async fn shutdown_engine() -> Result<Value> {
    let client = aria2::get_client().await?;
    client.shutdown().await
}

/// Pause all active tasks
#[tauri::command]
pub async fn pause_all_tasks() -> Result<Value> {
    let client = aria2::get_client().await?;
    client.pause_all().await
}

/// Resume all paused tasks
#[tauri::command]
pub async fn resume_all_tasks() -> Result<Value> {
    let client = aria2::get_client().await?;
    client.unpause_all().await
}

/// Remove a single download result record
#[tauri::command]
pub async fn remove_task_record(gid: String) -> Result<Value> {
    let client = aria2::get_client().await?;
    client.remove_download_result(&gid).await
}

/// Purge all completed/error/removed download results
#[tauri::command]
pub async fn purge_task_records() -> Result<Value> {
    let client = aria2::get_client().await?;
    client.purge_download_result().await
}

/// Open file in system default application
#[tauri::command]
pub async fn open_file(path: String) -> Result<()> {
    open::that(&path).map_err(|e| Error::Custom(format!("Failed to open file: {}", e)))?;
    Ok(())
}

/// Show file in system file manager
#[tauri::command]
pub async fn show_in_folder(path: String) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .args(["/select,", &path])
            .spawn()
            .map_err(|e| Error::Custom(format!("Failed to open folder: {}", e)))?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .args(["-R", &path])
            .spawn()
            .map_err(|e| Error::Custom(format!("Failed to open folder: {}", e)))?;
    }
    #[cfg(target_os = "linux")]
    {
        // Try xdg-open on the parent directory
        let parent = std::path::Path::new(&path)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or(path);
        std::process::Command::new("xdg-open")
            .arg(&parent)
            .spawn()
            .map_err(|e| Error::Custom(format!("Failed to open folder: {}", e)))?;
    }
    Ok(())
}

/// Save aria2 session
#[tauri::command]
pub async fn save_session() -> Result<Value> {
    let client = aria2::get_client().await?;
    client.save_session().await
}

/// Force pause a task (required for BT tasks)
#[tauri::command]
pub async fn force_pause_task(gid: String) -> Result<String> {
    let client = aria2::get_client().await?;
    client.force_pause(&gid).await
}

/// Force remove a task
#[tauri::command]
pub async fn force_remove_task(gid: String) -> Result<String> {
    let client = aria2::get_client().await?;
    client.force_remove(&gid).await
}

/// Get aria2 version
#[tauri::command]
pub async fn get_engine_version() -> Result<Value> {
    let client = aria2::get_client().await?;
    client.get_version().await
}

/// Get peers for a BT task
#[tauri::command]
pub async fn get_task_peers(gid: String) -> Result<Value> {
    let client = aria2::get_client().await?;
    client.get_peers(&gid).await
}

/// Change task-specific options
#[tauri::command]
pub async fn change_task_option(gid: String, options: Value) -> Result<Value> {
    let client = aria2::get_client().await?;
    client.change_option(&gid, options).await
}

/// Fetch tracker lists from remote sources
#[tauri::command]
pub async fn fetch_tracker_list(sources: Vec<String>) -> Result<Vec<String>> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| Error::Custom(format!("Failed to create HTTP client: {}", e)))?;

    let mut all_trackers = Vec::new();

    for source in &sources {
        match client.get(source).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    if let Ok(text) = response.text().await {
                        let trackers: Vec<String> = text
                            .lines()
                            .map(|line| line.trim().to_string())
                            .filter(|line| !line.is_empty())
                            .collect();
                        all_trackers.extend(trackers);
                    }
                } else {
                    tracing::warn!("Failed to fetch tracker source {}: HTTP {}", source, response.status());
                }
            }
            Err(e) => {
                tracing::warn!("Failed to fetch tracker source {}: {}", source, e);
            }
        }
    }

    // Deduplicate
    let mut seen = std::collections::HashSet::new();
    all_trackers.retain(|t| seen.insert(t.clone()));

    Ok(all_trackers)
}

/// Update tray menu labels for i18n
#[tauri::command]
pub async fn update_tray_menu(app: tauri::AppHandle, labels: TrayLabels) -> Result<()> {
    crate::tray::update_tray_labels(&app, &labels)
        .map_err(|e| Error::Custom(format!("Failed to update tray menu: {}", e)))?;
    Ok(())
}
