use assert_cmd::Command;
use predicates::prelude::*;

mod test_fixtures;

use test_fixtures::{linear_nav_series, write_config_with_cache_root, write_offline_cache};

#[test]
fn test_cli_help_lists_json_subcommand() {
    Command::cargo_bin("fanalyzer")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("json"))
        .stdout(predicate::str::contains("structured"));
}

#[test]
fn test_cli_json_subcommand_help() {
    Command::cargo_bin("fanalyzer")
        .unwrap()
        .args(["json", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--compact"))
        .stdout(predicate::str::contains("compact-series"))
        .stdout(predicate::str::contains("analyze"));
}

#[test]
fn test_cli_no_global_json_flag() {
    Command::cargo_bin("fanalyzer")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--json").not());
}

#[test]
fn test_cli_json_analyze_positional_code() {
    Command::cargo_bin("fanalyzer")
        .unwrap()
        .args(["json", "analyze", "110011", "--days", "90"])
        .assert()
        .stdout(predicate::str::contains("\"command\": \"analyze\""));
}

#[test]
fn test_cli_json_failure_envelope() {
    Command::cargo_bin("fanalyzer")
        .unwrap()
        .args(["json", "compare", "--codes", "110011"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("\"ok\": false"))
        .stdout(predicate::str::contains("\"command\": \"compare\""))
        .stdout(predicate::str::contains("INSUFFICIENT_SAMPLES"));
}

#[test]
fn test_cli_version() {
    Command::cargo_bin("fanalyzer")
        .unwrap()
        .arg("--version")
        .assert()
        .success();
}

#[test]
fn test_cli_rank_help() {
    Command::cargo_bin("fanalyzer")
        .unwrap()
        .args(["rank", "--help"])
        .assert()
        .success();
}

#[test]
fn test_cli_sectors_help() {
    Command::cargo_bin("fanalyzer")
        .unwrap()
        .args(["sectors", "--help"])
        .assert()
        .success();
}

#[test]
fn test_cli_holdings_help() {
    Command::cargo_bin("fanalyzer")
        .unwrap()
        .args(["holdings", "--help"])
        .assert()
        .success();
}

#[test]
fn test_cli_brief_help() {
    Command::cargo_bin("fanalyzer")
        .unwrap()
        .args(["brief", "--help"])
        .assert()
        .success();
}

#[test]
fn test_cli_screen_help() {
    Command::cargo_bin("fanalyzer")
        .unwrap()
        .args(["screen", "--help"])
        .assert()
        .success();
}

#[test]
fn test_cli_portfolio_help() {
    Command::cargo_bin("fanalyzer")
        .unwrap()
        .args(["portfolio", "--help"])
        .assert()
        .success();
}

#[test]
fn test_cli_schema_export_writes_index() {
    let temp = tempfile::tempdir().unwrap();
    Command::cargo_bin("fanalyzer")
        .unwrap()
        .args([
            "schema",
            "export",
            "--output-dir",
            temp.path().to_str().unwrap(),
        ])
        .assert()
        .success();
    let index = temp.path().join("index.json");
    assert!(index.exists(), "schema export should write index.json");
    let tools = temp.path().join("tools.v1.json");
    assert!(tools.exists(), "schema export should write tools.v1.json");
}

#[test]
fn test_cli_schema_tools_excludes_schema_command() {
    let raw = Command::cargo_bin("fanalyzer")
        .unwrap()
        .args(["schema", "tools"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(raw).unwrap();
    assert!(!text.contains("fanalyzer_schema"));
}

#[test]
fn test_cli_mcp_serve_help() {
    Command::cargo_bin("fanalyzer")
        .unwrap()
        .args(["mcp", "serve", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--profile"));
}

#[test]
fn test_mcp_tools_call_watchlist_list_returns_json_envelope() {
    let req = r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"fanalyzer_watchlist_list","arguments":{}}}"#;
    let raw = Command::cargo_bin("fanalyzer")
        .unwrap()
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args(["mcp", "serve", "--profile", "summary"])
        .write_stdin(format!("{req}\n"))
        .timeout(std::time::Duration::from_secs(10))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let line = String::from_utf8(raw).unwrap();
    let line = line.lines().next().expect("one json line");
    let rpc: serde_json::Value = serde_json::from_str(line).unwrap();
    let text = rpc["result"]["content"][0]["text"]
        .as_str()
        .expect("tool result text");
    let envelope: serde_json::Value = serde_json::from_str(text).unwrap();
    assert_eq!(envelope["ok"], true);
    assert_eq!(envelope["command"], "watchlist");
    assert!(envelope["data"]["items"].is_array());
}

#[test]
fn test_mcp_tools_call_analyze_offline_returns_json_envelope() {
    let temp = tempfile::tempdir().unwrap();
    let cache_root = temp.path().join("cache");
    let code = "000001";
    write_offline_cache(
        &cache_root,
        code,
        "集成测试基金",
        &linear_nav_series(code, 91),
    );
    let config_path = write_config_with_cache_root(temp.path(), &cache_root);

    let req = format!(
        r#"{{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{{"name":"fanalyzer_analyze","arguments":{{"code":"{code}","days":30}}}}}}"#
    );
    let raw = Command::cargo_bin("fanalyzer")
        .unwrap()
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args([
            "--config",
            config_path.to_str().unwrap(),
            "mcp",
            "serve",
            "--offline",
            "--profile",
            "summary",
        ])
        .write_stdin(format!("{req}\n"))
        .timeout(std::time::Duration::from_secs(10))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let line = String::from_utf8(raw).unwrap();
    let line = line.lines().next().expect("one json line");
    let rpc: serde_json::Value = serde_json::from_str(line).unwrap();
    let text = rpc["result"]["content"][0]["text"]
        .as_str()
        .expect("tool result text");
    let envelope: serde_json::Value = serde_json::from_str(text).unwrap();
    assert_eq!(envelope["ok"], true);
    assert_eq!(envelope["command"], "analyze");
    assert!(envelope["meta"]["duration_ms"].as_u64().is_some());
    assert_eq!(envelope["data"]["items"][0]["snapshot"]["code"], code);
}

#[test]
fn test_cli_config_flag_loads_custom_cache_root() {
    let temp = tempfile::tempdir().unwrap();
    let cache_root = temp.path().join("cache");
    let code = "000001";
    write_offline_cache(
        &cache_root,
        code,
        "配置测试基金",
        &linear_nav_series(code, 91),
    );
    let config_path = write_config_with_cache_root(temp.path(), &cache_root);

    Command::cargo_bin("fanalyzer")
        .unwrap()
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args([
            "--config",
            config_path.to_str().unwrap(),
            "--offline",
            "json",
            "analyze",
            code,
            "--days",
            "30",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"ok\": true"))
        .stdout(predicate::str::contains("\"duration_ms\""));
}

#[test]
fn test_cli_json_profile_flag() {
    Command::cargo_bin("fanalyzer")
        .unwrap()
        .args(["json", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--profile"));
}
