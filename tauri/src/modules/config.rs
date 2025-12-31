use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub upload: UploadConfig,
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

/// Get config file path: ~/.config/pulse/config.toml
pub fn get_config_path() -> PathBuf {
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

/// Create default config file if not exists
#[tauri::command]
pub fn init_config() -> Result<String, String> {
    let config_path = get_config_path();

    if config_path.exists() {
        return Ok(format!("Config already exists at {:?}", config_path));
    }

    // Create parent directory
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create config dir: {}", e))?;
    }

    let default_config = r#"[upload]
url = "https://your-upload-server.com/api/image"
token = "your-token-here"
base_url = "https://your-upload-server.com"
"#;

    fs::write(&config_path, default_config)
        .map_err(|e| format!("Failed to write config: {}", e))?;

    Ok(format!("Config created at {:?}", config_path))
}

/// Get config file path (for frontend)
#[tauri::command]
pub fn get_config_file_path() -> String {
    get_config_path().to_string_lossy().to_string()
}
