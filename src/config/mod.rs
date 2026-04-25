use serde::Deserialize;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    ReadFailed(#[from] std::io::Error),
    #[error("Failed to parse config: {0}")]
    ParseFailed(#[from] toml::de::Error),
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub api: ApiConfig,
    pub log: LogConfig,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct ApiConfig {
    pub base_url: String,
    #[serde(default)]
    pub timeout_secs: u64,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct LogConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
}

fn default_log_level() -> String {
    "info".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            api: ApiConfig {
                base_url: "https://api.example.com".to_string(),
                timeout_secs: 30,
            },
            log: LogConfig {
                level: default_log_level(),
            },
        }
    }
}

impl AppConfig {
    pub fn load_from_file(path: &Path) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn load() -> Self {
        let config_path = Path::new("config/default.toml");
        if config_path.exists() {
            match Self::load_from_file(config_path) {
                Ok(config) => {
                    tracing::info!(path = %config_path.display(), "Loaded config from file");
                    return config;
                }
                Err(e) => {
                    tracing::warn!(error = %e, "Failed to load config, using defaults");
                }
            }
        }
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert!(!config.api.base_url.is_empty());
        assert_eq!(config.log.level, "info");
    }

    #[test]
    fn test_load_missing_file() {
        let config = AppConfig::load_from_file(Path::new("nonexistent.toml"));
        assert!(config.is_err());
    }
}
