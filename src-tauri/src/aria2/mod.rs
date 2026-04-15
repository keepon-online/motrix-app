//! Aria2 engine management, RPC client, and event handling
//!
//! This module is organized into:
//! - `client` — Aria2Client and RPC communication
//! - `events` — Event types and notification handling
//! - Engine lifecycle functions (this file)

pub mod client;
pub mod events;

pub use client::Aria2Client;
pub use events::{Aria2Event, Aria2EventType};

use crate::{Error, Result};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::{Mutex, RwLock};

/// Global aria2 client instance
static ARIA2_CLIENT: RwLock<Option<Arc<Aria2Client>>> = RwLock::const_new(None);

/// Global aria2 child process handle (must be kept alive to prevent process from being killed)
static ARIA2_PROCESS: Mutex<Option<tauri_plugin_shell::process::CommandChild>> = Mutex::const_new(None);

/// True when the engine is being intentionally shut down
static SHUTTING_DOWN: AtomicBool = AtomicBool::new(false);

/// Kill any orphaned aria2c processes listening on the given port
fn kill_orphaned_aria2c(port: u16) {
    #[cfg(target_os = "windows")]
    {
        let output = std::process::Command::new("cmd")
            .args(["/C", &format!("netstat -ano | findstr \"LISTENING\" | findstr \":{port}\"")])
            .output();
        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if let Some(pid_str) = line.split_whitespace().last() {
                    if let Ok(pid) = pid_str.parse::<u32>() {
                        if pid > 0 {
                            tracing::info!("Killing orphaned process on port {} (PID: {})", port, pid);
                            let _ = std::process::Command::new("taskkill")
                                .args(["/F", "/PID", &pid.to_string()])
                                .output();
                        }
                    }
                }
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let output = std::process::Command::new("lsof")
            .args(["-ti", &format!(":{port}")])
            .output();
        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for pid_str in stdout.split_whitespace() {
                if let Ok(pid) = pid_str.parse::<u32>() {
                    if pid > 0 {
                        tracing::info!("Killing orphaned process on port {} (PID: {})", port, pid);
                        let _ = std::process::Command::new("kill")
                            .args(["-9", &pid.to_string()])
                            .output();
                    }
                }
            }
        }
    }
}

/// Force kill the managed aria2c child process
pub(crate) async fn force_kill_process() {
    let mut guard = ARIA2_PROCESS.lock().await;
    if let Some(child) = guard.take() {
        tracing::info!("Force killing aria2c child process");
        let _ = child.kill();
    }
}

/// Graceful shutdown: try RPC shutdown first, then force kill as fallback
pub async fn shutdown_and_cleanup() {
    SHUTTING_DOWN.store(true, Ordering::SeqCst);

    // Mark the client as shutting down so the reconnection loop stops
    if let Ok(client) = get_client().await {
        client.mark_shutdown();
        let _ = client.save_session().await;
        let rpc_ok = client.shutdown().await.is_ok();
        if !rpc_ok {
            tracing::warn!("RPC shutdown failed, force killing aria2c process");
            force_kill_process().await;
        }
    } else {
        force_kill_process().await;
    }

    let mut guard = ARIA2_CLIENT.write().await;
    *guard = None;
}

/// Initialize aria2 engine
pub async fn init_engine(app: &AppHandle) -> Result<()> {
    use tauri_plugin_store::StoreExt;
    use crate::config::AppConfig;

    SHUTTING_DOWN.store(false, Ordering::SeqCst);

    let store = app.store("config.json")?;
    let config = AppConfig::load_from_store(&store);

    let port = config.rpc_port;
    let secret = config.rpc_secret.clone();

    kill_orphaned_aria2c(port);
    start_aria2_process(app, &config).await?;

    let mut client = None;
    for attempt in 1..=10 {
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        match Aria2Client::new(app.clone(), port, secret.clone()).await {
            Ok(c) => {
                client = Some(c);
                break;
            }
            Err(e) => {
                tracing::warn!("Aria2 connection attempt {}/10 failed: {}", attempt, e);
                if attempt == 10 {
                    return Err(e);
                }
            }
        }
    }
    let client = Arc::new(client.expect("aria2 client should be initialized after retry loop"));

    let mut guard = ARIA2_CLIENT.write().await;
    *guard = Some(client);

    tracing::info!("Aria2 engine initialized on port {}", port);
    Ok(())
}

