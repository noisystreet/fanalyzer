//! 单基金分析用例。

use super::concurrency::{FUND_CONCURRENCY, map_concurrent};
use super::context::{CommandContext, resolve_fund_ids};
use super::fund_service;
use crate::domain::resolve_analysis_days;
use crate::models::FundAnalysisReport;
use crate::presentation::{
    AnalysisMeta, BatchPayload, ItemError, base_meta, compact_analysis_reports, emit,
    item_error_failed, item_error_insufficient, print_analysis, render_analysis,
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

enum AnalyzeBatchOutcome {
    Ok(Box<FundAnalysisReport>),
    Err(ItemError),
}

async fn analyze_one_for_batch(
    ctx: &CommandContext<'_>,
    id: String,
    days: u32,
    rolling_window: u32,
) -> AnalyzeBatchOutcome {
    match fund_service::analyze_fund(&ctx.session, &id, days, ctx.offline, rolling_window).await {
        Ok(Some(report)) => AnalyzeBatchOutcome::Ok(Box::new(report)),
        Ok(None) => {
            tracing::warn!(code = %id, "Insufficient data for analysis");
            AnalyzeBatchOutcome::Err(item_error_insufficient(&id))
        }
        Err(e) => {
            tracing::warn!(code = %id, error = %e, "Failed to analyze, skipping");
            AnalyzeBatchOutcome::Err(item_error_failed(&id, e))
        }
    }
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

    if ctx.structured() {
        let outcomes = map_concurrent(&ids, FUND_CONCURRENCY, |id| {
            analyze_one_for_batch(ctx, id, days, req.rolling_window)
        })
        .await;
        let mut errors = Vec::new();
        let mut items: Vec<FundAnalysisReport> = outcomes
            .into_iter()
            .filter_map(|outcome| match outcome {
                AnalyzeBatchOutcome::Ok(report) => Some(*report),
                AnalyzeBatchOutcome::Err(err) => {
                    errors.push(err);
                    None
                }
            })
            .collect();
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
        return Ok(());
    }

    for id in ids {
        tracing::info!(code = %id, days = days, "Analyzing fund");
        match fund_service::analyze_fund(&ctx.session, &id, days, ctx.offline, req.rolling_window)
            .await
        {
            Ok(Some(report)) => {
                print_analysis(&report.snapshot);
                render_analysis(&report, req.output.as_deref(), &req.format)?;
            }
            Ok(None) => {
                tracing::warn!(code = %id, "Insufficient data for analysis");
            }
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
    use crate::application::data_source::mock::MockFundDataSource;
    use crate::application::output_profile::OutputProfile;
    use crate::application::test_support::{linear_nav_series, strip_volatile_envelope_fields};
    use crate::application::{
        AnalyzeRequest, CommandContext, FundDataSource, StructuredOutput, run_analyze,
    };
    use crate::cache::FundCache;
    use crate::domain::DEFAULT_ROLLING_WINDOW;
    use crate::models::FundAnalysisReport;
    use crate::nav_cache::NavCache;
    use crate::presentation::BatchPayload;
    use serde_json::Value;
    use std::path::Path;
    use std::sync::Arc;
    use tempfile::tempdir;
    use tokio::sync::Mutex;

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

    #[tokio::test]
    async fn analyze_golden_envelope_with_mock_session() {
        let code = "000001";
        let navs = linear_nav_series(code, 91);
        let mock = MockFundDataSource::with_navs(code, "测试基金", navs);
        let dir = tempdir().unwrap();
        let cache_root = dir.path().join("cache");
        let name_cache = Arc::new(Mutex::new(FundCache::with_root(cache_root.clone())));
        let nav_store = NavCache::with_root(cache_root);
        let ctx = CommandContext::new(
            &mock as &dyn FundDataSource,
            &name_cache,
            &nav_store,
            false,
            Path::new("config/watchlist.toml"),
            StructuredOutput::for_capture(OutputProfile::Standard),
        );
        run_analyze(
            &ctx,
            AnalyzeRequest {
                code: Some(code.into()),
                pick_watchlist: false,
                days: 90,
                period: None,
                rolling_window: DEFAULT_ROLLING_WINDOW,
                output: None,
                format: "json".into(),
            },
        )
        .await
        .unwrap();

        let raw = ctx.take_captured().expect("captured json");
        let v: Value = serde_json::from_str(&raw).unwrap();
        let stable = strip_volatile_envelope_fields(v);

        assert_eq!(stable["ok"], true);
        assert_eq!(stable["command"], "analyze");
        assert_eq!(stable["meta"]["offline"], false);
        assert_eq!(stable["meta"]["days"], 90);
        assert_eq!(stable["meta"]["requested"], 1);
        assert_eq!(stable["meta"]["succeeded"], 1);
        assert_eq!(stable["data"]["items"][0]["snapshot"]["code"], "000001");
        assert_eq!(stable["data"]["items"][0]["snapshot"]["name"], "测试基金");
        let total_return = stable["data"]["items"][0]["snapshot"]["total_return"]
            .as_f64()
            .unwrap();
        assert!((total_return - 0.09).abs() < 1e-6);
    }

    #[tokio::test]
    async fn analyze_offline_golden_envelope_from_cache() {
        let code = "000001";
        let navs = linear_nav_series(code, 91);
        let dir = tempdir().unwrap();
        let cache_root = dir.path().join("cache");
        let nav_store = NavCache::with_root(cache_root.clone());
        nav_store.save_merged(code, &navs).unwrap();
        let name_cache = Arc::new(Mutex::new(FundCache::with_root(cache_root.clone())));
        name_cache.lock().await.set_mapping(code, "离线测试基金");

        let client = crate::api::eastmoney::EastMoneyClient::default();
        let ctx = CommandContext::new(
            &client as &dyn FundDataSource,
            &name_cache,
            &nav_store,
            true,
            Path::new("config/watchlist.toml"),
            StructuredOutput::for_capture(OutputProfile::Standard),
        );
        run_analyze(
            &ctx,
            AnalyzeRequest {
                code: Some(code.into()),
                pick_watchlist: false,
                days: 90,
                period: None,
                rolling_window: DEFAULT_ROLLING_WINDOW,
                output: None,
                format: "json".into(),
            },
        )
        .await
        .unwrap();

        let raw = ctx.take_captured().expect("captured json");
        let v: Value = serde_json::from_str(&raw).unwrap();
        let stable = strip_volatile_envelope_fields(v);
        assert_eq!(stable["ok"], true);
        assert_eq!(stable["meta"]["offline"], true);
        assert_eq!(
            stable["data"]["items"][0]["snapshot"]["name"],
            "离线测试基金"
        );
    }
}
