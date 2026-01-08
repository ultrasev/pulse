use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub upload: UploadConfig,
    #[serde(default)]
    pub mijia: MijiaConfig,
}

#[derive(Debug, Deserialize)]
pub struct UploadConfig {
    pub url: String,
    pub token: String,
    #[serde(default)]
    pub base_url: String,
}

impl Default for UploadConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            token: String::new(),
            base_url: String::new(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MijiaConfig {
    pub api_base: String,
    pub api_key: String,
}

impl Default for MijiaConfig {
    fn default() -> Self {
        Self {
            api_base: String::new(),
            api_key: String::new(),
        }
    }
}

/// Get config file path: ~/.config/pulse/config.toml (preferred) or ~/Library/Application Support/pulse/config.toml
pub fn get_config_path() -> PathBuf {
    // Prefer ~/.config/pulse/config.toml (Unix-style)
    if let Some(home) = dirs::home_dir() {
        let unix_style = home.join(".config").join("pulse").join("config.toml");
        if unix_style.exists() {
            return unix_style;
        }
    }

    // Fallback to macOS standard location
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("pulse")
        .join("config.toml")
}

/// Load config from file
pub fn load_config() -> Config {
    let config_path = get_config_path();

    if !config_path.exists() {
        log::warn!("Config file not found at {:?}", config_path);
        return Config::default();
    }

    match fs::read_to_string(&config_path) {
        Ok(contents) => {
            match toml::from_str(&contents) {
                Ok(config) => {
                    log::info!("Config loaded from {:?}", config_path);
                    config
                }
                Err(e) => {
                    log::error!("Failed to parse config: {}", e);
                    Config::default()
                }
            }
        }
        Err(e) => {
            log::error!("Failed to read config file: {}", e);
            Config::default()
        }
    }
}

/// Get mijia config for frontend
#[tauri::command]
pub fn get_mijia_config() -> MijiaConfig {
    load_config().mijia
}
