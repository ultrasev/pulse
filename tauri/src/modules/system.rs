use tauri::{State, AppHandle};
use sysinfo::{System, Disks, Networks};
use crate::modules::{SystemStats, AppState};

#[tauri::command]
pub fn get_system_stats(state: State<AppState>) -> SystemStats {
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

    let mut network_speed_up: u64 = 0;
    let mut network_speed_down: u64 = 0;
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

pub fn start_tray_update_loop(app: AppHandle) {
    std::thread::spawn(move || {
        let mut sys = System::new_all();
        let mut networks = Networks::new_with_refreshed_list();

        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));

            sys.refresh_cpu_all();
            networks.refresh(true);

            let cpu = sys.global_cpu_usage();

            let mut up = 0;
            let mut down = 0;
            for (_name, network) in &networks {
                up += network.transmitted();
                down += network.received();
            }

            crate::modules::tray::update_status_bar(&app, cpu, up, down);
        }
    });
}
