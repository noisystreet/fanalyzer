use serde::Deserialize;
use std::path::{Path, PathBuf};
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
    #[serde(default)]
    pub cache: CacheConfig,
}

fn default_timeout_secs() -> u64 {
    30
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct CacheConfig {
    /// 缓存根目录（默认 `dirs::cache_dir()/fanalyzer`）
    #[serde(default)]
    pub root: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct ApiConfig {
    pub base_url: String,
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
    /// 覆盖默认 UA；敏感环境可通过环境变量在部署层注入。
    #[serde(default)]
    pub user_agent: Option<String>,
    /// HTTP/HTTPS 代理，如 `http://127.0.0.1:7890`
    #[serde(default)]
    pub proxy: Option<String>,
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
                timeout_secs: default_timeout_secs(),
                user_agent: None,
                proxy: None,
            },
            log: LogConfig {
                level: default_log_level(),
            },
            cache: CacheConfig::default(),
        }
    }
}

impl AppConfig {
    pub fn cache_root(&self) -> std::path::PathBuf {
        self.cache
            .root
            .as_ref()
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                dirs::cache_dir()
                    .unwrap_or_else(|| std::path::PathBuf::from(".cache"))
                    .join("fanalyzer")
            })
    }

    pub fn load_from_file(path: &Path) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// 按优先级解析配置文件路径：`--config` / `FANALYZER_CONFIG` → CWD → 可执行文件相对路径。
    pub fn discover_config_path(explicit: Option<&Path>) -> Option<PathBuf> {
        if let Some(path) = explicit {
            if path.exists() {
                return Some(path.to_path_buf());
            }
            tracing::warn!(path = %path.display(), "Explicit config file not found");
        }

        let cwd_config = PathBuf::from("config/default.toml");
        if cwd_config.exists() {
            return Some(cwd_config);
        }

        if let Ok(exe) = std::env::current_exe()
            && let Some(exe_dir) = exe.parent()
        {
            for rel in [
                "config/default.toml",
                "../config/default.toml",
                "../../config/default.toml",
            ] {
                let candidate = exe_dir.join(rel);
                if candidate.exists() {
                    return candidate.canonicalize().ok().or(Some(candidate));
                }
            }
        }

        None
    }

    pub fn load(explicit: Option<&Path>) -> Self {
        if let Some(path) = Self::discover_config_path(explicit) {
            match Self::load_from_file(&path) {
                Ok(config) => {
                    tracing::info!(path = %path.display(), "Loaded config from file");
                    return config;
                }
                Err(e) => {
                    tracing::warn!(error = %e, path = %path.display(), "Failed to load config, using defaults");
                }
            }
        }
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

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

    #[test]
    fn parse_optional_proxy_and_ua_from_toml() {
        let s = r#"
[api]
base_url = "https://example.invalid"
timeout_secs = 60
user_agent = "CustomUA/1.0"
proxy = "http://127.0.0.1:7890"

[log]
level = "info"
"#;
        let c: AppConfig = toml::from_str(s).unwrap();
        assert_eq!(c.api.proxy.as_deref(), Some("http://127.0.0.1:7890"));
        assert_eq!(c.api.user_agent.as_deref(), Some("CustomUA/1.0"));
    }

    #[test]
    fn discover_prefers_explicit_config() {
        let dir = tempdir().unwrap();
        let cfg_path = dir.path().join("custom.toml");
        fs::write(
            &cfg_path,
            r#"
[api]
base_url = "https://example.invalid"

[log]
level = "debug"
"#,
        )
        .unwrap();
        let found = AppConfig::discover_config_path(Some(&cfg_path)).unwrap();
        assert_eq!(found, cfg_path);
        let loaded = AppConfig::load(Some(&cfg_path));
        assert_eq!(loaded.log.level, "debug");
    }

    #[test]
    fn load_uses_cache_root_from_file() {
        let dir = tempdir().unwrap();
        let cfg_path = dir.path().join("cfg.toml");
        let cache_root = dir.path().join("data-cache");
        fs::write(
            &cfg_path,
            format!(
                r#"
[api]
base_url = "https://example.invalid"

[log]
level = "info"

[cache]
root = "{}"
"#,
                cache_root.display()
            ),
        )
        .unwrap();
        let loaded = AppConfig::load(Some(&cfg_path));
        assert_eq!(loaded.cache_root(), cache_root);
    }
}
