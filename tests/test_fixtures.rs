//! 集成测试共享 fixture（离线缓存、配置、Schema 契约校验）。
#![allow(dead_code)]

use assert_cmd::Command;
use fanalyzer::models::FundNav;
use fanalyzer::nav_cache::NavCache;
use jsonschema::validator_for;
use serde_json::{Map, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Output;

/// 生成从 `points` 天前到今天的线性增长净值序列。
pub fn linear_nav_series(code: &str, points: usize) -> Vec<FundNav> {
    let today = chrono::Local::now().date_naive();
    (0..points)
        .map(|i| {
            let date = today - chrono::Duration::days((points - 1 - i) as i64);
            let nav = 1.0 + i as f64 * 0.001;
            FundNav {
                code: code.to_string(),
                date,
                nav,
                acc_nav: nav,
                daily_return: None,
            }
        })
        .collect()
}

pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn success_schema_rel(command: &str) -> String {
    format!("schemas/responses/{command}.success.json")
}

pub fn validate_envelope(instance: &Value, schema_rel: &str) {
    let schema_path = repo_root().join(schema_rel);
    let schema_text = fs::read_to_string(&schema_path)
        .unwrap_or_else(|e| panic!("read schema {}: {e}", schema_path.display()));
    let schema: Value = serde_json::from_str(&schema_text).expect("schema json");
    let validator = validator_for(&schema).expect("compile schema");
    let errors: Vec<String> = validator
        .iter_errors(instance)
        .map(|e| format!("{e}"))
        .collect();
    assert!(
        errors.is_empty(),
        "schema {} validation failed:\n{}",
        schema_rel,
        errors.join("\n")
    );
}

pub fn validate_success_command(instance: &Value, command: &str) {
    validate_envelope(instance, &success_schema_rel(command));
}

pub fn validate_failure_envelope(instance: &Value) {
    validate_envelope(instance, "schemas/envelope.failure.json");
}

pub fn parse_stdout_json(raw: &[u8]) -> Value {
    let text = String::from_utf8(raw.to_vec()).expect("utf8 stdout");
    serde_json::from_str(text.trim()).expect("stdout json envelope")
}

/// 写入离线分析所需的净值缓存与名称映射（多基金时合并 `fund_names.json`）。
pub fn write_offline_cache(cache_root: &Path, code: &str, name: &str, navs: &[FundNav]) {
    let nav_store = NavCache::with_root(cache_root.to_path_buf());
    nav_store.save_merged(code, navs).unwrap();

    let names_path = cache_root.join("fund_names.json");
    if let Some(parent) = names_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let mut mapping: Map<String, Value> = if names_path.exists() {
        fs::read_to_string(&names_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    } else {
        Map::new()
    };
    mapping.insert(code.to_string(), Value::String(name.to_string()));
    fs::write(
        names_path,
        serde_json::to_string_pretty(&mapping).expect("names json"),
    )
    .unwrap();
}

/// 生成指向 `cache_root` 的临时配置文件路径。
pub fn write_config_with_cache_root(dir: &Path, cache_root: &Path) -> PathBuf {
    let cfg_path = dir.join("test-config.toml");
    let content = format!(
        r#"
[api]
base_url = "https://example.invalid"

[log]
level = "info"

[cache]
root = "{}"
"#,
        cache_root.display()
    );
    fs::write(&cfg_path, content).unwrap();
    cfg_path
}

/// 双基金离线契约环境（缓存 + 组合 + 自选）。
pub struct OfflineContractEnv {
    pub config_path: PathBuf,
    pub portfolio_path: PathBuf,
    pub watchlist_path: PathBuf,
}

pub fn setup_offline_two_fund_env(temp: &Path) -> OfflineContractEnv {
    let cache_root = temp.join("cache");
    for (code, name) in [("000001", "基金A"), ("110011", "基金B")] {
        write_offline_cache(&cache_root, code, name, &linear_nav_series(code, 91));
    }
    let config_path = write_config_with_cache_root(temp, &cache_root);
    let portfolio_path = temp.join("portfolio.toml");
    fs::write(
        &portfolio_path,
        r#"
name = "schema-test"

[[holdings]]
code = "000001"
weight = 0.5

[[holdings]]
code = "110011"
weight = 0.5
"#,
    )
    .unwrap();
    let watchlist_path = temp.join("watchlist.toml");
    fs::write(
        &watchlist_path,
        r#"
funds = ["000001", "110011"]
"#,
    )
    .unwrap();
    OfflineContractEnv {
        config_path,
        portfolio_path,
        watchlist_path,
    }
}

pub fn run_json_cli(args: &[&str]) -> Output {
    Command::cargo_bin("fanalyzer")
        .unwrap()
        .current_dir(repo_root())
        .args(args)
        .assert()
        .success()
        .get_output()
        .clone()
}

pub fn run_json_cli_expect_failure(args: &[&str]) -> Output {
    Command::cargo_bin("fanalyzer")
        .unwrap()
        .current_dir(repo_root())
        .args(args)
        .assert()
        .failure()
        .get_output()
        .clone()
}

/// MCP `tools/call` 返回的工具结果信封（`result.content[0].text` 解析为 JSON）。
pub fn mcp_tool_envelope(serve_args: Vec<String>, tool_name: &str, arguments: Value) -> Value {
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": tool_name,
            "arguments": arguments,
        }
    });
    let output = Command::cargo_bin("fanalyzer")
        .unwrap()
        .current_dir(repo_root())
        .args(&serve_args)
        .write_stdin(format!("{request}\n"))
        .timeout(std::time::Duration::from_secs(15))
        .assert()
        .success()
        .get_output()
        .clone();
    parse_mcp_tool_envelope(&output.stdout)
}

