//! MCP Resources（只读上下文：schema 索引、自选、组合、配置摘要）。

use crate::config::AppConfig;
use serde_json::json;
use std::path::Path;

use super::protocol::{McpResource, ResourceContent, ResourceReadResult, ResourcesListResult};

pub const URI_SCHEMAS_INDEX: &str = "fanalyzer://schemas/index";
pub const URI_WATCHLIST: &str = "fanalyzer://watchlist";
pub const URI_PORTFOLIO: &str = "fanalyzer://portfolio";
pub const URI_CONFIG: &str = "fanalyzer://config";

pub fn list_resources() -> ResourcesListResult {
    ResourcesListResult {
        resources: vec![
            McpResource {
                uri: URI_SCHEMAS_INDEX.into(),
                name: "Schema Index".into(),
                description: Some("schemas/index.json：工具与响应 schema 索引".into()),
                mime_type: Some("application/json".into()),
            },
            McpResource {
                uri: URI_WATCHLIST.into(),
                name: "Watchlist".into(),
                description: Some("当前自选基金 TOML".into()),
                mime_type: Some("text/plain".into()),
            },
            McpResource {
                uri: URI_PORTFOLIO.into(),
                name: "Portfolio".into(),
                description: Some("组合权重 TOML（默认 config/portfolio.toml）".into()),
                mime_type: Some("text/plain".into()),
            },
            McpResource {
                uri: URI_CONFIG.into(),
                name: "Config Summary".into(),
                description: Some("生效配置摘要（不含 proxy / token）".into()),
                mime_type: Some("application/json".into()),
            },
        ],
    }
}

pub fn read_resource(
    uri: &str,
    schema_root: &Path,
    watchlist_path: &Path,
    portfolio_path: &Path,
    config: &AppConfig,
) -> Result<ResourceReadResult, String> {
    let (mime_type, text) = match uri {
        URI_SCHEMAS_INDEX => {
            let path = schema_root.join("index.json");
            read_text_file(&path, "application/json")?
        }
        URI_WATCHLIST => read_text_file(watchlist_path, "text/plain")?,
        URI_PORTFOLIO => read_text_file(portfolio_path, "text/plain")?,
        URI_CONFIG => ("application/json".into(), config_summary(config)),
        other => return Err(format!("未知资源 URI：{other}")),
    };
    Ok(ResourceReadResult {
        contents: vec![ResourceContent {
            uri: uri.to_string(),
            mime_type: Some(mime_type),
            text: Some(text),
        }],
    })
}

fn read_text_file(path: &Path, mime: &str) -> Result<(String, String), String> {
    std::fs::read_to_string(path)
        .map(|text| (mime.to_string(), text))
        .map_err(|e| format!("读取 {} 失败：{e}", path.display()))
}

fn config_summary(config: &AppConfig) -> String {
    let summary = json!({
        "api": {
            "base_url": config.api.base_url,
            "timeout_secs": config.api.timeout_secs,
        },
        "log": {
            "level": config.log.level,
        },
        "cache": {
            "root": config.cache_root().to_string_lossy(),
        },
    });
    serde_json::to_string_pretty(&summary).unwrap_or_else(|_| "{}".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::path::PathBuf;

    #[test]
    fn list_resources_includes_four_uris() {
        let list = list_resources();
        assert_eq!(list.resources.len(), 4);
        let uris: Vec<_> = list.resources.iter().map(|r| r.uri.as_str()).collect();
        assert!(uris.contains(&URI_SCHEMAS_INDEX));
        assert!(uris.contains(&URI_CONFIG));
    }

    #[test]
    fn read_config_summary_is_json_without_proxy() {
        let config = AppConfig::default();
        let result = read_resource(
            URI_CONFIG,
            Path::new("schemas"),
            Path::new("config/watchlist.toml"),
            Path::new("config/portfolio.toml"),
            &config,
        )
        .unwrap();
        let text = result.contents[0].text.as_ref().unwrap();
        let v: Value = serde_json::from_str(text).unwrap();
        assert!(v.get("api").is_some());
        assert!(v["api"].get("proxy").is_none());
    }

    #[test]
    fn read_schemas_index_from_repo() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("schemas");
        let result = read_resource(
            URI_SCHEMAS_INDEX,
            &root,
            Path::new("config/watchlist.toml"),
            Path::new("config/portfolio.toml"),
            &AppConfig::default(),
        )
        .unwrap();
        assert!(
            result.contents[0]
                .text
                .as_ref()
                .unwrap()
                .contains("success_envelopes")
        );
    }
}
