//! Agent 专用工具 schema（剥离 CLI 内部参数）。

use super::tools::{generate_tools, InputSchema, ToolDefinition, TOOL_PREFIX};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;

const AGENT_EXCLUDED: &[&str] = &[
    "compact",
    "compact-series",
    "watchlist-file",
    "format",
    "output",
    "output-dir",
    "profile",
];

/// 生成 Agent / MCP 暴露的工具列表（不含 CLI 内部参数）。
pub fn generate_agent_tools() -> Vec<ToolDefinition> {
    let mut tools: Vec<ToolDefinition> = generate_tools()
        .into_iter()
        .map(filter_agent_tool)
        .collect();
    tools.extend(composite_tools());
    tools.extend(agent_only_tools());
    tools
}

fn filter_agent_tool(mut tool: ToolDefinition) -> ToolDefinition {
    tool.input_schema
        .properties
        .retain(|k, _| !AGENT_EXCLUDED.contains(&k.as_str()));
    tool.input_schema
        .required
        .retain(|k| !AGENT_EXCLUDED.contains(&k.as_str()));
    tool.description = tool
        .description
        .split('（')
        .next()
        .unwrap_or(&tool.description)
        .trim()
        .to_string();
    tool
}

fn composite_tools() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: format!("{TOOL_PREFIX}research_fund"),
            description: "单基金研究：概况 + 分析 + 行业 + 重仓（复合工具，减少 Agent 步数）"
                .into(),
            input_schema: InputSchema {
                schema_type: "object",
                properties: BTreeMap::from([
                    (
                        "code".into(),
                        serde_json::json!({"type": "string", "description": "基金代码或名称"}),
                    ),
                    (
                        "days".into(),
                        serde_json::json!({"type": "integer", "default": 90, "description": "分析窗口（天）"}),
                    ),
                    (
                        "offline".into(),
                        serde_json::json!({"type": "boolean", "description": "仅使用本地缓存"}),
                    ),
                ]),
                required: vec!["code".into()],
                description: Some("等价于依次调用 info、analyze、sectors、holdings".into()),
            },
            output_schema: None,
        },
        ToolDefinition {
            name: format!("{TOOL_PREFIX}compare_watchlist"),
            description: "对比自选列表中全部基金".into(),
            input_schema: InputSchema {
                schema_type: "object",
                properties: BTreeMap::from([
                    (
                        "days".into(),
                        serde_json::json!({"type": "integer", "default": 90}),
                    ),
                    ("offline".into(), serde_json::json!({"type": "boolean"})),
                ]),
                required: vec![],
                description: None,
            },
            output_schema: Some("schemas/responses/compare.success.json".into()),
        },
    ]
}

fn agent_only_tools() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: format!("{TOOL_PREFIX}watchlist_list"),
            description: "列出当前自选基金".into(),
            input_schema: InputSchema {
                schema_type: "object",
                properties: BTreeMap::new(),
                required: vec![],
                description: None,
            },
            output_schema: Some("schemas/responses/watchlist.success.json".into()),
        },
        ToolDefinition {
            name: format!("{TOOL_PREFIX}watchlist_add"),
            description: "向自选列表追加基金代码".into(),
            input_schema: InputSchema {
                schema_type: "object",
                properties: BTreeMap::from([(
                    "codes".into(),
                    serde_json::json!({
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "基金代码列表"
                    }),
                )]),
                required: vec!["codes".into()],
                description: None,
            },
            output_schema: Some("schemas/responses/watchlist.success.json".into()),
        },
        ToolDefinition {
            name: format!("{TOOL_PREFIX}watchlist_remove"),
            description: "从自选列表移除基金代码".into(),
            input_schema: InputSchema {
                schema_type: "object",
                properties: BTreeMap::from([(
                    "codes".into(),
                    serde_json::json!({
                        "type": "array",
                        "items": {"type": "string"},
                    }),
                )]),
                required: vec!["codes".into()],
                description: None,
            },
            output_schema: Some("schemas/responses/watchlist.success.json".into()),
        },
        ToolDefinition {
            name: format!("{TOOL_PREFIX}portfolio_config"),
            description: "读取组合权重配置".into(),
            input_schema: InputSchema {
                schema_type: "object",
                properties: BTreeMap::from([(
                    "portfolio-file".into(),
                    serde_json::json!({
                        "type": "string",
                        "default": "config/portfolio.toml",
                        "description": "组合 TOML 路径"
                    }),
                )]),
                required: vec![],
                description: None,
            },
            output_schema: Some("schemas/responses/portfolio_config.success.json".into()),
        },
    ]
}

pub fn agent_tools_json(pretty: bool) -> anyhow::Result<String> {
    let catalog = serde_json::json!({
        "v": "1",
        "generator": "fanalyzer schema agent-tools",
        "tools": generate_agent_tools(),
    });
    if pretty {
        Ok(serde_json::to_string_pretty(&catalog)?)
    } else {
        Ok(serde_json::to_string(&catalog)?)
    }
}

pub fn write_agent_tools(path: &Path, pretty: bool) -> anyhow::Result<()> {
    let json = agent_tools_json(pretty)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, json)?;
    Ok(())
}

/// 将 outputSchema 路径替换为内联 JSON Schema。
pub fn embed_output_schemas(tools: &[ToolDefinition], schema_root: &Path) -> Vec<Value> {
    tools
        .iter()
        .map(|tool| {
            let mut v = serde_json::to_value(tool).expect("tool serializes");
            if let Some(ref path) = tool.output_schema {
                let rel = path.strip_prefix("schemas/").unwrap_or(path);
                let full = schema_root.join(rel);
                if full.exists() {
                    if let Ok(raw) = std::fs::read_to_string(&full) {
                        if let Ok(schema) = serde_json::from_str::<Value>(&raw) {
                            v.as_object_mut()
                                .expect("tool object")
                                .insert("outputSchema".into(), schema);
                        }
                    }
                }
            }
            v
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_tools_exclude_internal_params() {
        let tools = generate_agent_tools();
        let analyze = tools
            .iter()
            .find(|t| t.name == "fanalyzer_analyze")
            .expect("analyze");
        assert!(!analyze.input_schema.properties.contains_key("compact"));
        assert!(!analyze
            .input_schema
            .properties
            .contains_key("compact-series"));
        assert!(analyze.input_schema.properties.contains_key("code"));
    }

    #[test]
    fn composite_tools_present() {
        let tools = generate_agent_tools();
        assert!(tools.iter().any(|t| t.name == "fanalyzer_research_fund"));
        assert!(tools.iter().any(|t| t.name == "fanalyzer_watchlist_list"));
    }
}
