//! Application menu bar

use tauri::{
    menu::{Menu, MenuItemBuilder, SubmenuBuilder},
    Emitter, Runtime,
};

pub fn build_menu<R: Runtime>(app: &tauri::App<R>) -> Result<(), tauri::Error> {
    #[cfg(target_os = "macos")]
    {
        use tauri::menu::PredefinedMenuItem;

        let app_menu = SubmenuBuilder::new(app, "Motrix")
            .about(None)
            .separator()
            .services()
            .separator()
            .hide()
            .hide_others()
            .show_all()
            .separator()
            .quit()
            .build()?;

        let add_task = MenuItemBuilder::with_id("menu-add-task", "Add Task")
            .accelerator("CmdOrCtrl+N")
            .build(app)?;

        let file_menu = SubmenuBuilder::new(app, "File")
            .item(&add_task)
            .separator()
            .close_window()
            .build()?;

        let edit_menu = SubmenuBuilder::new(app, "Edit")
            .undo()
            .redo()
            .separator()
            .cut()
            .copy()
            .paste()
            .select_all()
            .build()?;

        let window_menu = SubmenuBuilder::new(app, "Window")
            .minimize()
            .item(&PredefinedMenuItem::maximize(app, None)?)
            .separator()
            .close_window()
            .build()?;

        let menu = Menu::with_items(app, &[&app_menu, &file_menu, &edit_menu, &window_menu])?;
        app.set_menu(menu)?;
    }

    #[cfg(not(target_os = "macos"))]
    {
        let add_task = MenuItemBuilder::with_id("menu-add-task", "Add Task")
            .accelerator("CmdOrCtrl+N")
            .build(app)?;

        let preferences = MenuItemBuilder::with_id("menu-preferences", "Preferences")
            .accelerator("CmdOrCtrl+,")
            .build(app)?;

        let file_menu = SubmenuBuilder::new(app, "File")
            .item(&add_task)
            .separator()
            .item(&preferences)
            .separator()
            .quit()
            .build()?;

        let edit_menu = SubmenuBuilder::new(app, "Edit")
            .cut()
            .copy()
            .paste()
            .select_all()
            .build()?;

        let menu = Menu::with_items(app, &[&file_menu, &edit_menu])?;
        app.set_menu(menu)?;
    }

    // Handle menu events
    let app_handle = app.handle().clone();
    app.on_menu_event(move |_app, event| {
        match event.id().as_ref() {
            "menu-add-task" => {
                let _ = app_handle.emit("menu-add-task", ());
            }
            "menu-preferences" => {
                let _ = app_handle.emit("menu-preferences", ());
            }
            _ => {}
        }
    });

    Ok(())
}
