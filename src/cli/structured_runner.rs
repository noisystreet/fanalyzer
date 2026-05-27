//! 结构化命令执行（捕获 JSON 信封，供 MCP 等调用）。

use crate::api::eastmoney::EastMoneyClient;
use crate::application::{CommandContext, OutputProfile, StructuredOutput};
use crate::cache::FundCache;
use crate::nav_cache::NavCache;
use crate::presentation::{error_from_anyhow, print_failure_capture, StructuredError};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::dispatch;
use super::Commands;

/// 执行结构化命令并返回 JSON 信封字符串（成功或失败均有 JSON）。
pub async fn run_structured_command(
    cmd: Commands,
    profile: OutputProfile,
    offline: bool,
    watchlist_path: &Path,
    client: &EastMoneyClient,
    name_cache: &Arc<Mutex<FundCache>>,
    nav_store: &NavCache,
) -> String {
    let structured = StructuredOutput::for_capture(profile);
    let cmd_name = cmd.name();
    match dispatch::dispatch_with_command(
        cmd,
        client,
        name_cache,
        nav_store,
        offline,
        watchlist_path,
        structured,
    )
    .await
    {
        Ok(captured) => captured.unwrap_or_else(|| missing_capture_json(cmd_name)),
        Err(e) => {
            let ctx = CommandContext::new(
                client,
                name_cache,
                nav_store,
                offline,
                watchlist_path,
                structured,
            );
            let structured_err = error_from_anyhow(&e);
            print_failure_capture(&ctx, cmd_name, &structured_err)
                .unwrap_or_else(|_| fallback_failure_json(cmd_name, &structured_err))
        }
    }
}

fn missing_capture_json(command: &str) -> String {
    fallback_failure_json(
        command,
        &StructuredError {
            code: "NO_OUTPUT".into(),
            message: "命令成功但未产生 JSON 输出".into(),
            retryable: Some(false),
            hint: None,
        },
    )
}

fn fallback_failure_json(command: &str, error: &StructuredError) -> String {
    serde_json::json!({
        "v": 1,
        "command": command,
        "ok": false,
        "error": error,
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fallback_failure_is_valid_json() {
        let json = fallback_failure_json(
            "analyze",
            &StructuredError {
                code: "TEST".into(),
                message: "msg".into(),
                retryable: None,
                hint: None,
            },
        );
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["ok"], false);
    }
}
