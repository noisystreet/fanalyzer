//! MCP `tools/call` 结果与 response schema 的契约测试（离线，无需联网）。

mod test_fixtures;

use serde_json::json;
use test_fixtures::{
    mcp_rpc_result, mcp_tool_envelope, offline_mcp_serve_args, offline_mcp_serve_args_with_tier,
    setup_offline_two_fund_env, validate_failure_envelope, validate_success_command,
};

fn mcp_offline_envelope(
    env: &test_fixtures::OfflineContractEnv,
    tool_name: &str,
    arguments: serde_json::Value,
) -> serde_json::Value {
    mcp_tool_envelope(offline_mcp_serve_args(env), tool_name, arguments)
}

#[test]
fn mcp_tools_list_includes_output_schema() {
    let temp = tempfile::tempdir().unwrap();
    let env = setup_offline_two_fund_env(temp.path());
    let rpc = mcp_rpc_result(offline_mcp_serve_args(&env), "tools/list", json!({}));
    let tools = rpc["result"]["tools"].as_array().expect("tools array");
    assert!(!tools.is_empty());
    let analyze = tools
        .iter()
        .find(|t| t["name"] == "fanalyzer_analyze")
        .expect("analyze tool");
    assert!(analyze.get("outputSchema").is_some());
}

#[test]
fn mcp_tools_minimal_tier_returns_six_tools() {
    let temp = tempfile::tempdir().unwrap();
    let env = setup_offline_two_fund_env(temp.path());
    let rpc = mcp_rpc_result(
        offline_mcp_serve_args_with_tier(&env, Some("minimal")),
        "tools/list",
        json!({}),
    );
    let tools = rpc["result"]["tools"].as_array().expect("tools array");
    assert_eq!(tools.len(), 6);
}

#[test]
fn mcp_resources_list_includes_schema_index() {
    let temp = tempfile::tempdir().unwrap();
    let env = setup_offline_two_fund_env(temp.path());
    let rpc = mcp_rpc_result(offline_mcp_serve_args(&env), "resources/list", json!({}));
    let resources = rpc["result"]["resources"]
        .as_array()
        .expect("resources array");
    assert!(resources
        .iter()
        .any(|r| r["uri"] == "fanalyzer://schemas/index"));
}

#[test]
fn mcp_resources_read_watchlist() {
    let temp = tempfile::tempdir().unwrap();
    let env = setup_offline_two_fund_env(temp.path());
    let rpc = mcp_rpc_result(
        offline_mcp_serve_args(&env),
        "resources/read",
        json!({"uri": "fanalyzer://watchlist"}),
    );
    let text = rpc["result"]["contents"][0]["text"]
        .as_str()
        .expect("watchlist text");
    assert!(text.contains("000001"));
}

#[test]
fn mcp_analyze_matches_analyze_success_schema() {
    let temp = tempfile::tempdir().unwrap();
    let env = setup_offline_two_fund_env(temp.path());
    let envelope = mcp_offline_envelope(
        &env,
        "fanalyzer_analyze",
        json!({"code": "000001", "days": 30}),
    );
    validate_success_command(&envelope, "analyze");
}

#[test]
fn mcp_compare_matches_compare_success_schema() {
    let temp = tempfile::tempdir().unwrap();
    let env = setup_offline_two_fund_env(temp.path());
    let envelope = mcp_offline_envelope(
        &env,
        "fanalyzer_compare",
        json!({"codes": ["000001", "110011"], "days": 30}),
    );
    validate_success_command(&envelope, "compare");
}

#[test]
fn mcp_compare_watchlist_matches_compare_success_schema() {
    let temp = tempfile::tempdir().unwrap();
    let env = setup_offline_two_fund_env(temp.path());
    let envelope = mcp_offline_envelope(&env, "fanalyzer_compare_watchlist", json!({"days": 30}));
    validate_success_command(&envelope, "compare");
}

#[test]
fn mcp_portfolio_matches_portfolio_success_schema() {
    let temp = tempfile::tempdir().unwrap();
    let env = setup_offline_two_fund_env(temp.path());
    let envelope = mcp_offline_envelope(
        &env,
        "fanalyzer_portfolio",
        json!({
            "portfolio-file": env.portfolio_path.to_string_lossy(),
            "days": 30
        }),
    );
    validate_success_command(&envelope, "portfolio");
}

#[test]
fn mcp_export_matches_export_success_schema() {
    let temp = tempfile::tempdir().unwrap();
    let env = setup_offline_two_fund_env(temp.path());
    let envelope = mcp_offline_envelope(
        &env,
        "fanalyzer_export",
        json!({"code": "000001", "days": 30}),
    );
    validate_success_command(&envelope, "export");
}

#[test]
fn mcp_portfolio_config_matches_success_schema() {
    let temp = tempfile::tempdir().unwrap();
    let env = setup_offline_two_fund_env(temp.path());
    let envelope = mcp_offline_envelope(
        &env,
        "fanalyzer_portfolio_config",
        json!({"portfolio-file": env.portfolio_path.to_string_lossy()}),
    );
    validate_success_command(&envelope, "portfolio_config");
}

#[test]
fn mcp_watchlist_list_matches_watchlist_success_schema() {
    let temp = tempfile::tempdir().unwrap();
    let env = setup_offline_two_fund_env(temp.path());
    let envelope = mcp_offline_envelope(&env, "fanalyzer_watchlist_list", json!({}));
    validate_success_command(&envelope, "watchlist");
}

#[test]
fn mcp_analyze_offline_flag_matches_analyze_success_schema() {
    let temp = tempfile::tempdir().unwrap();
    let env = setup_offline_two_fund_env(temp.path());
    let envelope = mcp_tool_envelope(
        offline_mcp_serve_args(&env),
        "fanalyzer_analyze",
        json!({"code": "000001", "days": 30, "offline": true}),
    );
    validate_success_command(&envelope, "analyze");
    assert_eq!(envelope["meta"]["offline"], true);
}

#[test]
fn mcp_watchlist_add_matches_watchlist_success_schema() {
    let temp = tempfile::tempdir().unwrap();
    let env = setup_offline_two_fund_env(temp.path());
    let envelope = mcp_offline_envelope(
        &env,
        "fanalyzer_watchlist_add",
        json!({"codes": ["159915"]}),
    );
    validate_success_command(&envelope, "watchlist");
}

#[test]
fn mcp_watchlist_remove_matches_watchlist_success_schema() {
    let temp = tempfile::tempdir().unwrap();
    let env = setup_offline_two_fund_env(temp.path());
    let envelope = mcp_offline_envelope(
        &env,
        "fanalyzer_watchlist_remove",
        json!({"codes": ["110011"]}),
    );
    validate_success_command(&envelope, "watchlist");
}

#[test]
fn mcp_research_fund_matches_research_fund_success_schema() {
    let temp = tempfile::tempdir().unwrap();
    let env = setup_offline_two_fund_env(temp.path());
    let envelope = mcp_offline_envelope(
        &env,
        "fanalyzer_research_fund",
        json!({"code": "000001", "days": 30}),
    );
    validate_success_command(&envelope, "research_fund");
    assert_eq!(envelope["data"]["info"]["command"], "info");
    assert_eq!(envelope["data"]["analyze"]["command"], "analyze");
}

#[test]
fn mcp_research_fund_missing_code_matches_failure_schema() {
    let temp = tempfile::tempdir().unwrap();
    let env = setup_offline_two_fund_env(temp.path());
    let envelope = mcp_offline_envelope(&env, "fanalyzer_research_fund", json!({}));
    validate_failure_envelope(&envelope);
}
