//! MCP 工具执行：映射 tool name → CLI 命令 / 复合流程。

use crate::api::eastmoney::EastMoneyClient;
use crate::application::{gather_research_fund, FundDataSource, OutputProfile, Session};
use crate::cache::FundCache;
use crate::cli::fund_code_arg::FundCodeArg;
use crate::cli::structured_runner::run_structured_command;
use crate::cli::Commands;
use crate::domain::DEFAULT_ROLLING_WINDOW;
use crate::nav_cache::NavCache;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct McpEnv<'a> {
    pub profile: OutputProfile,
    pub offline: bool,
    pub watchlist_path: &'a Path,
    pub client: &'a EastMoneyClient,
    pub name_cache: &'a Arc<Mutex<FundCache>>,
    pub nav_store: &'a NavCache,
}

pub async fn execute_tool(env: &McpEnv<'_>, name: &str, args: Value) -> (String, bool) {
    match name {
        "fanalyzer_research_fund" => research_fund(env, args).await,
        "fanalyzer_compare_watchlist" => {
            let cmd = Commands::Compare {
                codes: vec![],
                pick_watchlist: true,
                days: arg_u32(&args, "days", 90),
                period: arg_str(&args, "period"),
                sort: None,
                output: None,
                format: "json".into(),
            };
            run_and_classify(env, cmd).await
        }
        "fanalyzer_watchlist_list" => run_and_classify(env, Commands::WatchlistList).await,
        "fanalyzer_watchlist_add" => {
            let codes = arg_string_array(&args, "codes");
            run_and_classify(env, Commands::WatchlistAdd { codes }).await
        }
        "fanalyzer_watchlist_remove" => {
            let codes = arg_string_array(&args, "codes");
            run_and_classify(env, Commands::WatchlistRemove { codes }).await
        }
        "fanalyzer_portfolio_config" => {
            let portfolio_file = arg_path(&args, "portfolio-file", "config/portfolio.toml");
            run_and_classify(env, Commands::PortfolioConfig { portfolio_file }).await
        }
        other if other.starts_with("fanalyzer_") => {
            let sub = other.strip_prefix("fanalyzer_").unwrap_or(other);
            match build_command(sub, args) {
                Ok(cmd) => run_and_classify(env, cmd).await,
                Err(e) => (error_envelope(sub, &e.to_string()), true),
            }
        }
        _ => (error_envelope("mcp", &format!("未知工具：{name}")), true),
    }
}

async fn run_and_classify(env: &McpEnv<'_>, cmd: Commands) -> (String, bool) {
    let json = run_structured_command(
        cmd,
        env.profile,
        env.offline,
        env.watchlist_path,
        env.client,
        env.name_cache,
        env.nav_store,
    )
    .await;
    let is_error = serde_json::from_str::<Value>(&json)
        .ok()
        .and_then(|v| v.get("ok").and_then(|ok| ok.as_bool()))
        == Some(false);
    (json, is_error)
}

async fn research_fund(env: &McpEnv<'_>, args: Value) -> (String, bool) {
    let code = match args.get("code").and_then(|v| v.as_str()) {
        Some(c) => c.to_string(),
        None => return (error_envelope("research_fund", "缺少 code"), true),
    };
    let days = arg_u32(&args, "days", 90);
    let holdings_top = arg_u32(&args, "top", 10);
    let session = mcp_session(env);
    match gather_research_fund(
        &session,
        &code,
        days,
        env.offline,
        DEFAULT_ROLLING_WINDOW,
        holdings_top,
    )
    .await
    {
        Ok(result) => {
            let is_error = !result.ok;
            let text = result.to_envelope_json().unwrap_or_default();
            (text, is_error)
        }
        Err(e) => (error_envelope("research_fund", &e.to_string()), true),
    }
}

fn mcp_session<'a>(env: &'a McpEnv<'_>) -> Session<'a> {
    Session {
        source: env.client as &dyn FundDataSource,
        name_cache: env.name_cache,
        nav_store: env.nav_store,
    }
}

