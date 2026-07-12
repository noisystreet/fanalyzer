//! CLI 输出与 `schemas/responses/*.success.json` 的契约测试（无需联网）。

mod test_fixtures;

use test_fixtures::{
    OfflineContractEnv, parse_stdout_json, run_json_cli, run_json_cli_expect_failure,
    setup_offline_two_fund_env, validate_envelope, validate_success_command,
};

fn run_offline_json(env: &OfflineContractEnv, tail: &[&str]) -> serde_json::Value {
    let mut args = test_fixtures::offline_json_prefix(env);
    args.extend(tail.iter().map(|s| (*s).to_string()));
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    parse_stdout_json(&run_json_cli(&refs).stdout)
}

#[test]
fn analyze_cli_output_matches_success_schema() {
    let temp = tempfile::tempdir().unwrap();
    let env = setup_offline_two_fund_env(temp.path());
    let envelope = run_offline_json(&env, &["analyze", "000001", "--days", "30"]);
    validate_success_command(&envelope, "analyze");
}

#[test]
fn compare_cli_output_matches_success_schema() {
    let temp = tempfile::tempdir().unwrap();
    let env = setup_offline_two_fund_env(temp.path());
    let envelope = run_offline_json(
        &env,
        &["compare", "--codes", "000001,110011", "--days", "30"],
    );
    validate_success_command(&envelope, "compare");
}

#[test]
fn portfolio_cli_output_matches_success_schema() {
    let temp = tempfile::tempdir().unwrap();
    let env = setup_offline_two_fund_env(temp.path());
    let envelope = run_offline_json(
        &env,
        &[
            "portfolio",
            "--portfolio-file",
            env.portfolio_path.to_str().unwrap(),
            "--days",
            "30",
        ],
    );
    validate_success_command(&envelope, "portfolio");
}

#[test]
fn export_cli_output_matches_success_schema() {
    let temp = tempfile::tempdir().unwrap();
    let env = setup_offline_two_fund_env(temp.path());
    let envelope = run_offline_json(&env, &["export", "000001", "--days", "30"]);
    validate_success_command(&envelope, "export");
}

#[test]
fn portfolio_config_cli_output_matches_success_schema() {
    let temp = tempfile::tempdir().unwrap();
    let env = setup_offline_two_fund_env(temp.path());
    let envelope = run_offline_json(
        &env,
        &[
            "portfolio-config",
            "--portfolio-file",
            env.portfolio_path.to_str().unwrap(),
        ],
    );
    validate_success_command(&envelope, "portfolio_config");
}

#[test]
fn watchlist_cli_output_matches_success_schema() {
    let temp = tempfile::tempdir().unwrap();
    let env = setup_offline_two_fund_env(temp.path());
    let mut args = test_fixtures::offline_json_prefix(&env);
    args.extend([
        "--watchlist-file".into(),
        env.watchlist_path.to_string_lossy().into_owned(),
        "watchlist".into(),
        "list".into(),
    ]);
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let envelope = parse_stdout_json(&run_json_cli(&refs).stdout);
    validate_success_command(&envelope, "watchlist");
}

#[test]
fn compare_failure_cli_output_matches_failure_schema() {
    let temp = tempfile::tempdir().unwrap();
    let env = setup_offline_two_fund_env(temp.path());
    let mut args = test_fixtures::offline_json_prefix(&env);
    args.extend(["compare".into(), "--codes".into(), "000001".into()]);
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let envelope = parse_stdout_json(&run_json_cli_expect_failure(&refs).stdout);
    validate_envelope(&envelope, "schemas/envelope.failure.json");
    assert_eq!(envelope["ok"], false);
}
