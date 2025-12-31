mod modules;

use std::sync::Mutex;
use tauri::Manager;
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut, ShortcutState, GlobalShortcutExt};
use tauri::tray::{TrayIconBuilder, MouseButton, MouseButtonState};
use tauri::menu::{MenuBuilder, MenuItemBuilder};

use modules::AppState;
use modules::system::{get_system_stats, start_tray_update_loop};
use modules::upload::{get_clipboard_image, upload_image, handle_upload_shortcut};

// Native imports
use objc2::MainThreadMarker;
use objc2_app_kit::{NSStatusBar, NSVariableStatusItemLength};
use objc2_foundation::ns_string;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            sys: Mutex::new(sysinfo::System::new_all()),
            networks: Mutex::new(sysinfo::Networks::new_with_refreshed_list()),
            status_item: Mutex::new(None),
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                let _ = window.hide();
                api.prevent_close();
            }
            _ => {}
        })
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            // Register global shortcut for image upload (Shift+Cmd+U)
            log::info!("Registering global shortcut: Shift+Cmd+U for image upload");

            let handle = app.handle().clone();
            app.global_shortcut().on_shortcut(
                Shortcut::new(Some(Modifiers::SHIFT | Modifiers::SUPER), Code::KeyU),
                move |_app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        handle_upload_shortcut(handle.clone());
                    }
                }
            )?;

            // Setup tray
            let show_item = MenuItemBuilder::with_id("show", "Show Window").build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
            let tray_menu = MenuBuilder::new(app)
                .item(&show_item)
                .separator()
                .item(&quit_item)
                .build()?;

            let icon = tauri::image::Image::from_bytes(include_bytes!("../icons/tray-icon-rounded.png"))
                .expect("Failed to load tray icon");

            let _tray = TrayIconBuilder::new()
                .icon(icon)
                .tooltip("System Monitor")
                .menu(&tray_menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| {
                    match event.id().as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            // Setup native status bar
            let mtm = unsafe { MainThreadMarker::new_unchecked() };
            let status_bar = NSStatusBar::systemStatusBar();
            let status_item = status_bar.statusItemWithLength(NSVariableStatusItemLength);

            if let Some(button) = status_item.button(mtm) {
                 button.setTitle(ns_string!("System Monitor"));
            }

            let state = app.state::<AppState>();
            *state.status_item.lock().unwrap() = Some(modules::ThreadSafeStatusItem(status_item));

            start_tray_update_loop(app.handle().clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_system_stats,
            get_clipboard_image,
            upload_image
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
