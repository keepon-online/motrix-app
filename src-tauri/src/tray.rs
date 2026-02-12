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
