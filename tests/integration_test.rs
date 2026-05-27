use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_cli_help() {
    Command::cargo_bin("fanalyzer")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--json"));
}

#[test]
fn test_cli_json_alias_in_help() {
    Command::cargo_bin("fanalyzer")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("structured"));
}

#[test]
fn test_cli_json_compact_in_help() {
    Command::cargo_bin("fanalyzer")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("json-compact"))
        .stdout(predicate::str::contains("compact-series"));
}

#[test]
fn test_cli_json_failure_envelope() {
    Command::cargo_bin("fanalyzer")
        .unwrap()
        .args(["--json", "compare", "--codes", "110011"])
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

#[cfg(feature = "web")]
#[test]
fn test_cli_serve_help() {
    Command::cargo_bin("fanalyzer")
        .unwrap()
        .args(["serve", "--help"])
        .assert()
        .success();
}
