use std::sync::Mutex;
use std::time::Duration;
use tauri::{State, Manager, AppHandle};
use sysinfo::{System, Disks, Networks};

// Native imports
use objc2::rc::{Allocated, Retained};
use objc2::ClassType;
use objc2_app_kit::{
    NSColor, NSStatusBar, NSStatusItem,
    NSVariableStatusItemLength,
};
use objc2_foundation::{
    ns_string, NSDictionary, NSMutableAttributedString, NSString, MainThreadMarker, NSRange,
};
use objc2::runtime::AnyObject;

// Wrapper for Thread Safety
struct ThreadSafeStatusItem(Retained<NSStatusItem>);
unsafe impl Send for ThreadSafeStatusItem {}
unsafe impl Sync for ThreadSafeStatusItem {}

#[derive(serde::Serialize)]
struct SystemStats {
    cpu_usage: f32,
    memory_used: u64,
    memory_total: u64,
    disk_usage_percent: u64,
    network_speed_up: u64,
    network_speed_down: u64,
}

struct AppState {
    sys: Mutex<System>,
    networks: Mutex<Networks>,
    // Store wrapped item
    status_item: Mutex<Option<ThreadSafeStatusItem>>,
}

fn format_speed(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{:>3} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:>3} K/s", bytes / 1024)
    } else {
        format!("{:>3.1} M/s", bytes as f64 / 1024.0 / 1024.0)
    }
}

// Helper to determine color based on CPU usage
fn get_cpu_color(cpu: f32) -> Retained<NSColor> {
    if cpu >= 80.0 {
        NSColor::yellowColor()
    } else if cpu >= 50.0 {
        NSColor::orangeColor()
    } else {
        NSColor::controlTextColor()
    }
}

// Helper to determine color based on network speed (in bytes/s)
// < 5 MB/s: default color, 5-10 MB/s: orange, > 10 MB/s: red
fn get_network_color(bytes_per_sec: u64) -> Retained<NSColor> {
    let mb_per_sec = bytes_per_sec as f64 / (1024.0 * 1024.0);
    if mb_per_sec > 10.0 {
        NSColor::redColor()
    } else if mb_per_sec >= 5.0 {
        NSColor::orangeColor()
    } else {
        NSColor::controlTextColor()
    }
}

fn start_tray_update_loop(app: AppHandle) {
    std::thread::spawn(move || {
        let mut sys = System::new_all();
        let mut networks = Networks::new_with_refreshed_list();

        loop {
            std::thread::sleep(Duration::from_secs(1));

            sys.refresh_cpu_all();
            networks.refresh(true);

            let cpu = sys.global_cpu_usage();

            let mut up = 0;
            let mut down = 0;
            for (_name, network) in &networks {
                up += network.transmitted();
                down += network.received();
            }

            // 简化显示：只显示百分比，不显示 "CPU:" 前缀
            let cpu_str = format!("{:.0}%", cpu);
            let up_str = format!("{}", format_speed(up));
            let down_str = format!("{}", format_speed(down));

            // Helpers for lengths
            let sep1 = ",";
            let sep2 = ",";

            let cpu_len = cpu_str.encode_utf16().count();
            let sep1_len = sep1.encode_utf16().count();
            let up_len = up_str.encode_utf16().count();
            let sep2_len = sep2.encode_utf16().count();
            let down_len = down_str.encode_utf16().count();

            // Build attributed string with independent colors
            let full_text = format!("{}{}{}{}{}", cpu_str, sep1, up_str, sep2, down_str);

            // Clone app handle for the closure to capture
            let handle = app.clone();

            let _ = app.run_on_main_thread(move || {
                let mtm = unsafe { MainThreadMarker::new_unchecked() };

                let state = handle.state::<AppState>();
                let lock = state.status_item.lock().unwrap();

                if let Some(wrapper) = lock.as_ref() {
                    let item = &wrapper.0;

                    let full_ns = NSString::from_str(&full_text);

                     // Create mutable attributed string
                    let alloc_mut: Allocated<NSMutableAttributedString> = unsafe {
                        objc2::msg_send![NSMutableAttributedString::class(), alloc]
                    };
                    let mut_attr_str = NSMutableAttributedString::initWithString(alloc_mut, &full_ns);

                    // Apply CPU color
                    let cpu_range = NSRange::new(0, cpu_len);
                    let cpu_key = ns_string!("NSColor");
                    let cpu_dict = NSDictionary::from_slices(&[cpu_key], &[&*get_cpu_color(cpu)]);
                    let cpu_dict_ptr: &NSDictionary<NSString, AnyObject> = unsafe { std::mem::transmute(&*cpu_dict) };
                    unsafe {
                        mut_attr_str.setAttributes_range(Some(cpu_dict_ptr), cpu_range);
                    }

                    // Apply upload color
                    let up_start = cpu_len + sep1_len;
                    let up_range = NSRange::new(up_start, up_len);
                    let up_dict = NSDictionary::from_slices(&[cpu_key], &[&*get_network_color(up)]);
                    let up_dict_ptr: &NSDictionary<NSString, AnyObject> = unsafe { std::mem::transmute(&*up_dict) };
                    unsafe {
                        mut_attr_str.setAttributes_range(Some(up_dict_ptr), up_range);
                    }

                    // Apply download color
                    let down_start = up_start + up_len + sep2_len;
                    let down_range = NSRange::new(down_start, down_len);
                    let down_dict = NSDictionary::from_slices(&[cpu_key], &[&*get_network_color(down)]);
                    let down_dict_ptr: &NSDictionary<NSString, AnyObject> = unsafe { std::mem::transmute(&*down_dict) };
                    unsafe {
                        mut_attr_str.setAttributes_range(Some(down_dict_ptr), down_range);
                    }

                    if let Some(button) = item.button(mtm) {
                        button.setAttributedTitle(&mut_attr_str);
                    }
                }
            });
        }
    });
}

