//! 多基金对比用例。

use super::concurrency::{map_concurrent, FUND_CONCURRENCY};
use super::context::{resolve_many_fund_ids, CommandContext, Session};
use super::fund_service;
use crate::domain::{parse_sort_key, resolve_analysis_days, sort_analyses, AnalysisSortKey};
use crate::models::FundAnalysis;
use crate::presentation::{
    base_meta, emit, item_error_failed, item_error_insufficient, render_comparison, AnalysisMeta,
    BatchPayload, ItemError,
};
use chrono::Local;
use std::path::PathBuf;

pub struct CompareRequest {
    pub codes: Vec<String>,
    pub pick_watchlist: bool,
    pub days: u32,
    pub period: Option<String>,
    pub sort: Option<String>,
    pub output: Option<PathBuf>,
    pub format: String,
}

pub struct CompareGather {
    pub items: Vec<FundAnalysis>,
    pub errors: Vec<ItemError>,
}

enum AnalyzeOutcome {
    Ok(FundAnalysis),
    Err(ItemError),
}

async fn analyze_one(
    session: &Session<'_>,
    identifier: String,
    days: u32,
    offline: bool,
) -> AnalyzeOutcome {
    match fund_service::analyze_fund(
        session,
        &identifier,
        days,
        offline,
        crate::domain::DEFAULT_ROLLING_WINDOW,
    )
    .await
    {
        Ok(Some(r)) => AnalyzeOutcome::Ok(r.snapshot),
        Ok(None) => {
            tracing::warn!(identifier = %identifier, "分析数据不足，跳过");
            AnalyzeOutcome::Err(item_error_insufficient(&identifier))
        }
        Err(e) => {
            tracing::warn!(identifier = %identifier, error = %e, "跳过该标的");
            AnalyzeOutcome::Err(item_error_failed(&identifier, e))
        }
    }
}

/// 批量分析多只基金，跳过数据不足或失败的标的。
pub async fn gather_compare_analyses(
    session: &Session<'_>,
    identifiers: &[String],
    days: u32,
    offline: bool,
) -> CompareGather {
    let outcomes = map_concurrent(identifiers, FUND_CONCURRENCY, |identifier| {
        analyze_one(session, identifier, days, offline)
    })
    .await;

    let mut items = Vec::new();
    let mut errors = Vec::new();
    for outcome in outcomes {
        match outcome {
            AnalyzeOutcome::Ok(item) => items.push(item),
            AnalyzeOutcome::Err(err) => errors.push(err),
        }
    }
    CompareGather { items, errors }
}

/// 按指标对对比结果排序；未指定时按代码升序。
pub fn sort_compare_analyses(
    analyses: &mut [FundAnalysis],
    sort: Option<&str>,
) -> anyhow::Result<()> {
    if let Some(raw) = sort.filter(|s| !s.is_empty()) {
        let key = parse_sort_key(raw)?;
        sort_analyses(analyses, key, key.default_desc());
    } else {
        sort_analyses(analyses, AnalysisSortKey::Code, false);
    }
    Ok(())
}

