//! MCP `tools/call` 结果与 response schema 的契约测试（离线，无需联网）。

mod test_fixtures;

use serde_json::json;
use test_fixtures::{
    mcp_tool_envelope, offline_mcp_serve_args, setup_offline_two_fund_env, validate_success_command,
};

fn mcp_offline_envelope(
    env: &test_fixtures::OfflineContractEnv,
    tool_name: &str,
    arguments: serde_json::Value,
) -> serde_json::Value {
    mcp_tool_envelope(offline_mcp_serve_args(env), tool_name, arguments)
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
