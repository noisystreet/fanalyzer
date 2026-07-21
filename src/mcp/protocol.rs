//! MCP JSON-RPC 消息类型（2024-11-05 子集）。

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    #[allow(dead_code)]
    pub jsonrpc: Option<String>,
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: &'static str,
    pub id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcResponse {
    pub fn ok(id: Value, result: Value) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn err(id: Value, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
                data: None,
            }),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResult {
    pub protocol_version: &'static str,
    pub capabilities: InitializeCapabilities,
    pub server_info: ServerInfo,
}

#[derive(Debug, Serialize)]
pub struct InitializeCapabilities {
    pub tools: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct ServerInfo {
    pub name: &'static str,
    pub version: &'static str,
}

#[derive(Debug, Serialize)]
pub struct ToolsListResult {
    pub tools: Vec<McpTool>,
}

#[derive(Debug, Serialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
    #[serde(rename = "outputSchema", skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct ToolCallResult {
    pub content: Vec<ToolContent>,
    /// 与 `content[0].text` 同构的 JSON 信封；声明了 `outputSchema` 时客户端需要此字段。
    #[serde(rename = "structuredContent", skip_serializing_if = "Option::is_none")]
    pub structured_content: Option<Value>,
    #[serde(rename = "isError", skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct ToolContent {
    #[serde(rename = "type")]
    pub content_type: &'static str,
    pub text: String,
}

impl ToolCallResult {
    /// 从工具返回的信封 JSON 文本构建 MCP 结果（text + structuredContent 对齐）。
    pub fn from_envelope_json(text: String, is_error: bool) -> Self {
        let (structured, forced_error) = match serde_json::from_str::<Value>(&text) {
            Ok(v) => {
                let ok_false = v.get("ok").and_then(|ok| ok.as_bool()) == Some(false);
                (v, ok_false)
            }
            Err(_) => (
                serde_json::json!({
                    "v": 1,
                    "command": "mcp",
                    "ok": false,
                    "warnings": [],
                    "error": {
                        "code": "INVALID_OUTPUT",
                        "message": "工具未返回合法 JSON 信封",
                        "retryable": false,
                        "hint": "请升级 fanalyzer 或检查 MCP 执行路径"
                    }
                }),
                true,
            ),
        };
        let is_error = is_error || forced_error;
        let text = if text.trim().is_empty() {
            serde_json::to_string(&structured).unwrap_or_default()
        } else {
            text
        };
        Self {
            content: vec![ToolContent {
                content_type: "text",
                text,
            }],
            structured_content: Some(structured),
            is_error: is_error.then_some(true),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_envelope_json_sets_structured_content() {
        let text = r#"{"v":1,"command":"analyze","ok":true,"warnings":[],"data":{}}"#;
        let result = ToolCallResult::from_envelope_json(text.into(), false);
        assert!(result.structured_content.is_some());
        assert!(result.is_error.is_none());
        assert_eq!(
            result.structured_content.as_ref().unwrap()["command"],
            "analyze"
        );
    }

    #[test]
    fn from_envelope_json_ok_false_sets_is_error() {
        let text = r#"{"v":1,"command":"analyze","ok":false,"warnings":[],"error":{"code":"X","message":"m"}}"#;
        let result = ToolCallResult::from_envelope_json(text.into(), false);
        assert_eq!(result.is_error, Some(true));
    }

    #[test]
    fn from_envelope_json_invalid_text_wraps_failure() {
        let result = ToolCallResult::from_envelope_json("not-json".into(), false);
        assert_eq!(result.is_error, Some(true));
        let sc = result.structured_content.unwrap();
        assert_eq!(sc["ok"], false);
        assert_eq!(sc["error"]["code"], "INVALID_OUTPUT");
    }
}

#[derive(Debug, Serialize)]
pub struct ResourcesListResult {
    pub resources: Vec<McpResource>,
}

#[derive(Debug, Serialize)]
pub struct McpResource {
    pub uri: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ResourceReadResult {
    pub contents: Vec<ResourceContent>,
}

#[derive(Debug, Serialize)]
pub struct ResourceContent {
    pub uri: String,
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}
