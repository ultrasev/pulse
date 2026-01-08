use std::time::Duration;
use std::sync::OnceLock;
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Serialize)]
pub struct MijiaActionRequest {
    pub params: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MijiaActionResponse {
    pub success: bool,
    #[serde(default)]
    pub did: String,
    #[serde(default)]
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct MijiaSetPropRequest<T> {
    pub value: T,
}

const SPEAKER_DEVICE_ID: &str = "545918099";

fn get_client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))
}

fn get_config() -> Result<(String, String), String> {
    static CACHED: OnceLock<Option<(String, String)>> = OnceLock::new();

    let cached = CACHED.get_or_init(|| {
        let config = super::config::load_config();
        if config.mijia.api_base.is_empty() || config.mijia.api_key.is_empty() {
            None
        } else {
            log::info!("Mijia config cached: {}", config.mijia.api_base);
            Some((config.mijia.api_base, config.mijia.api_key))
        }
    });

    cached.as_ref().cloned().ok_or_else(|| "Mijia API not configured".to_string())
}

/// Execute device action
#[tauri::command]
pub fn execute_device_action(action: String, params: Option<Vec<String>>) -> Result<MijiaActionResponse, String> {
    let (api_base, api_key) = get_config()?;
    let client = get_client()?;

    let url = format!("{}/api/devices/{}/actions/{}", api_base, SPEAKER_DEVICE_ID, action);

    let request_body = if let Some(p) = params {
        MijiaActionRequest {
            params: Some(p.into_iter().map(|s| serde_json::json!(s)).collect()),
        }
    } else {
        MijiaActionRequest { params: None }
    };

    let body = serde_json::to_string(&request_body).map_err(|e| format!("JSON encode error: {}", e))?;

    let response = client
        .post(&url)
        .header("X-API-Key", api_key)
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("API error: {}", response.status()));
    }

    let text = response.text().map_err(|e| format!("Read response error: {}", e))?;
    serde_json::from_str::<MijiaActionResponse>(&text).map_err(|e| format!("Parse error: {}", e))
}

/// Get device property
#[tauri::command]
pub fn get_device_prop(prop: String) -> Result<serde_json::Value, String> {
    let (api_base, api_key) = get_config()?;
    let client = get_client()?;

    let url = format!("{}/api/devices/{}/props/{}", api_base, SPEAKER_DEVICE_ID, prop);

    let response = client
        .get(&url)
        .header("X-API-Key", api_key)
        .send()
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("API error: {}", response.status()));
    }

    let text = response.text().map_err(|e| format!("Read response error: {}", e))?;
    serde_json::from_str::<serde_json::Value>(&text).map_err(|e| format!("Parse error: {}", e))
}

/// Set device property
#[tauri::command]
pub fn set_device_prop(prop: String, value: serde_json::Value) -> Result<serde_json::Value, String> {
    let (api_base, api_key) = get_config()?;
    let client = get_client()?;

    let url = format!("{}/api/devices/{}/props/{}", api_base, SPEAKER_DEVICE_ID, prop);

    let request_body = MijiaSetPropRequest { value };

    let body = serde_json::to_string(&request_body).map_err(|e| format!("JSON encode error: {}", e))?;

    let response = client
        .put(&url)
        .header("X-API-Key", api_key)
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("API error: {}", response.status()));
    }

    let text = response.text().map_err(|e| format!("Read response error: {}", e))?;
    serde_json::from_str::<serde_json::Value>(&text).map_err(|e| format!("Parse error: {}", e))
}

/// Get playback state
#[tauri::command]
pub fn get_playback_state() -> Result<String, String> {
    let (api_base, api_key) = get_config()?;
    let client = get_client()?;

    let url = format!("{}/api/devices/{}/playback-state", api_base, SPEAKER_DEVICE_ID);

    let response = client
        .get(&url)
        .header("X-API-Key", api_key)
        .send()
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("API error: {}", response.status()));
    }

    let text = response.text().map_err(|e| format!("Read response error: {}", e))?;
    let data: serde_json::Value = serde_json::from_str(&text).map_err(|e| format!("Parse error: {}", e))?;
    data["state"]
        .as_str()
        .ok_or_else(|| "Missing state field".to_string())
        .map(|s: &str| s.to_string())
}
