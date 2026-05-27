//! MCP stdio 服务器（JSON-RPC 2.0，2024-11-05 子集）。

use crate::api::eastmoney::EastMoneyClient;
use crate::application::OutputProfile;
use crate::cache::FundCache;
use crate::nav_cache::NavCache;
use crate::schema::generate_agent_tools;
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::executor::{execute_tool, McpEnv};
use super::protocol::{
    InitializeCapabilities, InitializeResult, JsonRpcRequest, JsonRpcResponse, McpTool, ServerInfo,
    ToolCallResult, ToolContent, ToolsListResult,
};

pub struct McpServer<'a> {
    profile: OutputProfile,
    offline: bool,
    watchlist_path: &'a Path,
    client: &'a EastMoneyClient,
    name_cache: &'a Arc<Mutex<FundCache>>,
    nav_store: &'a NavCache,
}

impl<'a> McpServer<'a> {
    pub fn new(
        profile: OutputProfile,
        offline: bool,
        watchlist_path: &'a Path,
        client: &'a EastMoneyClient,
        name_cache: &'a Arc<Mutex<FundCache>>,
        nav_store: &'a NavCache,
    ) -> Self {
        Self {
            profile,
            offline,
            watchlist_path,
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
                ))
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
            "ping" => json!({}),
            _ => {
                return Some(JsonRpcResponse::err(
                    id,
                    -32601,
                    format!("Method not found: {}", req.method),
                ))
            }
        };

        Some(JsonRpcResponse::ok(id, result))
    }

    fn handle_initialize(&mut self) -> Value {
        let result = InitializeResult {
            protocol_version: "2024-11-05",
            capabilities: InitializeCapabilities { tools: json!({}) },
            server_info: ServerInfo {
                name: "fanalyzer",
                version: env!("CARGO_PKG_VERSION"),
            },
        };
        serde_json::to_value(result).expect("initialize serializes")
    }

    fn handle_tools_list(&self) -> Value {
        let tools: Vec<McpTool> = generate_agent_tools()
            .into_iter()
            .map(|t| McpTool {
                name: t.name,
                description: t.description,
                input_schema: serde_json::to_value(t.input_schema).unwrap_or(json!({})),
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
        let result = ToolCallResult {
            content: vec![ToolContent {
                content_type: "text",
                text,
            }],
            is_error: is_error.then_some(true),
        };
        serde_json::to_value(result).expect("tool call serializes")
    }
}
