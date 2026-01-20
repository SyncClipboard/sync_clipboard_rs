use serde::{Deserialize, Serialize};
use config::{Config as ConfigLoader, ConfigError, File};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub client: ClientConfig,
    pub auth: AuthConfig,
    pub history: HistoryConfig,
    pub general: GeneralConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub enabled: bool,
    pub port: u16,
    pub host: String,
    pub wevdav_enabled: bool,
    pub tls: Option<TlsConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClientConfig {
    pub enabled: bool,
    pub remote_host: String,
    pub remote_port: u16,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TlsConfig {
    pub cert: String,
    pub key: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeneralConfig {
    pub device_name: String,
    pub device_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthConfig {
    pub username: Option<String>,
    pub password: Option<String>,
    pub token: Option<String>,
    pub encrypt_password: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HistoryConfig {
    pub max_count: u32,
    pub log_retention_days: u64,
    #[serde(default = "default_db_path")]
    pub db_path: String,
}

fn default_db_path() -> String {
    "history.db".to_string()
}

fn get_config_path() -> PathBuf {
    PathBuf::from("config.toml")
}

impl Config {
    pub fn new() -> Result<Self, ConfigError> {
        let config_path = get_config_path();
        
        let s = ConfigLoader::builder()
            // Start off with default values
            .set_default("server.enabled", true)?
            .set_default("server.port", 5033)?
            .set_default("server.host", "0.0.0.0")?
            .set_default("server.wevdav_enabled", false)?
            .set_default("client.enabled", true)?
            .set_default("client.remote_host", "127.0.0.1")?
            .set_default("client.remote_port", 5033)?
            .set_default("general.device_name", hostname::get().ok().and_then(|s| s.into_string().ok()).unwrap_or_else(|| "Desktop".to_string()))?
            .set_default("general.device_id", uuid::Uuid::new_v4().to_string())?
            .set_default("auth.username", Option::<String>::None)?
            .set_default("auth.password", Option::<String>::None)?
            .set_default("auth.token", Option::<String>::None)?
            .set_default("auth.encrypt_password", Option::<String>::None)?
            .set_default("history.max_count", 100)?
            .set_default("history.log_retention_days", 7)?
            .set_default("history.db_path", default_db_path())?
            // Add in settings from the environment
            .add_source(config::Environment::with_prefix("SYNCCLIPBOARD").separator("_"))
            // Load from config.toml if exists (relative path for backward compatibility)
            .add_source(File::from(config_path.clone()).required(false))
            .build()?;

        let config: Self = s.try_deserialize()?;
        
        // 如果config文件不存在，生成并保存初始配置（包含设备ID）
        if !config_path.exists() {
            let _ = config.save();
        }
        
        Ok(config)
    }

    pub fn save(&self) -> Result<(), String> {
        let config_path = get_config_path();
        let toml_str = toml::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(config_path, toml_str).map_err(|e| e.to_string())?;
        Ok(())
    }
}
