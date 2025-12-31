use std::sync::Mutex;
use std::time::Duration;
use std::thread;
use std::fs;
use tauri::{State, Manager, AppHandle, Emitter};
use sysinfo::{System, Disks, Networks};
use arboard::Clipboard;
use base64::Engine;
use image::{ImageBuffer, RgbaImage};

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

#[derive(serde::Serialize, Clone, Debug)]
struct UploadResult {
    success: bool,
    url: Option<String>,
    filename: Option<String>,
    size: Option<String>,
    duration: Option<String>,
    error: Option<String>,
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

// Image data from clipboard
#[derive(serde::Serialize)]
struct ClipboardImage {
    has_image: bool,
    data_url: Option<String>,
    size_bytes: Option<usize>,
    error: Option<String>,
}

/// Get image from clipboard as base64 data URL
#[tauri::command]
fn get_clipboard_image() -> ClipboardImage {
    match Clipboard::new() {
        Ok(mut clipboard) => {
            match clipboard.get_image() {
                Ok(image_data) => {
                    let size = image_data.bytes.len();
                    let data_url = format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(&image_data.bytes));
                    ClipboardImage {
                        has_image: true,
                        data_url: Some(data_url),
                        size_bytes: Some(size),
                        error: None,
                    }
                }
                Err(_) => ClipboardImage {
                    has_image: false,
                    data_url: None,
                    size_bytes: None,
                    error: Some("No image in clipboard".to_string()),
                }
            }
        }
        Err(e) => ClipboardImage {
            has_image: false,
            data_url: None,
            size_bytes: None,
            error: Some(format!("Failed to access clipboard: {}", e)),
        }
    }
}

/// Convert raw RGBA bytes from clipboard to PNG format
fn rgba_to_png(rgba_data: &[u8], width: usize, height: usize) -> Result<Vec<u8>, String> {
    // Create ImageBuffer from RGBA data
    let img: RgbaImage = ImageBuffer::from_raw(
        width as u32,
        height as u32,
        rgba_data.to_vec(),
    ).ok_or("Failed to create image buffer")?;

    // Encode as PNG
    let mut png_bytes = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut png_bytes), image::ImageFormat::Png)
        .map_err(|e| format!("Failed to encode PNG: {}", e))?;

    Ok(png_bytes)
}

/// Upload image data to server with retry logic
#[tauri::command]
fn upload_image(image_base64: String, retry_count: Option<u32>) -> Result<UploadResult, String> {
    upload_image_with_retry(image_base64, retry_count.unwrap_or(0))
}

fn upload_image_with_retry(image_base64: String, retry_count: u32) -> Result<UploadResult, String> {
    let url = "http://REDACTED_HOST:38080/api/image";

    // Extract base64 data from data URL if present
    let base64_data = if image_base64.starts_with("data:image/") {
        image_base64.split(',').nth(1).unwrap_or(&image_base64)
    } else {
        &image_base64
    };

    // Decode base64 to bytes
    let image_bytes = base64::engine::general_purpose::STANDARD
        .decode(base64_data)
        .map_err(|e| {
            log::error!("Failed to decode base64: {}", e);
            format!("Failed to decode base64: {}", e)
        })?;

    let size_bytes = image_bytes.len();

    // Verify PNG magic bytes before uploading
    if image_bytes.len() >= 8 {
        let header = &image_bytes[0..8];
        log::info!("Upload image header: {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x}",
            header[0], header[1], header[2], header[3],
            header[4], header[5], header[6], header[7]);
    }

    log::info!("Uploading image: {} bytes, attempt {}", size_bytes, retry_count + 1);

    // Create multipart form using blocking client
    let part = reqwest::blocking::multipart::Part::bytes(image_bytes.clone())
        .file_name("image.png")
        .mime_str("image/png")
        .map_err(|e| {
            log::error!("Failed to create mime part: {}", e);
            format!("Failed to create mime part: {}", e)
        })?;

    let form = reqwest::blocking::multipart::Form::new()
        .part("file", part);

    // Create client with timeout
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(|e| {
            log::error!("Failed to create HTTP client: {}", e);
            format!("Failed to create HTTP client: {}", e)
        })?;

    // Make the request
    log::info!("Sending PUT request to {}", url);
    let response = client
        .put(url)
        .header("Authorization", "Bearer REDACTED")
        .multipart(form)
        .send();

    match response {
        Ok(resp) => {
            let status = resp.status();
            let response_text = resp.text().unwrap_or_else(|_| "Unable to decode response".to_string());

            log::info!("Upload response status: {}", status);
            log::info!("Upload response body: {}", response_text);

            if status.is_success() {
                // Parse JSON response
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&response_text) {
                    log::info!("Parsed JSON response: {}", json);

                    // The API returns the path directly, just concatenate
                    if let Some(url_path) = json["url"].as_str() {
                        log::info!("url_path from API: {}", url_path);

                        // Don't decode - use the raw path as-is from API
                        let full_url = format!("http://REDACTED_HOST:38080{}", url_path);
                        log::info!("Final image URL: {}", full_url);

                        let filename = json["originalFileName"].as_str().unwrap_or("image.png");
                        let size = format_size(size_bytes);
                        return Ok(UploadResult {
                            success: true,
                            url: Some(full_url),
                            filename: Some(filename.to_string()),
                            size: Some(size),
                            duration: None,
                            error: None,
                        });
                    } else {
                        log::error!("No 'url' field in response");
                        return Err(format!("No 'url' field in response: {}", response_text));
                    }
                } else {
                    log::error!("Failed to parse JSON response");
                    return Err(format!("Failed to parse JSON: {}", response_text));
                }
            } else if (status.is_server_error() || status == 429) && retry_count < 2 {
                // Retry on server error or rate limit
                log::warn!("Server error, retrying... status: {}", status);
                thread::sleep(Duration::from_secs(1));
                upload_image_with_retry(image_base64, retry_count + 1)
            } else {
                log::error!("Upload failed with status {}: {}", status, response_text);
                Err(format!("Upload failed with status {}: {}", status, response_text))
            }
        }
        Err(e) => {
            if e.is_timeout() || e.is_connect() && retry_count < 2 {
                // Retry on timeout or connection error
                log::warn!("Network error, retrying: {}", e);
                thread::sleep(Duration::from_secs(1));
                upload_image_with_retry(image_base64, retry_count + 1)
            } else {
                log::error!("Network error: {}", e);
                Err(format!("Network error: {}", e))
            }
        }
    }
}