/// Start aria2 process and spawn a watchdog to detect unexpected termination
async fn start_aria2_process(app: &AppHandle, config: &crate::config::AppConfig) -> Result<()> {
    use tauri_plugin_shell::ShellExt;

    let shell = app.shell();

    let app_data_dir = app.path()
        .app_data_dir()
        .map_err(|e| Error::Custom(format!("Failed to get app data dir: {}", e)))?;
    std::fs::create_dir_all(&app_data_dir)
        .map_err(|e| Error::Custom(format!("Failed to create app data dir: {}", e)))?;

    let session_path = app_data_dir.join("aria2.session");
    if !session_path.exists() {
        std::fs::File::create(&session_path)
            .map_err(|e| Error::Custom(format!("Failed to create session file: {}", e)))?;
    }

    let dht_path = app_data_dir.join("dht.dat");
    let dht6_path = app_data_dir.join("dht6.dat");

    let mut args = config.to_aria2_args();
    args.extend([
        format!("--save-session={}", session_path.display()),
        format!("--input-file={}", session_path.display()),
        "--save-session-interval=10".to_string(),
        format!("--dht-file-path={}", dht_path.display()),
        format!("--dht-file-path6={}", dht6_path.display()),
    ]);

    // Write sensitive options (rpc-secret, proxy password) to a conf file to avoid
    // exposure in process list (visible via `ps aux`, Task Manager, etc.)
    let conf_path = app_data_dir.join("aria2.conf");
    let mut conf_lines: Vec<String> = Vec::new();

    conf_lines.push(format!("rpc-secret={}", config.rpc_secret));

    if config.proxy_enabled && !config.proxy_password.is_empty() {
        conf_lines.push(format!("all-proxy-passwd={}", config.proxy_password));
    }

    let conf_content = conf_lines.join("\n");
    std::fs::write(&conf_path, conf_content)
        .map_err(|e| Error::Custom(format!("Failed to write aria2 conf: {}", e)))?;
    args.push(format!("--conf-path={}", conf_path.display()));

    let (rx, child) = shell
        .sidecar("aria2c")
        .map_err(|e| Error::Custom(format!("Failed to create aria2c sidecar: {}", e)))?
        .args(&args)
        .spawn()
        .map_err(|e| Error::Custom(format!("Failed to spawn aria2c: {}", e)))?;

    let mut process_guard = ARIA2_PROCESS.lock().await;
    *process_guard = Some(child);

    // Spawn process watchdog to detect unexpected termination
    let watchdog_app = app.clone();
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                tauri_plugin_shell::process::CommandEvent::Terminated(payload) => {
                    if SHUTTING_DOWN.load(Ordering::SeqCst) {
                        tracing::info!("Aria2 process terminated (intentional shutdown)");
                        return;
                    }
                    let code = payload.code.map(|c| c.to_string()).unwrap_or_else(|| "unknown".into());
                    tracing::warn!("Aria2 process terminated unexpectedly (exit code: {})", code);
                    let _ = watchdog_app.emit("aria2-connection", "terminated");
                }
                tauri_plugin_shell::process::CommandEvent::Stderr(line) => {
                    let text = String::from_utf8_lossy(&line);
                    tracing::debug!("aria2c stderr: {}", text.trim_end());
                }
                _ => {}
            }
        }
    });

    tracing::info!("Aria2 process started");
    Ok(())
}

/// Get the global aria2 client
pub async fn get_client() -> Result<Arc<Aria2Client>> {
    let guard = ARIA2_CLIENT.read().await;
    guard
        .clone()
        .ok_or_else(|| Error::Aria2Rpc("Aria2 client not initialized".into()))
}
