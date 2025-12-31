use std::time::Duration;
use std::thread;
use arboard::Clipboard;
use base64::Engine;
use image::{ImageBuffer, RgbaImage};
use tauri::{Emitter, Manager};
use crate::modules::UploadResult;

/// Get image from clipboard as base64 data URL
#[tauri::command]
pub fn get_clipboard_image() -> crate::modules::ClipboardImage {
    match Clipboard::new() {
        Ok(mut clipboard) => {
            match clipboard.get_image() {
                Ok(image_data) => {
                    let size = image_data.bytes.len();
                    let data_url = format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(&image_data.bytes));
                    crate::modules::ClipboardImage {
                        has_image: true,
                        data_url: Some(data_url),
                        size_bytes: Some(size),
                        error: None,
                    }
                }
                Err(_) => crate::modules::ClipboardImage {
                    has_image: false,
                    data_url: None,
                    size_bytes: None,
                    error: Some("No image in clipboard".to_string()),
                }
            }
        }
        Err(e) => crate::modules::ClipboardImage {
            has_image: false,
            data_url: None,
            size_bytes: None,
            error: Some(format!("Failed to access clipboard: {}", e)),
        }
    }
}

/// Convert raw RGBA bytes from clipboard to PNG format
pub fn rgba_to_png(rgba_data: &[u8], width: usize, height: usize) -> Result<Vec<u8>, String> {
    let img: RgbaImage = ImageBuffer::from_raw(
        width as u32,
        height as u32,
        rgba_data.to_vec(),
    ).ok_or("Failed to create image buffer")?;

    let mut png_bytes = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut png_bytes), image::ImageFormat::Png)
        .map_err(|e| format!("Failed to encode PNG: {}", e))?;

    Ok(png_bytes)
}

/// Upload image data to server with retry logic
#[tauri::command]
pub fn upload_image(image_base64: String, retry_count: Option<u32>) -> Result<UploadResult, String> {
    upload_image_with_retry(image_base64, retry_count.unwrap_or(0))
}

fn upload_image_with_retry(image_base64: String, retry_count: u32) -> Result<UploadResult, String> {
    let url = "http://REDACTED_HOST:38080/api/image";

    let base64_data = if image_base64.starts_with("data:image/") {
        image_base64.split(',').nth(1).unwrap_or(&image_base64)
    } else {
        &image_base64
    };

    let image_bytes = base64::engine::general_purpose::STANDARD
        .decode(base64_data)
        .map_err(|e| {
            log::error!("Failed to decode base64: {}", e);
            format!("Failed to decode base64: {}", e)
        })?;

    let size_bytes = image_bytes.len();

    if image_bytes.len() >= 8 {
        let header = &image_bytes[0..8];
        log::info!("Upload image header: {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x}",
            header[0], header[1], header[2], header[3],
            header[4], header[5], header[6], header[7]);
    }

    log::info!("Uploading image: {} bytes, attempt {}", size_bytes, retry_count + 1);

    let part = reqwest::blocking::multipart::Part::bytes(image_bytes.clone())
        .file_name("image.png")
        .mime_str("image/png")
        .map_err(|e| {
            log::error!("Failed to create mime part: {}", e);
            format!("Failed to create mime part: {}", e)
        })?;

    let form = reqwest::blocking::multipart::Form::new()
        .part("file", part);

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(|e| {
            log::error!("Failed to create HTTP client: {}", e);
            format!("Failed to create HTTP client: {}", e)
        })?;

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
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&response_text) {
                    log::info!("Parsed JSON response: {}", json);

                    if let Some(url_path) = json["url"].as_str() {
                        log::info!("url_path from API: {}", url_path);
                        let full_url = format!("http://REDACTED_HOST:38080{}", url_path);
                        log::info!("Final image URL: {}", full_url);

                        let filename = json["originalFileName"].as_str().unwrap_or("image.png");
                        let size = crate::modules::utils::format_size(size_bytes);
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

/// Handle global shortcut trigger for image upload
pub fn handle_upload_shortcut(handle: tauri::AppHandle) {
    log::info!("Global shortcut triggered: Shift+Cmd+U");
    tauri::async_runtime::spawn_blocking(move || {
        log::info!("Accessing clipboard...");
        if let Ok(mut clipboard) = Clipboard::new() {
            if let Ok(image_data) = clipboard.get_image() {
                log::info!("Got image from clipboard: {} bytes, {}x{}", image_data.bytes.len(), image_data.width, image_data.height);

                let png_bytes = match rgba_to_png(&image_data.bytes, image_data.width, image_data.height) {
                    Ok(data) => {
                        log::info!("Converted to PNG: {} bytes", data.len());
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

                let base64_data = base64::engine::general_purpose::STANDARD.encode(&png_bytes);
                let data_url = format!("data:image/png;base64,{}", base64_data);

                log::info!("Starting upload...");
                match upload_image_with_retry(data_url, 0) {
                    Ok(result) => {
                        log::info!("Upload successful: {:?}", result);

                        if let Some(window) = handle.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                            thread::sleep(Duration::from_millis(50));
                            let _ = window.emit("switch-to-upload", ());
                        }
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