fn format_size(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / 1024.0 / 1024.0)
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
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

            // Register global shortcut for image upload (Shift+Cmd+U)
            use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut, ShortcutState, GlobalShortcutExt};

            log::info!("Registering global shortcut: Shift+Cmd+U for image upload");

            let handle = app.handle().clone();
            app.global_shortcut().on_shortcut(
                Shortcut::new(Some(Modifiers::SHIFT | Modifiers::SUPER), Code::KeyU),
                move |_app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        log::info!("Global shortcut triggered: Shift+Cmd+U");
                        let handle = handle.clone();
                        tauri::async_runtime::spawn_blocking(move || {
                            // Perform upload in background thread
                            log::info!("Accessing clipboard...");
                            if let Ok(mut clipboard) = Clipboard::new() {
                                if let Ok(image_data) = clipboard.get_image() {
                                    log::info!("Got image from clipboard: {} bytes, {}x{}", image_data.bytes.len(), image_data.width, image_data.height);

                                    // Convert RGBA to PNG
                                    let png_bytes = match rgba_to_png(&image_data.bytes, image_data.width, image_data.height) {
                                        Ok(data) => {
                                            log::info!("Converted to PNG: {} bytes", data.len());
                                            // Verify PNG header
                                            if data.len() >= 8 {
                                                let header = &data[0..8];
                                                log::info!("PNG header bytes: {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x}",
                                                    header[0], header[1], header[2], header[3],
                                                    header[4], header[5], header[6], header[7]);
                                            }
                                            data
                                        }
                                        Err(e) => {
                                            log::error!("Failed to convert to PNG: {}", e);
                                            let _ = handle.emit("upload-result", UploadResult {
                                                success: false,
                                                url: None,
                                                filename: None,
                                                size: None,
                                                duration: None,
                                                error: Some(format!("Failed to convert image: {}", e)),
                                            });
                                            if let Some(window) = handle.get_webview_window("main") {
                                                let _ = window.show();
                                                let _ = window.set_focus();
                                                let _ = window.emit("switch-to-upload", ());
                                            }
                                            return;
                                        }
                                    };

                                    // DEBUG: Save PNG to verify
                                    if let Err(e) = fs::write("/tmp/clipboard_debug.png", &png_bytes) {
                                        log::error!("Failed to save debug image: {}", e);
                                    } else {
                                        log::info!("Saved debug PNG to /tmp/clipboard_debug.png");
                                    }

                                    let base64_data = base64::engine::general_purpose::STANDARD.encode(&png_bytes);
                                    let data_url = format!("data:image/png;base64,{}", base64_data);

                                    // Upload and emit result
                                    log::info!("Starting upload...");
                                    match upload_image_with_retry(data_url, 0) {
                                        Ok(result) => {
                                            log::info!("Upload successful: {:?}", result);

                                            // First show window and switch tab, then emit result
                                            if let Some(window) = handle.get_webview_window("main") {
                                                let _ = window.show();
                                                let _ = window.set_focus();
                                                // Small delay to ensure window is ready
                                                thread::sleep(Duration::from_millis(50));
                                                let _ = window.emit("switch-to-upload", ());
                                            }
                                            // Then emit result after tab switch
                                            thread::sleep(Duration::from_millis(50));
                                            let _ = handle.emit("upload-result", result);
                                        }
                                        Err(err) => {
                                            log::error!("Upload failed: {}", err);

                                            if let Some(window) = handle.get_webview_window("main") {
                                                let _ = window.show();
                                                let _ = window.set_focus();
                                                thread::sleep(Duration::from_millis(50));
                                                let _ = window.emit("switch-to-upload", ());
                                            }
                                            thread::sleep(Duration::from_millis(50));
                                            let _ = handle.emit("upload-result", UploadResult {
                                                success: false,
                                                url: None,
                                                filename: None,
                                                size: None,
                                                duration: None,
                                                error: Some(err),
                                            });
                                        }
                                    }
                                } else {
                                    log::warn!("No image in clipboard");

                                    if let Some(window) = handle.get_webview_window("main") {
                                        let _ = window.show();
                                        let _ = window.set_focus();
                                        thread::sleep(Duration::from_millis(50));
                                        let _ = window.emit("switch-to-upload", ());
                                    }
                                    thread::sleep(Duration::from_millis(50));
                                    let _ = handle.emit("upload-result", UploadResult {
                                        success: false,
                                        url: None,
                                        filename: None,
                                        size: None,
                                        duration: None,
                                        error: Some("No image in clipboard".to_string()),
                                    });
                                }
                            } else {
                                log::error!("Failed to access clipboard");
                            }
                        });
                    }
                }
            )?;

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

            // 加载托盘图标 (圆角版本)
            let icon = tauri::image::Image::from_bytes(include_bytes!("../icons/tray-icon-rounded.png"))
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
        .invoke_handler(tauri::generate_handler![
            get_system_stats,
            get_clipboard_image,
            upload_image
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
