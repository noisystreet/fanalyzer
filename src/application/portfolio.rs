//! 组合分析用例：加权收益、相关矩阵、重仓重叠。

use super::concurrency::{map_concurrent, FUND_CONCURRENCY};
use super::context::CommandContext;
use super::fund_service::{analyze_fund_with_navs, fetch_nav_series, resolve_fund_identifier};
use super::queries::load_fund_holdings;
use crate::domain::{
    align_daily_returns, build_portfolio_series, correlation_matrix, daily_returns,
    metrics_from_daily_returns, normalize_rolling_window, weighted_holdings_overlap,
    weighted_portfolio_returns, EqualWeightComparison,
};
use crate::insight_config::load_portfolio_insights;
use crate::models::{
    CorrelationMatrix, OverlapPair, PortfolioMember, PortfolioReport, PortfolioSummary,
    StockHoldings,
};
use crate::portfolio::PortfolioDefinition;
use crate::presentation::{base_meta, compact_portfolio_report, emit, PortfolioMeta};
use chrono::Local;
use std::collections::HashMap;
use std::path::Path;

const DEFAULT_INSIGHTS_PATH: &str = "config/portfolio_insights.toml";

pub struct PortfolioRequest {
    pub portfolio_path: std::path::PathBuf,
    pub days: u32,
    pub period: Option<String>,
    pub holdings_top: u32,
    pub rolling_window: u32,
    pub output: Option<std::path::PathBuf>,
    pub format: String,
}

/// 组合分析核心逻辑（CLI / Web 共用）。
pub async fn gather_portfolio_report(
    ctx: &CommandContext<'_>,
    def: &PortfolioDefinition,
    days: u32,
    period: Option<&str>,
    holdings_top: u32,
    rolling_window: u32,
) -> anyhow::Result<PortfolioReport> {
    let today = Local::now().date_naive();
    let window = crate::domain::resolve_analysis_days(period, days, today)?;
    let rolling = normalize_rolling_window(rolling_window);
    tracing::info!(
        name = %def.name,
        holdings = def.holdings.len(),
        days = window,
        rolling_window = rolling,
        "Analyzing portfolio"
    );

    let members = resolve_members(&ctx.session, def, ctx.offline).await?;
    let return_series =
        fetch_return_series(&ctx.session, &members, window, ctx.offline, rolling_window).await?;
    build_report(
        def,
        &members,
        &return_series,
        window,
        ctx,
        holdings_top,
        rolling,
    )
    .await
}

pub async fn run_portfolio(ctx: &CommandContext<'_>, req: PortfolioRequest) -> anyhow::Result<()> {
    let def = crate::portfolio::load_portfolio(&req.portfolio_path)?;
    let report = gather_portfolio_report(
        ctx,
        &def,
        req.days,
        req.period.as_deref(),
        req.holdings_top,
        req.rolling_window,
    )
    .await?;
    if ctx.structured() {
        let mut report = report;
        if ctx.compact_series() {
            compact_portfolio_report(&mut report);
        }
        if ctx.offline {
            ctx.warn("离线模式：重仓重叠未计算".to_string());
        }
        let meta = PortfolioMeta {
            base: base_meta(ctx),
            days: report.summary.period_days,
            period: req.period.clone(),
            rolling_window: req.rolling_window,
            holdings: report.summary.members.len(),
        };
        return emit(
            ctx,
            "portfolio",
            &report,
            Some(&meta),
            req.output.as_deref().filter(|_| req.format == "json"),
        );
    }
    crate::presentation::render_portfolio(&report, req.output.as_deref(), &req.format)
}

#[derive(Clone)]
struct ResolvedMember {
    code: String,
    name: String,
    weight: f64,
}

async fn resolve_one_member(
    session: &super::context::Session<'_>,
    identifier: String,
    weight: f64,
    offline: bool,
) -> anyhow::Result<ResolvedMember> {
    let (code, name) = resolve_fund_identifier(session, &identifier, offline).await?;
    Ok(ResolvedMember { code, name, weight })
}

async fn resolve_members(
    session: &super::context::Session<'_>,
    def: &PortfolioDefinition,
    offline: bool,
) -> anyhow::Result<Vec<ResolvedMember>> {
    let holdings: Vec<(String, f64)> = def.holdings.clone();
    let results = map_concurrent(&holdings, FUND_CONCURRENCY, |(identifier, weight)| {
        resolve_one_member(session, identifier, weight, offline)
    })
    .await;
    results.into_iter().collect()
}

struct MemberReturns {
    label: String,
    code: String,
    name: String,
    weight: f64,
    returns: Vec<(chrono::NaiveDate, f64)>,
    analysis: Option<crate::models::FundAnalysis>,
}

