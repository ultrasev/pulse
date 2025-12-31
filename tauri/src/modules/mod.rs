// Module declarations
pub mod system;
pub mod upload;
pub mod tray;
pub mod utils;
pub mod git;
pub mod config;

// Shared types and state
use std::sync::Mutex;
use sysinfo::{System, Networks};
use objc2::rc::Retained;
use objc2_app_kit::NSStatusItem;

// Wrapper for Thread Safety
pub struct ThreadSafeStatusItem(pub Retained<NSStatusItem>);
unsafe impl Send for ThreadSafeStatusItem {}
unsafe impl Sync for ThreadSafeStatusItem {}

#[derive(serde::Serialize)]
pub struct SystemStats {
    pub cpu_usage: f32,
    pub memory_used: u64,
    pub memory_total: u64,
    pub disk_usage_percent: u64,
    pub network_speed_up: u64,
    pub network_speed_down: u64,
}

#[derive(serde::Serialize, Clone, Debug)]
pub struct UploadResult {
    pub success: bool,
    pub url: Option<String>,
    pub filename: Option<String>,
    pub size: Option<String>,
    pub duration: Option<String>,
    pub error: Option<String>,
}

#[derive(serde::Serialize)]
pub struct ClipboardImage {
    pub has_image: bool,
    pub data_url: Option<String>,
    pub size_bytes: Option<usize>,
    pub error: Option<String>,
}

pub struct AppState {
    pub sys: Mutex<System>,
    pub networks: Mutex<Networks>,
    pub status_item: Mutex<Option<ThreadSafeStatusItem>>,
}
