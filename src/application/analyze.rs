//! 单基金分析用例。

use super::context::{resolve_fund_ids, CommandContext};
use super::fund_service;
use crate::domain::resolve_analysis_days;
use crate::presentation::{
    base_meta, compact_analysis_reports, emit, item_error_failed, item_error_insufficient,
    print_analysis, render_analysis, AnalysisMeta, BatchPayload,
};
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
    let requested = ids.len();
    let mut items = Vec::with_capacity(requested);
    let mut errors = Vec::new();
    for id in ids {
        tracing::info!(code = %id, days = days, "Analyzing fund");
        match fund_service::analyze_fund(&ctx.session, &id, days, ctx.offline, req.rolling_window)
            .await
        {
            Ok(Some(report)) => {
                if !ctx.structured() {
                    print_analysis(&report.snapshot);
                    render_analysis(&report, req.output.as_deref(), &req.format)?;
                }
                items.push(report);
            }
            Ok(None) => {
                tracing::warn!(code = %id, "Insufficient data for analysis");
                if ctx.structured() {
                    errors.push(item_error_insufficient(&id));
                }
            }
            Err(e) => {
                if ctx.structured() {
                    tracing::warn!(code = %id, error = %e, "Failed to analyze, skipping");
                    errors.push(item_error_failed(&id, e));
                } else {
                    tracing::error!(error = %e, "Failed to analyze");
                    return Err(e);
                }
            }
        }
    }
    if ctx.structured() {
        if items.is_empty() {
            anyhow::bail!("无有效分析结果（数据不足或全部失败）");
        }
        if !errors.is_empty() {
            ctx.warn(format!("{} 只标的未产生有效分析", errors.len()));
        }
        if ctx.compact_series() {
            compact_analysis_reports(&mut items);
        }
        let meta = AnalysisMeta {
            base: base_meta(ctx),
            days,
            period: req.period.clone(),
            rolling_window: Some(req.rolling_window),
            requested,
            succeeded: items.len(),
        };
        emit(
            ctx,
            "analyze",
            &BatchPayload { items, errors },
            Some(&meta),
            req.output.as_deref().filter(|_| req.format == "json"),
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::domain::DEFAULT_ROLLING_WINDOW;
    use crate::models::FundAnalysisReport;
    use crate::presentation::BatchPayload;

    #[test]
    fn default_rolling_window_matches_domain() {
        assert_eq!(DEFAULT_ROLLING_WINDOW, 60);
    }

    #[test]
    fn analyze_report_serializes_for_agent() {
        let report = FundAnalysisReport {
            snapshot: crate::models::FundAnalysis {
                code: "000001".into(),
                name: "测试".into(),
                period_days: 90,
                avg_nav: 1.0,
                max_nav: 1.1,
                min_nav: 0.9,
                total_return: 0.05,
                annualized_return: 0.08,
                volatility: 0.12,
                max_drawdown: 0.06,
                sharpe_ratio: 1.2,
                sortino_ratio: 1.3,
                calmar_ratio: 1.1,
                alpha: 0.01,
                beta: 0.95,
                manager_name: String::new(),
                manager_tenure_days: 0,
                manager_total_return: 0.0,
                management_fee: 0.0,
                custody_fee: 0.0,
            },
            series: None,
            benchmark_label: None,
        };
        let json = serde_json::to_string(&BatchPayload {
            items: vec![report],
            errors: vec![],
        })
        .unwrap();
        assert!(json.contains("000001"));
    }
}
