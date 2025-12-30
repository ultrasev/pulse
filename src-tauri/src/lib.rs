use std::sync::Mutex;
use tauri::State;
use sysinfo::{System, Disks, Networks};

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
        .manage(AppState {
            sys: Mutex::new(System::new_all()),
            networks: Mutex::new(Networks::new_with_refreshed_list()),
        })
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_system_stats])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