async fn fetch_one_member_returns(
    session: &super::context::Session<'_>,
    member: ResolvedMember,
    days: u32,
    offline: bool,
    rolling_window: u32,
) -> anyhow::Result<MemberReturns> {
    let navs = fetch_nav_series(session, &member.code, days, offline).await?;
    if navs.is_empty() {
        anyhow::bail!("`{}` 净值数据为空，无法完成组合分析", member.code);
    }
    let returns = daily_returns(&navs);
    let analysis = analyze_fund_with_navs(
        session,
        &member.code,
        &member.name,
        &navs,
        days,
        offline,
        rolling_window,
    )
    .await?
    .map(|r| r.snapshot);
    let label = format!("{} {}", member.code, member.name);
    Ok(MemberReturns {
        label,
        code: member.code,
        name: member.name,
        weight: member.weight,
        returns,
        analysis,
    })
}

async fn fetch_return_series(
    session: &super::context::Session<'_>,
    members: &[ResolvedMember],
    days: u32,
    offline: bool,
    rolling_window: u32,
) -> anyhow::Result<Vec<MemberReturns>> {
    let results = map_concurrent(members, FUND_CONCURRENCY, |member| {
        fetch_one_member_returns(session, member, days, offline, rolling_window)
    })
    .await;
    results.into_iter().collect()
}

async fn build_report(
    def: &PortfolioDefinition,
    members: &[ResolvedMember],
    series: &[MemberReturns],
    window_days: u32,
    ctx: &CommandContext<'_>,
    holdings_top: u32,
    rolling_window: usize,
) -> anyhow::Result<PortfolioReport> {
    let labeled: Vec<(String, Vec<(chrono::NaiveDate, f64)>)> = series
        .iter()
        .map(|s| (s.label.clone(), s.returns.clone()))
        .collect();
    let (dates, aligned) = align_daily_returns(&labeled)
        .ok_or_else(|| anyhow::anyhow!("组合成分净值日期交集不足（需要 ≥2 个交易日）"))?;
    let calendar_days = (dates
        .last()
        .unwrap()
        .signed_duration_since(*dates.first().unwrap()))
    .num_days()
    .max(1) as u32;

    let weights: Vec<f64> = series.iter().map(|s| s.weight).collect();
    let portfolio_daily = weighted_portfolio_returns(&weights, &aligned);
    let metrics = metrics_from_daily_returns(&portfolio_daily, calendar_days);

    let n = series.len();
    let equal_weights = vec![1.0 / n as f64; n];
    let equal_daily = weighted_portfolio_returns(&equal_weights, &aligned);
    let equal_metrics = metrics_from_daily_returns(&equal_daily, calendar_days);
    let equal_weight = EqualWeightComparison {
        total_return: equal_metrics.total_return,
        sharpe_ratio: equal_metrics.sharpe_ratio,
        max_drawdown: equal_metrics.max_drawdown,
    };

    let summary = build_summary(def, series, window_days, dates.len() as u32, &metrics);
    let correlation = build_correlation(series, &aligned);
    let overlaps = build_overlaps(ctx, members, holdings_top).await;
    let thresholds = load_portfolio_insights(Path::new(DEFAULT_INSIGHTS_PATH));
    let interpretation = Some(crate::domain::interpret_portfolio(
        &summary,
        &correlation,
        &overlaps,
        &thresholds,
        Some(equal_weight),
    ));
    let series = build_portfolio_series(&dates, &portfolio_daily, rolling_window);

    Ok(PortfolioReport {
        summary,
        correlation,
        overlaps,
        interpretation,
        series,
    })
}

fn build_summary(
    def: &PortfolioDefinition,
    series: &[MemberReturns],
    period_days: u32,
    aligned_days: u32,
    metrics: &crate::domain::PortfolioMetrics,
) -> PortfolioSummary {
    let members = series
        .iter()
        .map(|s| {
            let a = s.analysis.as_ref();
            let total_return = a.map(|x| x.total_return).unwrap_or(0.0);
            PortfolioMember {
                code: s.code.clone(),
                name: s.name.clone(),
                weight: s.weight,
                total_return,
                volatility: a.map(|x| x.volatility).unwrap_or(0.0),
                max_drawdown: a.map(|x| x.max_drawdown).unwrap_or(0.0),
                sharpe_ratio: a.map(|x| x.sharpe_ratio).unwrap_or(0.0),
                return_contribution: s.weight * total_return,
            }
        })
        .collect();

    PortfolioSummary {
        name: def.name.clone(),
        period_days,
        aligned_days,
        total_return: metrics.total_return,
        annualized_return: metrics.annualized_return,
        volatility: metrics.volatility,
        max_drawdown: metrics.max_drawdown,
        sharpe_ratio: metrics.sharpe_ratio,
        members,
    }
}

fn build_correlation(series: &[MemberReturns], aligned: &[Vec<f64>]) -> CorrelationMatrix {
    let labels: Vec<String> = series.iter().map(|s| s.code.clone()).collect();
    CorrelationMatrix {
        labels: labels.clone(),
        values: correlation_matrix(aligned),
    }
}

