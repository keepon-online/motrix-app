//! Aria2 JSON-RPC client with health tracking and automatic recovery

use crate::{Error, Result};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::{connect_async, tungstenite::Message};

use super::events::{handle_notification, Aria2Notification};

/// Atomic counter for generating unique RPC request IDs
static RPC_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Connection state constants emitted via `aria2-connection` event
const CONN_CONNECTED: &str = "connected";
const CONN_DISCONNECTED: &str = "disconnected";

/// Aria2 RPC client with health tracking and automatic reconnection
pub struct Aria2Client {
    secret: String,
    sender: mpsc::Sender<RpcRequest>,
    /// True when WebSocket connection is active and responding
    healthy: Arc<AtomicBool>,
    /// False only when intentionally shut down via `mark_shutdown()`
    alive: Arc<AtomicBool>,
}

struct RpcRequest {
    method: String,
    params: Vec<Value>,
    response_tx: oneshot::Sender<Result<Value>>,
}

impl Aria2Client {
    /// Check if the WebSocket connection is healthy
    pub fn is_healthy(&self) -> bool {
        self.healthy.load(Ordering::Acquire)
    }

    /// Check if the client is still alive (not intentionally shut down)
    pub fn is_alive(&self) -> bool {
        self.alive.load(Ordering::Acquire)
    }

    /// Mark the client as intentionally shutting down.
    /// This prevents the reconnection loop from retrying.
    pub fn mark_shutdown(&self) {
        self.alive.store(false, Ordering::Release);
    }

