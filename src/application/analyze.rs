//! 单基金分析用例。

use super::context::{resolve_fund_ids, CommandContext};
use super::fund_service;
use crate::domain::resolve_analysis_days;
use crate::presentation::{print_analysis, render_analysis};
use chrono::Local;
use std::path::PathBuf;

pub struct AnalyzeRequest {
    pub code: Option<String>,
    pub pick_watchlist: bool,
    pub days: u32,
    pub period: Option<String>,
    pub rolling_window: u32,
    pub output: Option<PathBuf>,
    pub format: String,
}

pub async fn run_analyze(ctx: &CommandContext<'_>, req: AnalyzeRequest) -> anyhow::Result<()> {
    let today = Local::now().date_naive();
    let days = resolve_analysis_days(req.period.as_deref(), req.days, today)?;
    let ids = resolve_fund_ids(
        req.code,
        req.pick_watchlist,
        ctx.watchlist_path,
        "--code/--watchlist",
    )?;
    for id in ids {
        tracing::info!(code = %id, days = days, "Analyzing fund");
        match fund_service::analyze_fund(&ctx.session, &id, days, ctx.offline, req.rolling_window)
            .await
        {
            Ok(Some(report)) => {
                print_analysis(&report.snapshot);
                render_analysis(&report, req.output.as_deref(), &req.format)?;
            }
            Ok(None) => tracing::warn!("Insufficient data for analysis"),
            Err(e) => {
                tracing::error!(error = %e, "Failed to analyze");
                return Err(e);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::domain::DEFAULT_ROLLING_WINDOW;

    #[test]
    fn default_rolling_window_matches_domain() {
        assert_eq!(DEFAULT_ROLLING_WINDOW, 60);
    }
}
