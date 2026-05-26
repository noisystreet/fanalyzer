//! 组合分析用例：加权收益、相关矩阵、重仓重叠。

use super::context::CommandContext;
use super::fund_service::{analyze_fund, fetch_nav_series, resolve_fund_identifier};
use super::queries::load_fund_holdings;
use crate::domain::{
    align_daily_returns, build_portfolio_series, correlation_matrix, daily_returns,
    metrics_from_daily_returns, weighted_holdings_overlap, weighted_portfolio_returns,
    DEFAULT_ROLLING_WINDOW,
};
use crate::models::{
    CorrelationMatrix, OverlapPair, PortfolioMember, PortfolioReport, PortfolioSummary,
    StockHoldings,
};
use crate::portfolio::PortfolioDefinition;
use chrono::Local;
use std::collections::HashMap;

pub struct PortfolioRequest {
    pub portfolio_path: std::path::PathBuf,
    pub days: u32,
    pub period: Option<String>,
    pub holdings_top: u32,
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
) -> anyhow::Result<PortfolioReport> {
    let today = Local::now().date_naive();
    let window = crate::domain::resolve_analysis_days(period, days, today)?;
    tracing::info!(
        name = %def.name,
        holdings = def.holdings.len(),
        days = window,
        "Analyzing portfolio"
    );

    let members = resolve_members(&ctx.session, def, ctx.offline).await?;
    let return_series = fetch_return_series(&ctx.session, &members, window, ctx.offline).await?;
    build_report(def, &members, &return_series, window, ctx, holdings_top).await
}

pub async fn run_portfolio(ctx: &CommandContext<'_>, req: PortfolioRequest) -> anyhow::Result<()> {
    let def = crate::portfolio::load_portfolio(&req.portfolio_path)?;
    let report =
        gather_portfolio_report(ctx, &def, req.days, req.period.as_deref(), req.holdings_top)
            .await?;
    crate::presentation::render_portfolio(&report, req.output.as_deref(), &req.format)
}

struct ResolvedMember {
    code: String,
    name: String,
    weight: f64,
}

async fn resolve_members(
    session: &super::context::Session<'_>,
    def: &PortfolioDefinition,
    offline: bool,
) -> anyhow::Result<Vec<ResolvedMember>> {
    let mut out = Vec::with_capacity(def.holdings.len());
    for (identifier, weight) in &def.holdings {
        let (code, name) = resolve_fund_identifier(session, identifier, offline).await?;
        out.push(ResolvedMember {
            code,
            name,
            weight: *weight,
        });
    }
    Ok(out)
}

struct MemberReturns {
    label: String,
    code: String,
    name: String,
    weight: f64,
    returns: Vec<(chrono::NaiveDate, f64)>,
    analysis: Option<crate::models::FundAnalysis>,
}

async fn fetch_return_series(
    session: &super::context::Session<'_>,
    members: &[ResolvedMember],
    days: u32,
    offline: bool,
) -> anyhow::Result<Vec<MemberReturns>> {
    let mut out = Vec::with_capacity(members.len());
    for m in members {
        let navs = fetch_nav_series(session, &m.code, days, offline).await?;
        if navs.is_empty() {
            anyhow::bail!("`{}` 净值数据为空，无法完成组合分析", m.code);
        }
        let returns = daily_returns(&navs);
        let analysis = analyze_fund(session, &m.code, days, offline)
            .await?
            .map(|r| r.snapshot);
        let label = format!("{} {}", m.code, m.name);
        out.push(MemberReturns {
            label,
            code: m.code.clone(),
            name: m.name.clone(),
            weight: m.weight,
            returns,
            analysis,
        });
    }
    Ok(out)
}

async fn build_report(
    def: &PortfolioDefinition,
    members: &[ResolvedMember],
    series: &[MemberReturns],
    window_days: u32,
    ctx: &CommandContext<'_>,
    holdings_top: u32,
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

    let summary = build_summary(def, series, window_days, dates.len() as u32, &metrics);
    let correlation = build_correlation(series, &aligned);
    let overlaps = build_overlaps(ctx, members, holdings_top).await;
    let interpretation = Some(crate::domain::interpret_portfolio(
        &summary,
        &correlation,
        &overlaps,
    ));
    let series = build_portfolio_series(&dates, &portfolio_daily, DEFAULT_ROLLING_WINDOW);

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

async fn fetch_all_holdings(
    session: &super::context::Session<'_>,
    members: &[ResolvedMember],
    top: u32,
) -> HashMap<String, StockHoldings> {
    let mut map = HashMap::new();
    for m in members {
        match load_fund_holdings(session, &m.code, top).await {
            Ok(h) => {
                map.insert(m.code.clone(), h);
            }
            Err(e) => {
                tracing::warn!(code = %m.code, error = %e, "跳过该标的重仓重叠");
            }
        }
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
