//! System tray management

use crate::aria2;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, Runtime,
};

pub fn create_tray<R: Runtime>(app: &tauri::App<R>) -> Result<(), tauri::Error> {
    let show_i = MenuItem::with_id(app, "show", "Show Motrix", true, None::<&str>)?;
    let pause_all_i = MenuItem::with_id(app, "pause_all", "Pause All", true, None::<&str>)?;
    let resume_all_i = MenuItem::with_id(app, "resume_all", "Resume All", true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&show_i, &pause_all_i, &resume_all_i, &quit_i])?;

    let _tray = TrayIconBuilder::new()
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
                let _app = app.clone();
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
                let _app = app.clone();
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
                // Shutdown aria2 before quitting
                tauri::async_runtime::spawn(async {
                    if let Ok(client) = aria2::get_client().await {
                        let _ = client.shutdown().await;
                    }
                });
                app.exit(0);
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
