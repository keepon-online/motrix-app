//! Aria2 JSON-RPC client and engine management

use crate::{Error, Result};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::{mpsc, oneshot, Mutex, RwLock};

/// Atomic counter for generating unique RPC request IDs (safe for JSON number precision)
static RPC_ID_COUNTER: AtomicU64 = AtomicU64::new(1);
use tokio_tungstenite::{connect_async, tungstenite::Message};

/// Aria2 event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Aria2Event {
    pub event_type: Aria2EventType,
    pub gid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Aria2EventType {
    DownloadStart,
    DownloadPause,
    DownloadStop,
    DownloadComplete,
    DownloadError,
    BtDownloadComplete,
}

/// Aria2 notification from WebSocket
#[derive(Debug, Deserialize)]
struct Aria2Notification {
    method: String,
    params: Vec<NotificationParam>,
}

#[derive(Debug, Deserialize)]
struct NotificationParam {
    gid: String,
}

/// Aria2 RPC client
pub struct Aria2Client {
    secret: String,
    sender: mpsc::Sender<RpcRequest>,
}

struct RpcRequest {
    method: String,
    params: Vec<Value>,
    response_tx: oneshot::Sender<Result<Value>>,
}

/// Global aria2 client instance
static ARIA2_CLIENT: RwLock<Option<Arc<Aria2Client>>> = RwLock::const_new(None);

/// Global aria2 child process handle (must be kept alive to prevent process from being killed)
static ARIA2_PROCESS: Mutex<Option<tauri_plugin_shell::process::CommandChild>> = Mutex::const_new(None);

/// Initialize aria2 engine
pub async fn init_engine(app: &AppHandle) -> Result<()> {
    use tauri_plugin_store::StoreExt;
    use crate::config::AppConfig;

    // Load full config from store
    let store = app.store("config.json")?;
    let config: AppConfig = if let Some(config_val) = store.get("config") {
        serde_json::from_value(config_val.clone()).unwrap_or_default()
    } else {
        AppConfig::default()
    };

    let port = config.rpc_port;
    let secret = config.rpc_secret.clone();

    // Start aria2 process using config
    start_aria2_process(app, &config).await?;

    // Wait for aria2 to start, retry connection up to 10 times
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
    let client = Arc::new(client.unwrap());

    // Store globally
    let mut guard = ARIA2_CLIENT.write().await;
    *guard = Some(client);

    tracing::info!("Aria2 engine initialized on port {}", port);
    Ok(())
}

/// Start aria2 process
async fn start_aria2_process(app: &AppHandle, config: &crate::config::AppConfig) -> Result<()> {
    use tauri_plugin_shell::ShellExt;

    let shell = app.shell();

    // Ensure session directory exists
    let app_data_dir = app.path()
        .app_data_dir()
        .map_err(|e| Error::Custom(format!("Failed to get app data dir: {}", e)))?;
    std::fs::create_dir_all(&app_data_dir)
        .map_err(|e| Error::Custom(format!("Failed to create app data dir: {}", e)))?;

    let session_path = app_data_dir.join("aria2.session");
    // Create session file if it doesn't exist
    if !session_path.exists() {
        std::fs::File::create(&session_path)
            .map_err(|e| Error::Custom(format!("Failed to create session file: {}", e)))?;
    }

    let dht_path = app_data_dir.join("dht.dat");
    let dht6_path = app_data_dir.join("dht6.dat");

    // Build aria2 arguments from config
    let mut args = config.to_aria2_args();
    // Append session and DHT paths (not part of AppConfig)
    args.extend([
        format!("--save-session={}", session_path.display()),
        format!("--input-file={}", session_path.display()),
        "--save-session-interval=10".to_string(),
        format!("--dht-file-path={}", dht_path.display()),
        format!("--dht-file-path6={}", dht6_path.display()),
    ]);

    // Write sensitive options (proxy password) to a conf file to avoid exposure in process list
    let conf_path = app_data_dir.join("aria2.conf");
    if config.proxy_enabled && !config.proxy_password.is_empty() {
        let conf_content = format!("all-proxy-passwd={}\n", config.proxy_password);
        std::fs::write(&conf_path, conf_content)
            .map_err(|e| Error::Custom(format!("Failed to write aria2 conf: {}", e)))?;
        args.push(format!("--conf-path={}", conf_path.display()));
    } else {
        // Remove stale conf file and disable default conf
        let _ = std::fs::remove_file(&conf_path);
        args.push("--no-conf".to_string());
    }

    // Spawn aria2c process
    let (mut _rx, child) = shell
        .sidecar("aria2c")
        .map_err(|e| Error::Custom(format!("Failed to create aria2c sidecar: {}", e)))?
        .args(&args)
        .spawn()
        .map_err(|e| Error::Custom(format!("Failed to spawn aria2c: {}", e)))?;

    // Store child process globally to keep it alive (dropping CommandChild kills the process)
    let mut process_guard = ARIA2_PROCESS.lock().await;
    *process_guard = Some(child);

    tracing::info!("Aria2 process started");
    Ok(())
}