#[tauri::command]
fn get_system_stats(state: State<AppState>) -> SystemStats {
    let mut sys = state.sys.lock().unwrap();
    let mut networks = state.networks.lock().unwrap();

    sys.refresh_all();
    networks.refresh(true);

    let cpu_usage = sys.global_cpu_usage();
    let memory_used = sys.used_memory();
    let memory_total = sys.total_memory();

    let disks = Disks::new_with_refreshed_list();
    let mut disk_usage_percent = 0;
    for disk in &disks {
        if disk.mount_point().to_string_lossy() == "/" {
             let total = disk.total_space();
             let available = disk.available_space();
             if total > 0 {
                 disk_usage_percent = ((total - available) as f64 / total as f64 * 100.0) as u64;
             }
             break;
        }
    }

    let mut network_speed_up = 0;
    let mut network_speed_down = 0;
    for (_name, network) in &*networks {
        network_speed_up += network.transmitted();
        network_speed_down += network.received();
    }

    SystemStats {
        cpu_usage,
        memory_used,
        memory_total,
        disk_usage_percent,
        network_speed_up,
        network_speed_down,
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
        .manage(AppState {
            sys: Mutex::new(System::new_all()),
            networks: Mutex::new(Networks::new_with_refreshed_list()),
            status_item: Mutex::new(None),
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                // 隐藏窗口而不是退出，这样可以通过托盘菜单的 unhide: 恢复
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

            // 使用 Tauri 的 tray 系统来处理点击事件
            use tauri::tray::{TrayIconBuilder, MouseButton, MouseButtonState};
            use tauri::menu::{MenuBuilder, MenuItemBuilder};

            let show_item = MenuItemBuilder::with_id("show", "Show Window").build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
            let tray_menu = MenuBuilder::new(app)
                .item(&show_item)
                .separator()
                .item(&quit_item)
                .build()?;

            // 加载托盘图标
            let icon = tauri::image::Image::from_bytes(include_bytes!("../icons/32x32.png"))
                .expect("Failed to load tray icon");

            let _tray = TrayIconBuilder::new()
                .icon(icon)
                .tooltip("System Monitor")
                .menu(&tray_menu)
                .show_menu_on_left_click(false)  // 左键点击不显示菜单，左键点击直接显示窗口
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
                    // 左键点击托盘图标时显示窗口
                    if let tauri::tray::TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            let mtm = unsafe { MainThreadMarker::new_unchecked() };

            let status_bar = NSStatusBar::systemStatusBar();
            let status_item = status_bar.statusItemWithLength(NSVariableStatusItemLength);

            if let Some(button) = status_item.button(mtm) {
                 button.setTitle(ns_string!("System Monitor"));
            }

            // 原生 NSStatusItem 不设置菜单，菜单由 Tauri tray 处理

            let state = app.state::<AppState>();
            *state.status_item.lock().unwrap() = Some(ThreadSafeStatusItem(status_item));

            start_tray_update_loop(app.handle().clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_system_stats])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