pub async fn run_compare(ctx: &CommandContext<'_>, req: CompareRequest) -> anyhow::Result<()> {
    let ids = resolve_many_fund_ids(req.codes, req.pick_watchlist, ctx.watchlist_path)?;
    if ids.len() < 2 {
        anyhow::bail!("对比至少需要 2 只基金（当前 {} 条）", ids.len());
    }
    let today = Local::now().date_naive();
    let days = resolve_analysis_days(req.period.as_deref(), req.days, today)?;
    tracing::info!(codes = ?ids, days = days, "Comparing funds");

    let requested = ids.len();
    let gathered = gather_compare_analyses(&ctx.session, &ids, days, ctx.offline).await;

    if gathered.items.len() < 2 {
        if ctx.structured() {
            anyhow::bail!("有效样本不足（需要≥2）；请检查离线缓存或数据源");
        }
        tracing::warn!("有效样本不足（需要≥2）；请检查离线缓存或数据源");
        return Ok(());
    }

    if !gathered.errors.is_empty() {
        ctx.warn(format!(
            "{} 只标的分析失败或数据不足",
            gathered.errors.len()
        ));
    }

    let mut analyses = gathered.items;
    sort_compare_analyses(&mut analyses, req.sort.as_deref())?;

    if ctx.structured() {
        let meta = AnalysisMeta {
            base: base_meta(ctx),
            days,
            period: req.period.clone(),
            rolling_window: None,
            requested,
            succeeded: analyses.len(),
        };
        emit(
            ctx,
            "compare",
            &BatchPayload {
                items: analyses,
                errors: gathered.errors,
            },
            Some(&meta),
            req.output.as_deref().filter(|_| req.format == "json"),
        )?;
        return Ok(());
    }

    render_comparison(&analyses, req.output.as_deref(), &req.format)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::FundAnalysis;

    fn sample_analysis(code: &str, sharpe: f64) -> FundAnalysis {
        FundAnalysis {
            code: code.to_string(),
            name: code.to_string(),
            period_days: 90,
            avg_nav: 1.0,
            max_nav: 1.0,
            min_nav: 1.0,
            total_return: 0.0,
            annualized_return: 0.0,
            volatility: 0.0,
            max_drawdown: 0.0,
            sharpe_ratio: sharpe,
            sortino_ratio: 0.0,
            calmar_ratio: 0.0,
            alpha: 0.0,
            beta: 0.0,
            manager_name: String::new(),
            manager_tenure_days: 0,
            manager_total_return: 0.0,
            management_fee: 0.0,
            custody_fee: 0.0,
        }
    }

    #[test]
    fn sort_compare_analyses_by_sharpe_desc() {
        let mut analyses = vec![
            sample_analysis("000001", 1.0),
            sample_analysis("110011", 2.5),
        ];
        sort_compare_analyses(&mut analyses, Some("sharpe")).unwrap();
        assert_eq!(analyses[0].code, "110011");
    }

    #[test]
    fn sort_compare_analyses_default_by_code() {
        let mut analyses = vec![
            sample_analysis("110011", 1.0),
            sample_analysis("000001", 2.0),
        ];
        sort_compare_analyses(&mut analyses, None).unwrap();
        assert_eq!(analyses[0].code, "000001");
    }

    #[tokio::test]
    async fn compare_golden_envelope_offline_two_funds() {
        use crate::application::output_profile::OutputProfile;
        use crate::application::test_support::{
            seed_offline_two_funds, strip_volatile_envelope_fields,
        };
        use crate::application::{CommandContext, FundDataSource, StructuredOutput};
        use std::path::Path;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let cache_root = dir.path().join("cache");
        let (nav_store, name_cache) =
            seed_offline_two_funds(&cache_root, &[("000001", "基金A"), ("110011", "基金B")]).await;
        let client = crate::api::eastmoney::EastMoneyClient::default();
        let ctx = CommandContext::new(
            &client as &dyn FundDataSource,
            &name_cache,
            &nav_store,
            true,
            Path::new("config/watchlist.toml"),
            StructuredOutput::for_capture(OutputProfile::Standard),
        );
        run_compare(
            &ctx,
            CompareRequest {
                codes: vec!["000001".into(), "110011".into()],
                pick_watchlist: false,
                days: 90,
                period: None,
                sort: None,
                output: None,
                format: "json".into(),
            },
        )
        .await
        .unwrap();

        let raw = ctx.take_captured().expect("captured json");
        let stable = strip_volatile_envelope_fields(serde_json::from_str(&raw).unwrap());
        assert_eq!(stable["ok"], true);
        assert_eq!(stable["command"], "compare");
        assert_eq!(stable["meta"]["succeeded"], 2);
        assert_eq!(stable["data"]["items"].as_array().unwrap().len(), 2);
    }
}