/// Get the global aria2 client
pub async fn get_client() -> Result<Arc<Aria2Client>> {
    let guard = ARIA2_CLIENT.read().await;
    guard
        .clone()
        .ok_or_else(|| Error::Aria2Rpc("Aria2 client not initialized".to_string()))
}

impl Aria2Client {
    /// Create a new aria2 client
    pub async fn new(app_handle: AppHandle, port: u16, secret: String) -> Result<Self> {
        let url = format!("ws://127.0.0.1:{}/jsonrpc", port);
        let (ws_stream, _) = connect_async(&url)
            .await
            .map_err(|e| Error::WebSocket(e.to_string()))?;

        let (write, read) = ws_stream.split();
        let (tx, mut rx) = mpsc::channel::<RpcRequest>(100);
        let event_app_handle = app_handle.clone();

        // Spawn message handler with reconnection
        let ws_url = url.clone();
        tokio::spawn(async move {
            let mut pending: std::collections::HashMap<u64, oneshot::Sender<Result<Value>>> =
                std::collections::HashMap::new();
            let mut write = write;
            let mut read = read;

            loop {
                tokio::select! {
                    // Handle outgoing requests
                    Some(req) = rx.recv() => {
                        let id = RPC_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
                        let msg = json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "method": req.method,
                            "params": req.params,
                        });

                        pending.insert(id, req.response_tx);

                        if let Err(e) = write.send(Message::Text(msg.to_string())).await {
                            tracing::error!("Failed to send message: {}", e);
                            // Remove pending request and send error back so caller doesn't hang
                            if let Some(tx) = pending.remove(&id) {
                                let _ = tx.send(Err(Error::Aria2Rpc(format!("WebSocket send failed: {}", e))));
                            }
                        }
                    }
                    // Handle incoming responses and notifications
                    Some(msg) = read.next() => {
                        match msg {
                            Ok(Message::Text(text)) => {
                                // Try to parse as RPC response first
                                if let Ok(response) = serde_json::from_str::<RpcResponse>(&text) {
                                    if let Some(tx) = pending.remove(&response.id) {
                                        let result = if let Some(error) = response.error {
                                            Err(Error::Aria2Rpc(error.message))
                                        } else {
                                            Ok(response.result.unwrap_or(Value::Null))
                                        };
                                        let _ = tx.send(result);
                                    }
                                }
                                // Try to parse as notification
                                else if let Ok(notification) = serde_json::from_str::<Aria2Notification>(&text) {
                                    Self::handle_notification(&event_app_handle, notification);
                                }
                            }
                            Err(e) => {
                                tracing::error!("WebSocket error: {}, attempting reconnect...", e);
                                let _ = event_app_handle.emit("aria2-connection", "disconnected");
                                // Fail all pending requests
                                for (_, tx) in pending.drain() {
                                    let _ = tx.send(Err(Error::Aria2Rpc("WebSocket disconnected".to_string())));
                                }
                                // Attempt reconnection with retries
                                let mut reconnected = false;
                                for attempt in 1..=10 {
                                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                                    match connect_async(&ws_url).await {
                                        Ok((new_stream, _)) => {
                                            let (new_write, new_read) = new_stream.split();
                                            write = new_write;
                                            read = new_read;
                                            tracing::info!("WebSocket reconnected on attempt {}", attempt);
                                            let _ = event_app_handle.emit("aria2-connection", "connected");
                                            reconnected = true;
                                            break;
                                        }
                                        Err(e) => {
                                            tracing::warn!("Reconnect attempt {}/10 failed: {}", attempt, e);
                                        }
                                    }
                                }
                                if !reconnected {
                                    tracing::error!("Failed to reconnect after 10 attempts, giving up");
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }
                    else => break,
                }
            }
        });

        Ok(Self {
            secret,
            sender: tx,
        })
    }

    /// Handle aria2 notification events
    fn handle_notification(app_handle: &AppHandle, notification: Aria2Notification) {
        let event_type = match notification.method.as_str() {
            "aria2.onDownloadStart" => Aria2EventType::DownloadStart,
            "aria2.onDownloadPause" => Aria2EventType::DownloadPause,
            "aria2.onDownloadStop" => Aria2EventType::DownloadStop,
            "aria2.onDownloadComplete" => Aria2EventType::DownloadComplete,
            "aria2.onDownloadError" => Aria2EventType::DownloadError,
            "aria2.onBtDownloadComplete" => Aria2EventType::BtDownloadComplete,
            _ => {
                tracing::debug!("Unknown aria2 notification: {}", notification.method);
                return;
            }
        };

        if let Some(param) = notification.params.first() {
            let event = Aria2Event {
                event_type: event_type.clone(),
                gid: param.gid.clone(),
            };

            tracing::info!("Aria2 event: {:?} for gid {}", event_type, param.gid);

            // Emit event to frontend
            if let Err(e) = app_handle.emit("aria2-event", &event) {
                tracing::error!("Failed to emit aria2 event: {}", e);
            }

            // Notifications are handled by the frontend (useAria2Events) with i18n support
        }
    }


    /// Call an aria2 RPC method
    pub async fn call(&self, method: &str, params: Vec<Value>) -> Result<Value> {
        let (tx, rx) = oneshot::channel();

        // Add secret token to params
        let mut full_params = vec![json!(format!("token:{}", self.secret))];
        full_params.extend(params);

        let request = RpcRequest {
            method: format!("aria2.{}", method),
            params: full_params,
            response_tx: tx,
        };

        self.sender
            .send(request)
            .await
            .map_err(|_| Error::Aria2Rpc("Failed to send request".to_string()))?;

        // Add timeout to prevent hanging forever if WebSocket connection is broken
        match tokio::time::timeout(tokio::time::Duration::from_secs(30), rx).await {
            Ok(result) => result.map_err(|_| Error::Aria2Rpc("Failed to receive response".to_string()))?,
            Err(_) => Err(Error::Aria2Rpc("RPC call timed out".to_string())),
        }
    }

    /// Add URI download
    pub async fn add_uri(&self, uris: Vec<String>, options: Option<Value>) -> Result<String> {
        let mut params = vec![json!(uris)];
        if let Some(opts) = options {
            params.push(opts);
        }
        let result = self.call("addUri", params).await?;
        result
            .as_str()
            .map(String::from)
            .ok_or_else(|| Error::Aria2Rpc("Invalid response".to_string()))
    }

    /// Add torrent download
    pub async fn add_torrent(&self, torrent: &str, options: Option<Value>) -> Result<String> {
        let mut params = vec![json!(torrent)];
        params.push(json!([])); // uris
        if let Some(opts) = options {
            params.push(opts);
        }
        let result = self.call("addTorrent", params).await?;
        result
            .as_str()
            .map(String::from)
            .ok_or_else(|| Error::Aria2Rpc("Invalid response".to_string()))
    }

    /// Pause task
    pub async fn pause(&self, gid: &str) -> Result<String> {
        let result = self.call("pause", vec![json!(gid)]).await?;
        result
            .as_str()
            .map(String::from)
            .ok_or_else(|| Error::Aria2Rpc("Invalid response".to_string()))
    }

    /// Resume task
    pub async fn unpause(&self, gid: &str) -> Result<String> {
        let result = self.call("unpause", vec![json!(gid)]).await?;
        result
            .as_str()
            .map(String::from)
            .ok_or_else(|| Error::Aria2Rpc("Invalid response".to_string()))
    }

    /// Remove task
    pub async fn remove(&self, gid: &str) -> Result<String> {
        let result = self.call("remove", vec![json!(gid)]).await?;
        result
            .as_str()
            .map(String::from)
            .ok_or_else(|| Error::Aria2Rpc("Invalid response".to_string()))
    }

    /// Get task status
    pub async fn tell_status(&self, gid: &str) -> Result<Value> {
        self.call("tellStatus", vec![json!(gid)]).await
    }

    /// Get active tasks
    pub async fn tell_active(&self) -> Result<Value> {
        self.call("tellActive", vec![]).await
    }

    /// Get waiting tasks
    pub async fn tell_waiting(&self, offset: i32, num: i32) -> Result<Value> {
        self.call("tellWaiting", vec![json!(offset), json!(num)])
            .await
    }

    /// Get stopped tasks
    pub async fn tell_stopped(&self, offset: i32, num: i32) -> Result<Value> {
        self.call("tellStopped", vec![json!(offset), json!(num)])
            .await
    }

    /// Get global statistics
    pub async fn get_global_stat(&self) -> Result<Value> {
        self.call("getGlobalStat", vec![]).await
    }

    /// Change global options
    pub async fn change_global_option(&self, options: Value) -> Result<Value> {
        self.call("changeGlobalOption", vec![options]).await
    }

    /// Pause all active tasks
    pub async fn pause_all(&self) -> Result<Value> {
        self.call("pauseAll", vec![]).await
    }

    /// Resume all paused tasks
    pub async fn unpause_all(&self) -> Result<Value> {
        self.call("unpauseAll", vec![]).await
    }

    /// Remove download result (clear completed/error record)
    pub async fn remove_download_result(&self, gid: &str) -> Result<Value> {
        self.call("removeDownloadResult", vec![json!(gid)]).await
    }

    /// Purge all completed/error/removed download results
    pub async fn purge_download_result(&self) -> Result<Value> {
        self.call("purgeDownloadResult", vec![]).await
    }

    /// Save session to file
    pub async fn save_session(&self) -> Result<Value> {
        self.call("saveSession", vec![]).await
    }

    /// Force pause a task (needed for BT tasks)
    pub async fn force_pause(&self, gid: &str) -> Result<String> {
        let result = self.call("forcePause", vec![json!(gid)]).await?;
        result
            .as_str()
            .map(String::from)
            .ok_or_else(|| Error::Aria2Rpc("Invalid response".to_string()))
    }

    /// Force remove a task
    pub async fn force_remove(&self, gid: &str) -> Result<String> {
        let result = self.call("forceRemove", vec![json!(gid)]).await?;
        result
            .as_str()
            .map(String::from)
            .ok_or_else(|| Error::Aria2Rpc("Invalid response".to_string()))
    }

    /// Get aria2 version info
    pub async fn get_version(&self) -> Result<Value> {
        self.call("getVersion", vec![]).await
    }

    /// Get peers for a BT task
    pub async fn get_peers(&self, gid: &str) -> Result<Value> {
        self.call("getPeers", vec![json!(gid)]).await
    }

    /// Change task-specific options
    pub async fn change_option(&self, gid: &str, options: Value) -> Result<Value> {
        self.call("changeOption", vec![json!(gid), options]).await
    }

    /// Shutdown aria2
    pub async fn shutdown(&self) -> Result<Value> {
        // Save session before shutdown
        let _ = self.save_session().await;
        self.call("shutdown", vec![]).await
    }
}

#[derive(Debug, Deserialize)]
struct RpcResponse {
    id: u64,
    result: Option<Value>,
    error: Option<RpcError>,
}

#[derive(Debug, Deserialize)]
struct RpcError {
    #[allow(dead_code)]
    code: i32,
    message: String,
}
