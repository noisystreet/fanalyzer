//! `analyze` 子命令参数。

use super::fund_session::analyze_fund;
use super::output::print_analysis;
use super::Cli;
use crate::analysis_period::resolve_analysis_days;
use crate::api::eastmoney::EastMoneyClient;
use crate::cache::FundCache;
use crate::nav_cache::NavCache;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct AnalyzeOpts {
    pub code: Option<String>,
    pub pick_watchlist: bool,
    pub days: u32,
    pub period: Option<String>,
}

pub async fn run_analyze_cmd(
    cli: &Cli,
    client: &EastMoneyClient,
    cache: &Arc<Mutex<FundCache>>,
    nav_store: &NavCache,
    opts: AnalyzeOpts,
) -> anyhow::Result<()> {
    let days = resolve_analysis_days(opts.period.as_deref(), opts.days)?;
    let ids = crate::cli::handlers::identifiers_one_or_watchlist(
        opts.code,
        opts.pick_watchlist,
        &cli.watchlist_file,
        "--code/--watchlist",
    )?;
    for id in ids {
        tracing::info!(code = %id, days = days, "Analyzing fund");
        match analyze_fund(client, cache, nav_store, &id, days, cli.offline).await {
            Ok(Some(analysis)) => print_analysis(&analysis),
            Ok(None) => tracing::warn!("Insufficient data for analysis"),
            Err(e) => {
                tracing::error!(error = %e, "Failed to analyze");
                return Err(e);
            }
        }
    }
    Ok(())
}
