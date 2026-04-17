//! Aria2 event types and notification handling

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

/// Aria2 event types emitted to the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Aria2Event {
    pub event_type: Aria2EventType,
    pub gid: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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
pub(super) struct Aria2Notification {
    pub method: String,
    pub params: Vec<NotificationParam>,
}

#[derive(Debug, Deserialize)]
pub(super) struct NotificationParam {
    pub gid: String,
}

/// Handle aria2 notification events and emit to frontend
pub(super) fn handle_notification(app_handle: &AppHandle, notification: Aria2Notification) {
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
            event_type,
            gid: param.gid.clone(),
        };

        tracing::info!("Aria2 event: {:?} for gid {}", event_type, param.gid);

        if let Err(e) = app_handle.emit("aria2-event", &event) {
            tracing::error!("Failed to emit aria2 event: {}", e);
        }
    }
}