    /// Create a new aria2 client connected to the given WebSocket URL
    pub async fn new(app_handle: AppHandle, port: u16, secret: String) -> Result<Self> {
        let url = format!("ws://127.0.0.1:{}/jsonrpc", port);
        let (ws_stream, _) = connect_async(&url)
            .await
            .map_err(|e| Error::WebSocket(e.to_string()))?;

        let (write, read) = ws_stream.split();
        let (tx, mut rx) = mpsc::channel::<RpcRequest>(100);
        let healthy = Arc::new(AtomicBool::new(true));
        let alive = Arc::new(AtomicBool::new(true));

        let task_healthy = healthy.clone();
        let task_alive = alive.clone();
        let task_secret = secret.clone();
        let event_app_handle = app_handle.clone();
        let ws_url = url;

        // Spawn the main message loop with reconnection and heartbeat
        tokio::spawn(async move {
            let mut pending: HashMap<u64, oneshot::Sender<Result<Value>>> = HashMap::new();
            let mut write = write;
            let mut read = read;
            let mut last_activity = std::time::Instant::now();

            // Heartbeat timer: tick every 30s to detect stale connections
            let mut heartbeat = tokio::time::interval_at(
                tokio::time::Instant::now() + Duration::from_secs(30),
                Duration::from_secs(30),
            );
            heartbeat.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

            loop {
                // Check for intentional shutdown
                if !task_alive.load(Ordering::Acquire) {
                    for (_, tx) in pending.drain() {
                        let _ = tx.send(Err(Error::Aria2Rpc("Engine shutting down".into())));
                    }
                    break;
                }

                // --- Unhealthy: reconnection mode ---
                if !task_healthy.load(Ordering::Acquire) {
                    // Drain queued requests with errors (call() fast-fails, but some
                    // may have been queued before the unhealthy flag was set)
                    while let Ok(req) = rx.try_recv() {
                        let _ = req.response_tx.send(Err(Error::Aria2Rpc(
                            "Download engine disconnected, reconnecting...".into(),
                        )));
                    }

                    // Wait before retrying, but also check for shutdown periodically
                    tokio::select! {
                        _ = tokio::time::sleep(Duration::from_secs(3)) => {}
                        _ = tokio::time::sleep(Duration::from_millis(200)) => {
                            if !task_alive.load(Ordering::Acquire) { break; }
                            continue;
                        }
                    }

                    match connect_async(&ws_url).await {
                        Ok((new_stream, _)) => {
                            let (new_write, new_read) = new_stream.split();
                            write = new_write;
                            read = new_read;
                            task_healthy.store(true, Ordering::Release);
                            last_activity = std::time::Instant::now();
                            tracing::info!("WebSocket reconnected successfully");
                            let _ = event_app_handle.emit("aria2-connection", CONN_CONNECTED);
                        }
                        Err(e) => {
                            tracing::debug!("Reconnect attempt failed: {}", e);
                        }
                    }
                    continue;
                }

                // --- Healthy: normal operation with heartbeat ---
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
                            if let Some(tx) = pending.remove(&id) {
                                let _ = tx.send(Err(Error::Aria2Rpc(format!("WebSocket send failed: {}", e))));
                            }
                        }
                    }
                    // Handle incoming responses and notifications
                    Some(msg) = read.next() => {
                        match msg {
                            Ok(Message::Text(text)) => {
                                last_activity = std::time::Instant::now();
                                if let Ok(response) = serde_json::from_str::<RpcResponse>(&text) {
                                    if let Some(tx) = pending.remove(&response.id) {
                                        let result = if let Some(error) = response.error {
                                            Err(Error::Aria2Rpc(error.message))
                                        } else {
                                            Ok(response.result.unwrap_or(Value::Null))
                                        };
                                        let _ = tx.send(result);
                                    }
                                } else if let Ok(notification) = serde_json::from_str::<Aria2Notification>(&text) {
                                    handle_notification(&event_app_handle, notification);
                                }
                            }
                            Err(e) => {
                                tracing::error!("WebSocket error: {}, entering reconnection mode", e);
                                task_healthy.store(false, Ordering::Release);
                                let _ = event_app_handle.emit("aria2-connection", CONN_DISCONNECTED);
                                for (_, tx) in pending.drain() {
                                    let _ = tx.send(Err(Error::Aria2Rpc("WebSocket disconnected".into())));
                                }
                            }
                            _ => {}
                        }
                    }
                    // Heartbeat: detect stale connections
                    _ = heartbeat.tick() => {
                        if last_activity.elapsed() > Duration::from_secs(30) {
                            let id = RPC_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
                            let msg = json!({
                                "jsonrpc": "2.0",
                                "id": id,
                                "method": "aria2.getVersion",
                                "params": [json!(format!("token:{}", task_secret))],
                            });
                            if let Err(e) = write.send(Message::Text(msg.to_string())).await {
                                tracing::warn!("Heartbeat failed: {}, marking unhealthy", e);
                                task_healthy.store(false, Ordering::Release);
                                let _ = event_app_handle.emit("aria2-connection", CONN_DISCONNECTED);
                                for (_, tx) in pending.drain() {
                                    let _ = tx.send(Err(Error::Aria2Rpc("Heartbeat failed".into())));
                                }
                            } else {
                                // Fire-and-forget: track in pending so response is consumed
                                let (tx, _) = oneshot::channel();
                                pending.insert(id, tx);
                            }
                        }
                    }
                    else => break,
                }
            }
        });

        Ok(Self {
            secret,
            sender: tx,
            healthy,
            alive,
        })
    }

    /// Call an aria2 RPC method. Fast-fails if the connection is unhealthy.
    pub async fn call(&self, method: &str, params: Vec<Value>) -> Result<Value> {
        if !self.healthy.load(Ordering::Acquire) {
            return Err(Error::Aria2Rpc(
                "Download engine disconnected, reconnecting...".into(),
            ));
        }
        if !self.alive.load(Ordering::Acquire) {
            return Err(Error::Aria2Rpc("Download engine is shutting down".into()));
        }

        let (tx, rx) = oneshot::channel();

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
            .map_err(|_| Error::Aria2Rpc("Failed to send request — engine task exited".into()))?;

        match tokio::time::timeout(Duration::from_secs(30), rx).await {
            Ok(result) => result.map_err(|_| Error::Aria2Rpc("Failed to receive response".into()))?,
            Err(_) => Err(Error::Aria2Rpc("RPC call timed out (30s)".into())),
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
            .ok_or_else(|| Error::Aria2Rpc("Invalid response".into()))
    }

    /// Add torrent download
    pub async fn add_torrent(&self, torrent: &str, options: Option<Value>) -> Result<String> {
        let mut params = vec![json!(torrent)];
        params.push(json!([]));
        if let Some(opts) = options {
            params.push(opts);
        }
        let result = self.call("addTorrent", params).await?;
        result
            .as_str()
            .map(String::from)
            .ok_or_else(|| Error::Aria2Rpc("Invalid response".into()))
    }

    /// Add metalink download
    pub async fn add_metalink(&self, metalink: &str, options: Option<Value>) -> Result<Value> {
        let mut params = vec![json!(metalink)];
        if let Some(opts) = options {
            params.push(opts);
        }
        self.call("addMetalink", params).await
    }

    /// Pause task
    pub async fn pause(&self, gid: &str) -> Result<String> {
        let result = self.call("pause", vec![json!(gid)]).await?;
        result.as_str().map(String::from).ok_or_else(|| Error::Aria2Rpc("Invalid response".into()))
    }

    /// Resume task
    pub async fn unpause(&self, gid: &str) -> Result<String> {
        let result = self.call("unpause", vec![json!(gid)]).await?;
        result.as_str().map(String::from).ok_or_else(|| Error::Aria2Rpc("Invalid response".into()))
    }

    /// Remove task
    pub async fn remove(&self, gid: &str) -> Result<String> {
        let result = self.call("remove", vec![json!(gid)]).await?;
        result.as_str().map(String::from).ok_or_else(|| Error::Aria2Rpc("Invalid response".into()))
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
        self.call("tellWaiting", vec![json!(offset), json!(num)]).await
    }

    /// Get stopped tasks
    pub async fn tell_stopped(&self, offset: i32, num: i32) -> Result<Value> {
        self.call("tellStopped", vec![json!(offset), json!(num)]).await
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

    /// Remove download result
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

    /// Force pause a task
    pub async fn force_pause(&self, gid: &str) -> Result<String> {
        let result = self.call("forcePause", vec![json!(gid)]).await?;
        result.as_str().map(String::from).ok_or_else(|| Error::Aria2Rpc("Invalid response".into()))
    }

    /// Force remove a task
    pub async fn force_remove(&self, gid: &str) -> Result<String> {
        let result = self.call("forceRemove", vec![json!(gid)]).await?;
        result.as_str().map(String::from).ok_or_else(|| Error::Aria2Rpc("Invalid response".into()))
    }

    /// Get aria2 version info (also used as heartbeat ping)
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

    /// Change task position in the waiting queue
    pub async fn change_position(&self, gid: &str, pos: i32, how: &str) -> Result<Value> {
        self.call("changePosition", vec![json!(gid), json!(pos), json!(how)]).await
    }

    /// Shutdown aria2
    pub async fn shutdown(&self) -> Result<Value> {
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
