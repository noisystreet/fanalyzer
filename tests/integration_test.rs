use assert_cmd::Command;

#[test]
fn test_cli_help() {
    Command::cargo_bin("fanalyzer")
        .unwrap()
        .arg("--help")
        .assert()
        .success();
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

#[cfg(feature = "web")]
#[test]
fn test_cli_serve_help() {
    Command::cargo_bin("fanalyzer")
        .unwrap()
        .args(["serve", "--help"])
        .assert()
        .success();
}
