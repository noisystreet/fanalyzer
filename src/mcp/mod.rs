//! MCP Server（stdio，供 Cursor / Trae 等 Agent 客户端集成）。

mod executor;
mod protocol;
mod resources;
mod server;

use crate::api::eastmoney::{into_anyhow, EastMoneyClient, EastMoneyClientOptions};
use crate::application::OutputProfile;
use crate::cache::FundCache;
use crate::config::AppConfig;
use crate::nav_cache::NavCache;
use crate::schema::ToolTier;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

pub use server::McpServer;

/// `fanalyzer mcp` 子命令。
#[derive(clap::Subcommand, Debug)]
pub enum McpCommands {
    /// 启动 stdio MCP 服务器
    Serve {
        /// 输出 profile：summary / standard / full
        #[arg(long, default_value = "standard")]
        profile: String,
        /// 暴露工具集：minimal / standard / full
        #[arg(long, default_value = "full", value_name = "TIER")]
        tools: String,
        /// 仅从本地缓存读取
        #[arg(long)]
        offline: bool,
        #[arg(long, default_value = "config/watchlist.toml", value_name = "PATH")]
        watchlist_file: PathBuf,
        #[arg(long, default_value = "config/portfolio.toml", value_name = "PATH")]
        portfolio_file: PathBuf,
    },
}

pub async fn run(cmd: McpCommands, config: AppConfig) -> anyhow::Result<()> {
    match cmd {
        McpCommands::Serve {
            profile,
            tools,
            offline,
            watchlist_file,
            portfolio_file,
        } => {
            let profile = OutputProfile::parse(&profile)?;
            let tool_tier = ToolTier::parse(&tools)?;
            let opts = EastMoneyClientOptions {
                timeout_secs: config.api.timeout_secs.max(1),
                user_agent: config.api.user_agent.clone(),
                proxy: config.api.proxy.clone(),
            };
            let client = EastMoneyClient::with_options(opts).map_err(into_anyhow)?;
            let cache_root = config.cache_root();
            let name_cache = Arc::new(Mutex::new(FundCache::with_root(cache_root.clone())));
            let nav_store = NavCache::with_root(cache_root);
            let mut server = McpServer::new(
                profile,
                offline,
                tool_tier,
                &watchlist_file,
                portfolio_file,
                config,
                &client,
                &name_cache,
                &nav_store,
            );
            tracing::info!(tools = ?tool_tier, "MCP server listening on stdio");
            server.run_stdio().await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::protocol::{JsonRpcRequest, JsonRpcResponse};
    use serde_json::json;

    #[test]
    fn parse_initialize_request() {
        let req: JsonRpcRequest =
            serde_json::from_str(r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#)
                .unwrap();
        assert_eq!(req.method, "initialize");
    }

    #[test]
    fn response_ok_serializes() {
        let resp = JsonRpcResponse::ok(json!(1), json!({"tools": []}));
        let s = serde_json::to_string(&resp).unwrap();
        assert!(s.contains("\"result\""));
    }
}
