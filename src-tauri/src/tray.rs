//! System tray management

use crate::aria2;
use serde::Deserialize;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, Runtime,
};

/// Tray menu labels for i18n
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrayLabels {
    pub show: String,
    pub pause_all: String,
    pub resume_all: String,
    pub quit: String,
}

pub fn create_tray<R: Runtime>(app: &tauri::App<R>) -> Result<(), tauri::Error> {
    let show_i = MenuItem::with_id(app, "show", "Show Motrix", true, None::<&str>)?;
    let pause_all_i = MenuItem::with_id(app, "pause_all", "Pause All", true, None::<&str>)?;
    let resume_all_i = MenuItem::with_id(app, "resume_all", "Resume All", true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&show_i, &pause_all_i, &resume_all_i, &quit_i])?;

    let _tray = TrayIconBuilder::with_id("main")
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "pause_all" => {
                tauri::async_runtime::spawn(async move {
                    match aria2::get_client().await {
                        Ok(client) => {
                            if let Err(e) = client.pause_all().await {
                                tracing::error!("Failed to pause all: {}", e);
                            } else {
                                tracing::info!("All tasks paused");
                            }
                        }
                        Err(e) => tracing::error!("Aria2 client error: {}", e),
                    }
                });
            }
            "resume_all" => {
                tauri::async_runtime::spawn(async move {
                    match aria2::get_client().await {
                        Ok(client) => {
                            if let Err(e) = client.unpause_all().await {
                                tracing::error!("Failed to resume all: {}", e);
                            } else {
                                tracing::info!("All tasks resumed");
                            }
                        }
                        Err(e) => tracing::error!("Aria2 client error: {}", e),
                    }
                });
            }
            "quit" => {
                tracing::info!("Quitting application");
                let app_handle = app.clone();
                tauri::async_runtime::spawn(async move {
                    aria2::shutdown_and_cleanup().await;
                    app_handle.exit(0);
                });
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;

    Ok(())
}

/// Update tray menu labels (called from frontend on locale change)
pub fn update_tray_labels<R: Runtime>(app: &tauri::AppHandle<R>, labels: &TrayLabels) -> Result<(), tauri::Error> {
    if let Some(tray) = app.tray_by_id("main") {
        // Rebuild menu with updated labels
        let show_i = MenuItem::with_id(app, "show", &labels.show, true, None::<&str>)?;
        let pause_all_i = MenuItem::with_id(app, "pause_all", &labels.pause_all, true, None::<&str>)?;
        let resume_all_i = MenuItem::with_id(app, "resume_all", &labels.resume_all, true, None::<&str>)?;
        let quit_i = MenuItem::with_id(app, "quit", &labels.quit, true, None::<&str>)?;

        let menu = Menu::with_items(app, &[&show_i, &pause_all_i, &resume_all_i, &quit_i])?;
        tray.set_menu(Some(menu))?;
    }
    Ok(())
}

/// Format bytes/s into a compact speed string (e.g. "1.2M", "512K")
fn format_compact_speed(bytes_per_sec: u64) -> String {
    if bytes_per_sec == 0 {
        return String::new();
    }
    if bytes_per_sec >= 1_073_741_824 {
        format!("{:.1}G", bytes_per_sec as f64 / 1_073_741_824.0)
    } else if bytes_per_sec >= 1_048_576 {
        format!("{:.1}M", bytes_per_sec as f64 / 1_048_576.0)
    } else if bytes_per_sec >= 1024 {
        format!("{}K", bytes_per_sec / 1024)
    } else {
        format!("{}B", bytes_per_sec)
    }
}

/// Update tray icon with speed information
pub fn update_tray_speed<R: Runtime>(
    app: &tauri::AppHandle<R>,
    download_speed: &str,
    upload_speed: &str,
    enabled: bool,
) -> Result<(), tauri::Error> {
    let Some(tray) = app.tray_by_id("main") else {
        return Ok(());
    };

    let dl: u64 = download_speed.parse().unwrap_or(0);
    let ul: u64 = upload_speed.parse().unwrap_or(0);

    if !enabled || (dl == 0 && ul == 0) {
        tray.set_tooltip(Some("Motrix"))?;
        #[cfg(target_os = "macos")]
        tray.set_title(None)?;
        return Ok(());
    }

    let dl_str = format_compact_speed(dl);
    let ul_str = format_compact_speed(ul);

    // Build tooltip text
    let tooltip = if !dl_str.is_empty() && !ul_str.is_empty() {
        format!("↓ {}/s  ↑ {}/s", dl_str, ul_str)
    } else if !dl_str.is_empty() {
        format!("↓ {}/s", dl_str)
    } else {
        format!("↑ {}/s", ul_str)
    };

    tray.set_tooltip(Some(&tooltip))?;

    // On macOS, show compact speed next to tray icon
    #[cfg(target_os = "macos")]
    {
        let title = if !dl_str.is_empty() {
            format!("↓{}", dl_str)
        } else if !ul_str.is_empty() {
            format!("↑{}", ul_str)
        } else {
            String::new()
        };
        tray.set_title(Some(&title))?;
    }

    Ok(())
}
