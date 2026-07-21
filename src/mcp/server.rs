//! MCP stdio 服务器（JSON-RPC 2.0，2024-11-05 子集）。

use crate::api::eastmoney::EastMoneyClient;
use crate::application::OutputProfile;
use crate::cache::FundCache;
use crate::config::AppConfig;
use crate::nav_cache::NavCache;
use crate::schema::{ToolTier, discover_schema_root, filter_agent_tools, resolve_output_schema};
use serde_json::{Value, json};
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

use super::executor::{McpEnv, execute_tool};
use super::protocol::{
    InitializeCapabilities, InitializeResult, JsonRpcRequest, JsonRpcResponse, McpTool, ServerInfo,
    ToolCallResult, ToolsListResult,
};
use super::resources;

pub struct McpServer<'a> {
    profile: OutputProfile,
    offline: bool,
    tool_tier: ToolTier,
    watchlist_path: &'a Path,
    portfolio_path: PathBuf,
    schema_root: PathBuf,
    config: AppConfig,
    client: &'a EastMoneyClient,
    name_cache: &'a Arc<Mutex<FundCache>>,
    nav_store: &'a NavCache,
}

impl<'a> McpServer<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        profile: OutputProfile,
        offline: bool,
        tool_tier: ToolTier,
        watchlist_path: &'a Path,
        portfolio_path: PathBuf,
        config: AppConfig,
        client: &'a EastMoneyClient,
        name_cache: &'a Arc<Mutex<FundCache>>,
        nav_store: &'a NavCache,
    ) -> Self {
        Self {
            profile,
            offline,
            tool_tier,
            watchlist_path,
            portfolio_path,
            schema_root: discover_schema_root(),
            config,
            client,
            name_cache,
            nav_store,
        }
    }

    pub async fn run_stdio(&mut self) -> anyhow::Result<()> {
        let stdin = io::stdin();
        let mut stdout = io::stdout();
        for line in stdin.lock().lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let response = self.handle_line(&line).await;
            if let Some(resp) = response {
                let out = serde_json::to_string(&resp)?;
                writeln!(stdout, "{out}")?;
                stdout.flush()?;
            }
        }
        Ok(())
    }

    async fn handle_line(&mut self, line: &str) -> Option<JsonRpcResponse> {
        let req: JsonRpcRequest = match serde_json::from_str(line) {
            Ok(r) => r,
            Err(e) => {
                return Some(JsonRpcResponse::err(
                    Value::Null,
                    -32700,
                    format!("Parse error: {e}"),
                ));
            }
        };
        let id = req.id.clone().unwrap_or(Value::Null);

        if req.method == "notifications/initialized" {
            return None;
        }

        let result = match req.method.as_str() {
            "initialize" => self.handle_initialize(),
            "tools/list" => self.handle_tools_list(),
            "tools/call" => self.handle_tools_call(&req.params).await,
            "resources/list" => self.handle_resources_list(),
            "resources/read" => self.handle_resources_read(&req.params),
            "ping" => json!({}),
            _ => {
                return Some(JsonRpcResponse::err(
                    id,
                    -32601,
                    format!("Method not found: {}", req.method),
                ));
            }
        };

        Some(JsonRpcResponse::ok(id, result))
    }

    fn handle_initialize(&mut self) -> Value {
        let result = InitializeResult {
            protocol_version: "2024-11-05",
            capabilities: InitializeCapabilities {
                tools: json!({}),
                resources: Some(json!({})),
            },
            server_info: ServerInfo {
                name: "fanalyzer",
                version: env!("CARGO_PKG_VERSION"),
            },
        };
        serde_json::to_value(result).expect("initialize serializes")
    }

    fn handle_tools_list(&self) -> Value {
        let tools: Vec<McpTool> = filter_agent_tools(self.tool_tier)
            .into_iter()
            .map(|tool| McpTool {
                name: tool.name.clone(),
                description: tool.description.clone(),
                input_schema: serde_json::to_value(&tool.input_schema).unwrap_or(json!({})),
                output_schema: resolve_output_schema(&tool, &self.schema_root),
            })
            .collect();
        serde_json::to_value(ToolsListResult { tools }).expect("tools list serializes")
    }

    async fn handle_tools_call(&self, params: &Value) -> Value {
        let name = params
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let args = params.get("arguments").cloned().unwrap_or(json!({}));
        let env = McpEnv {
            profile: self.profile,
            offline: self.offline,
            watchlist_path: self.watchlist_path,
            client: self.client,
            name_cache: self.name_cache,
            nav_store: self.nav_store,
        };
        let (text, is_error) = execute_tool(&env, name, args).await;
        let result = ToolCallResult::from_envelope_json(text, is_error);
        serde_json::to_value(result).expect("tool call serializes")
    }

    fn handle_resources_list(&self) -> Value {
        serde_json::to_value(resources::list_resources()).expect("resources list serializes")
    }

    fn handle_resources_read(&self, params: &Value) -> Value {
        let uri = params
            .get("uri")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        match resources::read_resource(
            uri,
            &self.schema_root,
            self.watchlist_path,
            &self.portfolio_path,
            &self.config,
        ) {
            Ok(result) => serde_json::to_value(result).expect("resource read serializes"),
            Err(message) => json!({
                "contents": [],
                "error": message,
            }),
        }
    }
}