pub fn parse_mcp_tool_envelope(stdout: &[u8]) -> Value {
    let text = String::from_utf8(stdout.to_vec()).expect("utf8 mcp stdout");
    let line = text.lines().next().expect("one json-rpc line");
    let rpc: Value = serde_json::from_str(line).expect("json-rpc");
    let text = rpc["result"]["content"][0]["text"]
        .as_str()
        .expect("tool result text");
    let from_text: Value = serde_json::from_str(text).expect("tool envelope json");
    let structured = rpc["result"]
        .get("structuredContent")
        .cloned()
        .expect("structuredContent required when tools declare outputSchema");
    assert_eq!(
        structured, from_text,
        "structuredContent must match content[0].text JSON"
    );
    from_text
}

pub fn offline_mcp_serve_args(env: &OfflineContractEnv) -> Vec<String> {
    offline_mcp_serve_args_with_tier(env, None)
}

pub fn offline_mcp_serve_args_with_tier(
    env: &OfflineContractEnv,
    tools_tier: Option<&str>,
) -> Vec<String> {
    let mut args = vec![
        "--config".into(),
        env.config_path.to_string_lossy().into_owned(),
        "mcp".into(),
        "serve".into(),
        "--offline".into(),
        "--profile".into(),
        "summary".into(),
        "--watchlist-file".into(),
        env.watchlist_path.to_string_lossy().into_owned(),
        "--portfolio-file".into(),
        env.portfolio_path.to_string_lossy().into_owned(),
    ];
    if let Some(tier) = tools_tier {
        args.push("--tools".into());
        args.push(tier.into());
    }
    args
}

pub fn mcp_rpc_result(serve_args: Vec<String>, method: &str, params: Value) -> Value {
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
        "params": params,
    });
    let output = Command::cargo_bin("fanalyzer")
        .unwrap()
        .current_dir(repo_root())
        .args(&serve_args)
        .write_stdin(format!("{request}\n"))
        .timeout(std::time::Duration::from_secs(15))
        .assert()
        .success()
        .get_output()
        .clone();
    let text = String::from_utf8(output.stdout).expect("utf8 mcp stdout");
    let line = text.lines().next().expect("one json-rpc line");
    serde_json::from_str(line).expect("json-rpc")
}

pub fn offline_json_prefix(env: &OfflineContractEnv) -> Vec<String> {
    vec![
        "--config".into(),
        env.config_path.to_string_lossy().into_owned(),
        "--offline".into(),
        "json".into(),
    ]
}
