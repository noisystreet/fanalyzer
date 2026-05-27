//! 从 Clap 命令树自动生成 Agent / MCP 工具 JSON Schema。

use crate::cli::Cli;
use clap::{Arg, ArgAction, Command, CommandFactory};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::Path;

const SCHEMA_VERSION: &str = "1";
pub const TOOL_PREFIX: &str = "fanalyzer_";

/// MCP / OpenAI function calling 工具描述。
#[derive(Debug, serde::Serialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: InputSchema,
    #[serde(rename = "outputSchema", skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct InputSchema {
    #[serde(rename = "type")]
    pub schema_type: &'static str,
    pub properties: BTreeMap<String, Value>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub required: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// 生成 `json` 子命令下全部工具的 schema 数组。
pub fn generate_tools() -> Vec<ToolDefinition> {
    let root = Cli::command();
    let json_cmd = root
        .find_subcommand("json")
        .expect("cli must define json subcommand");
    let globals = global_arg_schemas(&root);

    json_cmd
        .get_subcommands()
        .map(|leaf| leaf_to_tool(json_cmd, leaf, &globals))
        .collect()
}

pub fn tools_json(pretty: bool) -> anyhow::Result<String> {
    let catalog = json!({
        "v": SCHEMA_VERSION,
        "generator": "fanalyzer schema tools",
        "tools": generate_tools(),
    });
    if pretty {
        Ok(serde_json::to_string_pretty(&catalog)?)
    } else {
        Ok(serde_json::to_string(&catalog)?)
    }
}

pub fn write_tools(path: &Path, pretty: bool) -> anyhow::Result<()> {
    let json = tools_json(pretty)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, json)?;
    Ok(())
}

fn leaf_to_tool(
    json_parent: &Command,
    leaf: &Command,
    globals: &BTreeMap<String, Value>,
) -> ToolDefinition {
    let command = leaf.get_name();
    let mut properties = globals.clone();
    merge_command_args(json_parent, &mut properties);
    merge_command_args(leaf, &mut properties);

    let required = required_keys(&properties, leaf);
    let about = leaf
        .get_about()
        .or_else(|| json_parent.get_about())
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("Run fanalyzer json {command}"));

    ToolDefinition {
        name: format!("{TOOL_PREFIX}{command}"),
        description: format!(
            "{about}（stdout 为 JSON 信封，见 schemas/responses/{command}.success.json）"
        ),
        input_schema: InputSchema {
            schema_type: "object",
            properties,
            required,
            description: Some(format!(
                "参数对应 CLI：`fanalyzer json [--compact] [--compact-series] {command} ...`"
            )),
        },
        output_schema: Some(format!("schemas/responses/{command}.success.json")),
    }
}

fn global_arg_schemas(root: &Command) -> BTreeMap<String, Value> {
    let mut map = BTreeMap::new();
    for arg in root.get_arguments().filter(|a| a.is_global_set()) {
        if let Some((key, prop)) = arg_to_property(arg) {
            map.insert(key, prop);
        }
    }
    map
}

fn merge_command_args(cmd: &Command, properties: &mut BTreeMap<String, Value>) {
    for arg in cmd.get_arguments() {
        if let Some((key, prop)) = arg_to_property(arg) {
            merge_property(properties, key, prop);
        }
    }
}

fn merge_property(properties: &mut BTreeMap<String, Value>, key: String, prop: Value) {
    if key == "code" {
        if let Some(existing) = properties.get("code") {
            let desc = prop
                .get("description")
                .or_else(|| existing.get("description"))
                .and_then(|v| v.as_str())
                .unwrap_or("基金代码或名称");
            properties.insert(
                "code".into(),
                json!({
                    "type": "string",
                    "description": desc,
                }),
            );
            return;
        }
    }
    properties.entry(key).or_insert(prop);
}

fn required_keys(properties: &BTreeMap<String, Value>, leaf: &Command) -> Vec<String> {
    let mut required = Vec::new();
    for arg in leaf.get_arguments() {
        if arg.is_required_set() {
            if let Some(key) = property_key(arg) {
                if properties.contains_key(&key) {
                    required.push(key);
                }
            }
        }
    }
    required.sort();
    required.dedup();
    required
}

fn property_key(arg: &Arg) -> Option<String> {
    let id = arg.get_id().as_str();
    if is_skipped_arg(id) {
        return None;
    }
    if id == "positional" || id == "flag" {
        return Some("code".into());
    }
    arg.get_long()
        .map(|s| s.to_string())
        .or_else(|| Some(id.to_string()))
}

fn is_skipped_arg(id: &str) -> bool {
    matches!(id, "help" | "version" | "subcommand")
}

fn arg_to_property(arg: &Arg) -> Option<(String, Value)> {
    let key = property_key(arg)?;
    let help = arg.get_help().map(|h| h.to_string());
    let default = arg
        .get_default_values()
        .first()
        .map(|v| v.to_string_lossy().to_string());

    let mut prop = match arg.get_action() {
        ArgAction::SetTrue | ArgAction::SetFalse => json!({ "type": "boolean" }),
        ArgAction::Count => json!({ "type": "integer", "minimum": 0 }),
        ArgAction::Append | ArgAction::Set => infer_value_schema(arg, default.as_deref()),
        _ => json!({ "type": "string" }),
    };

    if let Some(map) = prop.as_object_mut() {
        if let Some(h) = help {
            map.insert("description".into(), json!(h));
        }
        if let Some(d) = default {
            map.insert("default".into(), json!(d));
        }
    }

    Some((key, prop))
}

fn infer_value_schema(arg: &Arg, default: Option<&str>) -> Value {
    if arg.get_value_names().is_some_and(|names| {
        names
            .iter()
            .any(|n| n.contains("PATH") || n.contains("FILE"))
    }) {
        return json!({ "type": "string" });
    }
    if let Some(d) = default {
        if d.parse::<i64>().is_ok() {
            return json!({ "type": "integer" });
        }
        if d.parse::<f64>().is_ok() {
            return json!({ "type": "number" });
        }
    }
    let id = arg.get_id().as_str();
    if id.ends_with("_top")
        || id == "days"
        || id == "limit"
        || id == "top"
        || id == "deep_limit"
        || id == "holdings_top"
        || id == "industry_top"
        || id == "rolling_window"
        || id == "rank_top"
    {
        return json!({ "type": "integer" });
    }
    if id.starts_with("min_") || id.starts_with("max_") {
        return json!({ "type": "number" });
    }
    if id == "codes" {
        return json!({
            "type": "array",
            "items": { "type": "string" },
            "description": "基金代码或名称列表"
        });
    }
    json!({ "type": "string" })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_analyze_tool_with_code_and_days() {
        let tools = generate_tools();
        let analyze = tools
            .iter()
            .find(|t| t.name == "fanalyzer_analyze")
            .expect("analyze tool");
        assert!(analyze.input_schema.properties.contains_key("code"));
        assert!(analyze.input_schema.properties.contains_key("days"));
        assert!(analyze.input_schema.properties.contains_key("compact"));
        assert!(analyze.input_schema.properties.contains_key("offline"));
    }

    #[test]
    fn tools_json_is_valid_json() {
        let raw = tools_json(true).unwrap();
        let v: Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(v["v"], SCHEMA_VERSION);
        assert!(v["tools"].as_array().unwrap().len() >= 10);
    }

    #[test]
    fn schema_command_not_in_tools_list() {
        let tools = generate_tools();
        assert!(!tools.iter().any(|t| t.name.contains("schema")));
    }
}
