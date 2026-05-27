//! CLI 输出与 `schemas/responses/*.success.json` 的契约测试（无需联网）。

mod test_fixtures;

use assert_cmd::Command;
use jsonschema::validator_for;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use test_fixtures::{linear_nav_series, write_config_with_cache_root, write_offline_cache};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn validate_envelope(instance: &Value, schema_rel: &str) {
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

fn parse_stdout_json(raw: &[u8]) -> Value {
    let text = String::from_utf8(raw.to_vec()).expect("utf8 stdout");
    serde_json::from_str(text.trim()).expect("stdout json envelope")
}

fn offline_env(temp: &Path) -> (PathBuf, PathBuf) {
    let cache_root = temp.join("cache");
    let code = "000001";
    write_offline_cache(
        &cache_root,
        code,
        "契约测试基金",
        &linear_nav_series(code, 91),
    );
    let config_path = write_config_with_cache_root(temp, &cache_root);
    (config_path, cache_root)
}

#[test]
fn analyze_cli_output_matches_success_schema() {
    let temp = tempfile::tempdir().unwrap();
    let (config_path, _) = offline_env(temp.path());
    let raw = Command::cargo_bin("fanalyzer")
        .unwrap()
        .current_dir(repo_root())
        .args([
            "--config",
            config_path.to_str().unwrap(),
            "--offline",
            "json",
            "analyze",
            "000001",
            "--days",
            "30",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    validate_envelope(
        &parse_stdout_json(&raw),
        "schemas/responses/analyze.success.json",
    );
}

#[test]
fn compare_cli_output_matches_success_schema() {
    let temp = tempfile::tempdir().unwrap();
    let cache_root = temp.path().join("cache");
    write_offline_cache(
        &cache_root,
        "000001",
        "基金A",
        &linear_nav_series("000001", 91),
    );
    write_offline_cache(
        &cache_root,
        "110011",
        "基金B",
        &linear_nav_series("110011", 91),
    );
    let config_path = write_config_with_cache_root(temp.path(), &cache_root);
    let raw = Command::cargo_bin("fanalyzer")
        .unwrap()
        .current_dir(repo_root())
        .args([
            "--config",
            config_path.to_str().unwrap(),
            "--offline",
            "json",
            "compare",
            "--codes",
            "000001,110011",
            "--days",
            "30",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    validate_envelope(
        &parse_stdout_json(&raw),
        "schemas/responses/compare.success.json",
    );
}

#[test]
fn portfolio_cli_output_matches_success_schema() {
    let temp = tempfile::tempdir().unwrap();
    let cache_root = temp.path().join("cache");
    write_offline_cache(
        &cache_root,
        "000001",
        "基金A",
        &linear_nav_series("000001", 91),
    );
    write_offline_cache(
        &cache_root,
        "110011",
        "基金B",
        &linear_nav_series("110011", 91),
    );
    let config_path = write_config_with_cache_root(temp.path(), &cache_root);
    let portfolio_path = temp.path().join("portfolio.toml");
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
    let raw = Command::cargo_bin("fanalyzer")
        .unwrap()
        .current_dir(repo_root())
        .args([
            "--config",
            config_path.to_str().unwrap(),
            "--offline",
            "json",
            "portfolio",
            "--portfolio-file",
            portfolio_path.to_str().unwrap(),
            "--days",
            "30",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    validate_envelope(
        &parse_stdout_json(&raw),
        "schemas/responses/portfolio.success.json",
    );
}

#[test]
fn watchlist_cli_output_matches_success_schema() {
    let raw = Command::cargo_bin("fanalyzer")
        .unwrap()
        .current_dir(repo_root())
        .args(["json", "watchlist", "list"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    validate_envelope(
        &parse_stdout_json(&raw),
        "schemas/responses/watchlist.success.json",
    );
}
