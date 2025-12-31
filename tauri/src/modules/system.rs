use tauri::{State, AppHandle};
use sysinfo::{System, Disks, Networks};
use crate::modules::{SystemStats, AppState};
use std::process::Command;

#[cfg(target_os = "macos")]
fn get_macos_memory_usage() -> Option<u64> {
    let output = Command::new("vm_stat").output().ok()?;
    let output_str = String::from_utf8_lossy(&output.stdout);

    // Default to 16KB for Apple Silicon, fallback to 4KB if unknown
    // We try to parse the header "Mach Virtual Memory Statistics: (page size of 16384 bytes)"
    let mut page_size = 16384;
    if let Some(first_line) = output_str.lines().next() {
        if let Some(start) = first_line.find("page size of ") {
            if let Some(end) = first_line[start..].find(" bytes") {
                if let Ok(size) = first_line[start + 13..start + end].parse::<u64>() {
                    page_size = size;
                }
            }
        }
    }

    let mut pages_anonymous = 0;
    let mut pages_purgeable = 0;

    for line in output_str.lines() {
        if line.starts_with("Anonymous pages:") {
            if let Some(val) = line.split(':').nth(1) {
                pages_anonymous = val.trim().trim_end_matches('.').parse::<u64>().unwrap_or(0);
            }
        } else if line.starts_with("Pages purgeable:") {
             if let Some(val) = line.split(':').nth(1) {
                pages_purgeable = val.trim().trim_end_matches('.').parse::<u64>().unwrap_or(0);
            }
        }
    }

    // "App Memory" calculation matching Activity Monitor
    // App Memory = (Anonymous pages - Purgeable pages) * Page Size
    // This represents the physical memory used by user-space apps
    let used_bytes = (pages_anonymous.saturating_sub(pages_purgeable)) * page_size;
    Some(used_bytes)
}

#[tauri::command]
pub fn get_system_stats(state: State<AppState>) -> SystemStats {
    let mut sys = state.sys.lock().unwrap();
    let mut networks = state.networks.lock().unwrap();

    sys.refresh_all();
    networks.refresh(true);

    let cpu_usage = sys.global_cpu_usage();
    let memory_total = sys.total_memory();

    // Use platform-specific calculation for macOS, fallback to sysinfo for others
    #[cfg(target_os = "macos")]
    let memory_used = get_macos_memory_usage().unwrap_or_else(|| sys.used_memory());

    #[cfg(not(target_os = "macos"))]
    let memory_used = sys.used_memory();

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
