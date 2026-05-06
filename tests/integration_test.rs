use assert_cmd::Command;

#[test]
fn test_cli_help() {
    Command::cargo_bin("analysis_fund")
        .unwrap()
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_cli_version() {
    Command::cargo_bin("analysis_fund")
        .unwrap()
        .arg("--version")
        .assert()
        .success();
}

#[test]
fn test_cli_rank_help() {
    Command::cargo_bin("analysis_fund")
        .unwrap()
        .args(["rank", "--help"])
        .assert()
        .success();
}
