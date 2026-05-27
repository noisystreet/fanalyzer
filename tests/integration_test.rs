use assert_cmd::Command;
use predicates::prelude::*;

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
        .success()
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

#[cfg(feature = "web")]
#[test]
fn test_cli_serve_help() {
    Command::cargo_bin("fanalyzer")
        .unwrap()
        .args(["serve", "--help"])
        .assert()
        .success();
}