fn build_command(sub: &str, args: Value) -> anyhow::Result<Commands> {
    let fund_code = FundCodeArg {
        positional: args
            .get("code")
            .and_then(|v| v.as_str())
            .map(str::to_string),
        flag: None,
    };
    let pick_watchlist = args
        .get("watchlist")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    Ok(match sub {
        "fetch" => Commands::Fetch {
            fund_code,
            pick_watchlist,
            limit: arg_u32(&args, "limit", 20),
        },
        "analyze" => Commands::Analyze {
            fund_code,
            pick_watchlist,
            days: arg_u32(&args, "days", 30),
            period: arg_str(&args, "period"),
            output: None,
            format: "json".into(),
            rolling_window: arg_u32(&args, "rolling-window", 60),
        },
        "compare" => Commands::Compare {
            codes: arg_string_array(&args, "codes"),
            pick_watchlist,
            days: arg_u32(&args, "days", 30),
            period: arg_str(&args, "period"),
            sort: arg_str(&args, "sort"),
            output: None,
            format: "json".into(),
        },
        "portfolio" => Commands::Portfolio {
            portfolio_file: arg_path(&args, "portfolio-file", "config/portfolio.toml"),
            days: arg_u32(&args, "days", 90),
            period: arg_str(&args, "period"),
            holdings_top: arg_u32(&args, "holdings-top", 10),
            output: None,
            format: "json".into(),
            rolling_window: arg_u32(&args, "rolling-window", 60),
        },
        "export" => Commands::Export {
            fund_code,
            pick_watchlist,
            days: arg_u32(&args, "days", 30),
            output: None,
            output_dir: None,
            format: "json".into(),
        },
        "info" => Commands::Info {
            fund_code,
            pick_watchlist,
        },
        "sectors" => Commands::Sectors {
            fund_code,
            pick_watchlist,
        },
        "holdings" => Commands::Holdings {
            fund_code,
            pick_watchlist,
            top: arg_u32(&args, "top", 10),
        },
        "rank" => Commands::Rank {
            kind: arg_string(&args, "kind", ""),
            top: arg_u32(&args, "top", 100),
            sort: arg_string(&args, "sort", "1n"),
        },
        "brief" => Commands::Brief {
            fund_code,
            pick_watchlist,
            days: arg_u32(&args, "days", 90),
            period: arg_str(&args, "period"),
            industry_top: arg_u32(&args, "industry-top", 5),
            holdings_top: arg_u32(&args, "holdings-top", 10),
            output: None,
        },
        "screen" => build_screen_command(args),
        other => anyhow::bail!("未知子命令：{other}"),
    })
}

fn build_screen_command(args: Value) -> Commands {
    Commands::Screen {
        kind: arg_string(&args, "kind", ""),
        sort: arg_string(&args, "sort", "1n"),
        rank_top: arg_u32(&args, "rank-top", 30),
        days: args.get("days").and_then(|v| v.as_u64()).map(|d| d as u32),
        period: arg_str(&args, "period"),
        min_rank_return: arg_f64(&args, "min-rank-return"),
        max_drawdown: arg_f64(&args, "max-drawdown"),
        min_sharpe: arg_f64(&args, "min-sharpe"),
        max_mgmt_fee: arg_f64(&args, "max-mgmt-fee"),
        min_alpha: arg_f64(&args, "min-alpha"),
        max_volatility: arg_f64(&args, "max-volatility"),
        min_total_return: arg_f64(&args, "min-total-return"),
        deep_limit: arg_u32(&args, "deep-limit", 15),
        full_scan: args
            .get("full-scan")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        sort_by: arg_str(&args, "sort-by"),
        limit: arg_u32(&args, "limit", 10),
        output: None,
        format: "json".into(),
    }
}

fn arg_str(args: &Value, key: &str) -> Option<String> {
    args.get(key).and_then(|v| v.as_str()).map(str::to_string)
}

fn arg_string(args: &Value, key: &str, default: &str) -> String {
    arg_str(args, key).unwrap_or_else(|| default.to_string())
}

fn arg_u32(args: &Value, key: &str, default: u32) -> u32 {
    args.get(key)
        .and_then(|v| v.as_u64())
        .map(|n| n as u32)
        .unwrap_or(default)
}

fn arg_f64(args: &Value, key: &str) -> Option<f64> {
    args.get(key).and_then(|v| v.as_f64())
}

fn arg_string_array(args: &Value, key: &str) -> Vec<String> {
    args.get(key)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

fn arg_path(args: &Value, key: &str, default: &str) -> PathBuf {
    arg_str(args, key)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(default))
}

fn error_envelope(command: &str, message: &str) -> String {
    json!({
        "v": 1,
        "command": command,
        "ok": false,
        "warnings": [],
        "error": {"code": "MCP_TOOL_ERROR", "message": message}
    })
    .to_string()
}
