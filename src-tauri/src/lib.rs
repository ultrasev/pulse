use std::sync::Mutex;
use std::time::Duration;
use tauri::{State, Manager, AppHandle};
use sysinfo::{System, Disks, Networks};

// Native imports
use objc2::rc::{Allocated, Retained};
use objc2::ClassType;
use objc2_app_kit::{
    NSColor, NSStatusBar, NSStatusItem, NSMenu, NSMenuItem,
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
                // 最小化而不是退出
                let _ = window.minimize();
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

            app.set_activation_policy(tauri::ActivationPolicy::Regular);

            let mtm = unsafe { MainThreadMarker::new_unchecked() };

            let status_bar = NSStatusBar::systemStatusBar();
            let status_item = status_bar.statusItemWithLength(NSVariableStatusItemLength);

            if let Some(button) = status_item.button(mtm) {
                 button.setTitle(ns_string!("System Monitor"));
            }

            // 创建原生菜单
            let menu = NSMenu::new(mtm);

            // 添加 "Show Window" 菜单项 - 使用 unhide: 来显示应用
            let alloc_show: Allocated<NSMenuItem> = unsafe { objc2::msg_send![NSMenuItem::class(), alloc] };
            let show_item = unsafe {
                NSMenuItem::initWithTitle_action_keyEquivalent(
                    alloc_show,
                    ns_string!("Show Window"),
                    Some(objc2::sel!(unhide:)),
                    ns_string!("s")
                )
            };
            menu.addItem(&show_item);

            // 添加分隔线
            menu.addItem(&NSMenuItem::separatorItem(mtm));

            // 添加 "Quit" 菜单项
            let alloc_item: Allocated<NSMenuItem> = unsafe { objc2::msg_send![NSMenuItem::class(), alloc] };
            let quit_item = unsafe {
                NSMenuItem::initWithTitle_action_keyEquivalent(
                    alloc_item,
                    ns_string!("Quit"),
                    Some(objc2::sel!(terminate:)),
                    ns_string!("q")
                )
            };
            menu.addItem(&quit_item);

            status_item.setMenu(Some(&menu));

            let state = app.state::<AppState>();
            *state.status_item.lock().unwrap() = Some(ThreadSafeStatusItem(status_item));

            start_tray_update_loop(app.handle().clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_system_stats])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