async fn build_overlaps(
    ctx: &CommandContext<'_>,
    members: &[ResolvedMember],
    holdings_top: u32,
) -> Vec<OverlapPair> {
    if ctx.offline {
        tracing::warn!("`--offline` 跳过重仓重叠分析（需联网拉取 holdings）");
        return Vec::new();
    }
    let top = holdings_top.clamp(1, 50);
    let holdings = fetch_all_holdings(&ctx.session, members, top).await;
    let mut pairs = Vec::new();
    for i in 0..members.len() {
        for j in (i + 1)..members.len() {
            let Some(a) = holdings.get(&members[i].code) else {
                continue;
            };
            let Some(b) = holdings.get(&members[j].code) else {
                continue;
            };
            let (overlap, shared) = weighted_holdings_overlap(&a.rows, &b.rows);
            pairs.push(OverlapPair {
                fund_a_code: members[i].code.clone(),
                fund_a_name: members[i].name.clone(),
                fund_b_code: members[j].code.clone(),
                fund_b_name: members[j].name.clone(),
                overlap_pct: overlap * 100.0,
                shared_count: shared,
            });
        }
    }
    pairs
}

enum HoldingsFetchOutcome {
    Ok(String, StockHoldings),
    Skip,
}

async fn fetch_one_holdings(
    session: &super::context::Session<'_>,
    code: String,
    top: u32,
) -> HoldingsFetchOutcome {
    match load_fund_holdings(session, &code, top).await {
        Ok(h) => HoldingsFetchOutcome::Ok(code, h),
        Err(e) => {
            tracing::warn!(code = %code, error = %e, "跳过该标的重仓重叠");
            HoldingsFetchOutcome::Skip
        }
    }
}

async fn fetch_all_holdings(
    session: &super::context::Session<'_>,
    members: &[ResolvedMember],
    top: u32,
) -> HashMap<String, StockHoldings> {
    let codes: Vec<String> = members.iter().map(|m| m.code.clone()).collect();
    let outcomes = map_concurrent(&codes, FUND_CONCURRENCY, |code| {
        fetch_one_holdings(session, code, top)
    })
    .await;
    let mut map = HashMap::new();
    for outcome in outcomes {
        if let HoldingsFetchOutcome::Ok(code, h) = outcome {
            map.insert(code, h);
        }
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::DEFAULT_ROLLING_WINDOW;

    #[test]
    fn build_summary_contribution() {
        let def = PortfolioDefinition {
            name: "t".into(),
            holdings: vec![("000001".into(), 0.6), ("110011".into(), 0.4)],
        };
        let series = vec![
            MemberReturns {
                label: "a".into(),
                code: "000001".into(),
                name: "A".into(),
                weight: 0.6,
                returns: vec![],
                analysis: None,
            },
            MemberReturns {
                label: "b".into(),
                code: "110011".into(),
                name: "B".into(),
                weight: 0.4,
                returns: vec![],
                analysis: None,
            },
        ];
        let metrics = crate::domain::PortfolioMetrics {
            total_return: 0.1,
            annualized_return: 0.12,
            volatility: 0.15,
            max_drawdown: 0.05,
            sharpe_ratio: 0.8,
        };
        let s = build_summary(&def, &series, 90, 60, &metrics);
        assert_eq!(s.members.len(), 2);
        assert!((s.total_return - 0.1).abs() < 1e-9);
    }

    #[test]
    fn default_rolling_window_constant() {
        assert_eq!(DEFAULT_ROLLING_WINDOW, 60);
    }

    #[tokio::test]
    async fn portfolio_golden_envelope_offline() {
        use crate::application::output_profile::OutputProfile;
        use crate::application::test_support::{
            seed_offline_two_funds, strip_volatile_envelope_fields,
        };
        use crate::application::{CommandContext, FundDataSource, StructuredOutput};
        use std::fs;
        use std::path::Path;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let cache_root = dir.path().join("cache");
        let (nav_store, name_cache) =
            seed_offline_two_funds(&cache_root, &[("000001", "基金A"), ("110011", "基金B")]).await;
        let portfolio_path = dir.path().join("portfolio.toml");
        fs::write(
            &portfolio_path,
            r#"
name = "test-portfolio"

[[holdings]]
code = "000001"
weight = 0.5

[[holdings]]
code = "110011"
weight = 0.5
"#,
        )
        .unwrap();

        let client = crate::api::eastmoney::EastMoneyClient::default();
        let ctx = CommandContext::new(
            &client as &dyn FundDataSource,
            &name_cache,
            &nav_store,
            true,
            Path::new("config/watchlist.toml"),
            StructuredOutput::for_capture(OutputProfile::Standard),
        );
        run_portfolio(
            &ctx,
            PortfolioRequest {
                portfolio_path,
                days: 90,
                period: None,
                holdings_top: 10,
                rolling_window: DEFAULT_ROLLING_WINDOW,
                output: None,
                format: "json".into(),
            },
        )
        .await
        .unwrap();

        let raw = ctx.take_captured().expect("captured json");
        let stable = strip_volatile_envelope_fields(serde_json::from_str(&raw).unwrap());
        assert_eq!(stable["ok"], true);
        assert_eq!(stable["command"], "portfolio");
        assert_eq!(
            stable["data"]["summary"]["members"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
    }
}
