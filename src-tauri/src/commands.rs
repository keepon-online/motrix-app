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

/// Add torrent download task from file path (reads and base64-encodes on backend)
#[tauri::command]
pub async fn add_torrent_file(file_path: String, options: Option<Value>) -> Result<String> {
    use base64::Engine;
    let data = std::fs::read(&file_path)
        .map_err(|e| Error::Custom(format!("Failed to read torrent file: {}", e)))?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&data);
    let client = aria2::get_client().await?;
    client.add_torrent(&b64, options).await
}

/// Add metalink download task from file path
#[tauri::command]
pub async fn add_metalink_file(file_path: String, options: Option<Value>) -> Result<Value> {
    use base64::Engine;
    let data = std::fs::read(&file_path)
        .map_err(|e| Error::Custom(format!("Failed to read metalink file: {}", e)))?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&data);
    let client = aria2::get_client().await?;
    client.add_metalink(&b64, options).await
}

/// Add metalink download task from base64 data (for drag-drop)
#[tauri::command]
pub async fn add_metalink_file_base64(metalink: String, options: Option<Value>) -> Result<Value> {
    let client = aria2::get_client().await?;
    client.add_metalink(&metalink, options).await
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
            let waiting = client.tell_waiting(0, 1000).await?;

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
        "waiting" => client.tell_waiting(0, 1000).await,
        "stopped" => client.tell_stopped(0, 10000).await,
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

/// Delete local files for a task (restricted to download directory)
#[tauri::command]
pub async fn delete_task_files(app: tauri::AppHandle, file_paths: Vec<String>) -> Result<()> {
    // Get download directory from config for path validation
    let download_dir = {
        let store = app.store("config.json")?;
        let config: AppConfig = if let Some(data) = store.get("config") {
            serde_json::from_value(data.clone()).unwrap_or_default()
        } else {
            AppConfig::default()
        };
        config.download_dir
    };

    for path in &file_paths {
        let p = std::path::Path::new(path);
        // Resolve to canonical path to prevent path traversal
        let canonical = p.canonicalize()
            .map_err(|e| Error::Custom(format!("Invalid path {}: {}", path, e)))?;
        let dir_canonical = download_dir.canonicalize().unwrap_or(download_dir.clone());

        if !canonical.starts_with(&dir_canonical) {
            return Err(Error::Custom(format!("Path {} is outside download directory", path)));
        }

        if canonical.exists() {
            if canonical.is_dir() {
                std::fs::remove_dir_all(&canonical)
                    .map_err(|e| Error::Custom(format!("Failed to delete directory {}: {}", path, e)))?;
            } else {
                std::fs::remove_file(&canonical)
                    .map_err(|e| Error::Custom(format!("Failed to delete file {}: {}", path, e)))?;
            }
        }
    }
    Ok(())
}

/// Parse a torrent file and return file list info
#[tauri::command]
pub async fn parse_torrent_file(file_path: String) -> Result<Value> {
    let data = std::fs::read(&file_path)
        .map_err(|e| Error::Custom(format!("Failed to read torrent file: {}", e)))?;

    let torrent: serde_bencode::value::Value = serde_bencode::from_bytes(&data)
        .map_err(|e| Error::Custom(format!("Failed to parse torrent: {}", e)))?;

    let dict = match &torrent {
        serde_bencode::value::Value::Dict(d) => d,
        _ => return Err(Error::Custom("Invalid torrent format".to_string())),
    };

    let info = dict.get(&b"info".to_vec())
        .ok_or_else(|| Error::Custom("Missing info dictionary".to_string()))?;

    let info_dict = match info {
        serde_bencode::value::Value::Dict(d) => d,
        _ => return Err(Error::Custom("Invalid info dictionary".to_string())),
    };

    let name = info_dict.get(&b"name".to_vec())
        .and_then(|v| match v {
            serde_bencode::value::Value::Bytes(b) => String::from_utf8(b.clone()).ok(),
            _ => None,
        })
        .unwrap_or_default();

    let comment = dict.get(&b"comment".to_vec())
        .and_then(|v| match v {
            serde_bencode::value::Value::Bytes(b) => String::from_utf8(b.clone()).ok(),
            _ => None,
        })
        .unwrap_or_default();

    let mut files = Vec::new();

    if let Some(files_val) = info_dict.get(&b"files".to_vec()) {
        // Multi-file torrent
        if let serde_bencode::value::Value::List(file_list) = files_val {
            for (index, file_entry) in file_list.iter().enumerate() {
                if let serde_bencode::value::Value::Dict(file_dict) = file_entry {
                    let length = file_dict.get(&b"length".to_vec())
                        .and_then(|v| match v {
                            serde_bencode::value::Value::Int(n) => Some(*n),
                            _ => None,
                        })
                        .unwrap_or(0);

                    let path = file_dict.get(&b"path".to_vec())
                        .and_then(|v| match v {
                            serde_bencode::value::Value::List(parts) => {
                                let path_parts: Vec<String> = parts.iter()
                                    .filter_map(|p| match p {
                                        serde_bencode::value::Value::Bytes(b) => String::from_utf8(b.clone()).ok(),
                                        _ => None,
                                    })
                                    .collect();
                                Some(path_parts.join("/"))
                            }
                            _ => None,
                        })
                        .unwrap_or_default();

                    files.push(serde_json::json!({
                        "index": index,
                        "path": path,
                        "length": length,
                    }));
                }
            }
        }
    } else {
        // Single file torrent
        let length = info_dict.get(&b"length".to_vec())
            .and_then(|v| match v {
                serde_bencode::value::Value::Int(n) => Some(*n),
                _ => None,
            })
            .unwrap_or(0);

        files.push(serde_json::json!({
            "index": 0,
            "path": name.clone(),
            "length": length,
        }));
    }

    Ok(serde_json::json!({
        "name": name,
        "comment": comment,
        "files": files,
    }))
}

/// Update tray menu labels for i18n
#[tauri::command]
pub async fn update_tray_menu(app: tauri::AppHandle, labels: TrayLabels) -> Result<()> {
    crate::tray::update_tray_labels(&app, &labels)
        .map_err(|e| Error::Custom(format!("Failed to update tray menu: {}", e)))?;
    Ok(())
}

/// Prevent system sleep (during active downloads)
#[tauri::command]
pub async fn prevent_sleep() -> Result<()> {
    crate::power::prevent_sleep()
        .map_err(|e| Error::Custom(format!("Failed to prevent sleep: {}", e)))?;
    Ok(())
}

/// Allow system sleep (when no active downloads)
#[tauri::command]
pub async fn allow_sleep() -> Result<()> {
    crate::power::allow_sleep()
        .map_err(|e| Error::Custom(format!("Failed to allow sleep: {}", e)))?;
    Ok(())
}

/// Change task position in the waiting queue
#[tauri::command]
pub async fn change_task_position(gid: String, pos: i32, how: String) -> Result<Value> {
    let client = aria2::get_client().await?;
    client.change_position(&gid, pos, &how).await
}
