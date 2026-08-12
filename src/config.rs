use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use log::info;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CheckType {
    Icmp,
    Tcp,
}

impl Default for CheckType {
    fn default() -> Self {
        CheckType::Icmp
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DeviceConfig {
    #[serde(default = "default_device_name")]
    pub name: String,
    
    #[serde(default)]
    pub target: String,
    
    #[serde(default)]
    pub check_type: CheckType,
    
    #[serde(default)]
    pub port: Option<u16>,
    
    #[serde(default)]
    pub token: String,
    
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_device_name() -> String {
    "New Device".to_string()
}

fn default_true() -> bool {
    true
}

fn default_interval() -> u64 {
    30
}

fn default_branch_name() -> String {
    "Main Branch".to_string()
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    #[serde(default = "default_branch_name")]
    pub branch_name: String,

    #[serde(default)]
    pub kuma_base_url: String,

    #[serde(default = "default_interval")]
    pub interval_sec: u64,

    #[serde(default)]
    pub autostart: bool,

    #[serde(default = "default_true")]
    pub enable_logging: bool,

    #[serde(default)]
    pub agent_push_token: String,

    #[serde(default)]
    pub devices: Vec<DeviceConfig>,

    // Legacy fields for backward compatibility with older config versions
    #[serde(default)]
    pub kuma_push_url: Option<String>,
    #[serde(default)]
    pub ping_target: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            branch_name: default_branch_name(),
            kuma_base_url: String::new(),
            interval_sec: default_interval(),
            autostart: false,
            enable_logging: true,
            agent_push_token: String::new(),
            devices: Vec::new(),
            kuma_push_url: None,
            ping_target: None,
        }
    }
}

pub fn get_app_data_dir() -> PathBuf {
    if let Ok(app_data) = env::var("APPDATA") {
        let dir = std::path::Path::new(&app_data).join("KumaAgent");
        if !dir.exists() {
            let _ = fs::create_dir_all(&dir);
        }
        dir
    } else {
        env::current_exe()
            .unwrap_or_default()
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .to_path_buf()
    }
}

pub fn get_config_path() -> PathBuf {
    get_app_data_dir().join("config.toml")
}

pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_path = get_config_path();
    info!("Loading configuration from: {:?}", config_path);

    if !config_path.exists() {
        info!("Config file not found, using default configuration.");
        return Ok(Config::default());
    }

    let config_content = fs::read_to_string(&config_path)?;
    let mut config: Config = toml::from_str(&config_content)?;

    // Migrate legacy config if needed
    if let Some(legacy_url) = config.kuma_push_url.take() {
        if !legacy_url.is_empty() && config.kuma_base_url.is_empty() && config.agent_push_token.is_empty() {
            if legacy_url.starts_with("http") {
                config.agent_push_token = legacy_url;
            }
        }
    }
    if let Some(legacy_target) = config.ping_target.take() {
        if !legacy_target.is_empty() && config.devices.is_empty() {
            config.devices.push(DeviceConfig {
                name: "Gateway / Ping Target".to_string(),
                target: legacy_target,
                check_type: CheckType::Icmp,
                port: None,
                token: String::new(),
                enabled: true,
            });
        }
    }

    info!("Configuration loaded successfully with {} devices.", config.devices.len());
    Ok(config)
}

pub fn save_config(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = get_config_path();
    let toml_string = toml::to_string_pretty(config)?;
    fs::write(&config_path, toml_string)?;
    info!("Configuration saved successfully to {:?}", config_path);
    Ok(())
}
