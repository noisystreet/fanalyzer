//! 单基金分析用例。

use super::context::{resolve_fund_ids, CommandContext};
use super::fund_service;
use crate::domain::resolve_analysis_days;
use crate::presentation::print_analysis;

pub struct AnalyzeRequest {
    pub code: Option<String>,
    pub pick_watchlist: bool,
    pub days: u32,
    pub period: Option<String>,
}

pub async fn run_analyze(ctx: &CommandContext<'_>, req: AnalyzeRequest) -> anyhow::Result<()> {
    let days = resolve_analysis_days(req.period.as_deref(), req.days)?;
    let ids = resolve_fund_ids(
        req.code,
        req.pick_watchlist,
        ctx.watchlist_path,
        "--code/--watchlist",
    )?;
    for id in ids {
        tracing::info!(code = %id, days = days, "Analyzing fund");
        match fund_service::analyze_fund(&ctx.session, &id, days, ctx.offline).await {
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
