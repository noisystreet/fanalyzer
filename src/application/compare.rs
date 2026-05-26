//! 多基金对比用例。

use super::context::{resolve_many_fund_ids, CommandContext, Session};
use super::fund_service;
use crate::domain::{parse_sort_key, resolve_analysis_days, sort_analyses, AnalysisSortKey};
use crate::models::FundAnalysis;
use crate::presentation::render_comparison;
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

async fn try_push_analysis(
    session: &Session<'_>,
    identifier: &str,
    days: u32,
    offline: bool,
    out: &mut Vec<FundAnalysis>,
) {
    match fund_service::analyze_fund(session, identifier, days, offline).await {
        Ok(Some(r)) => out.push(r.snapshot),
        Ok(None) => tracing::warn!(identifier = %identifier, "分析数据不足，跳过"),
        Err(e) => tracing::warn!(identifier = %identifier, error = %e, "跳过该标的"),
    }
}

/// 批量分析多只基金，跳过数据不足或失败的标的。
pub async fn gather_compare_analyses(
    session: &Session<'_>,
    identifiers: &[String],
    days: u32,
    offline: bool,
) -> Vec<FundAnalysis> {
    let mut analyses = Vec::new();
    for identifier in identifiers {
        try_push_analysis(session, identifier, days, offline, &mut analyses).await;
    }
    analyses
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

    let mut analyses = gather_compare_analyses(&ctx.session, &ids, days, ctx.offline).await;

    if analyses.len() < 2 {
        tracing::warn!("有效样本不足（需要≥2）；请检查离线缓存或数据源");
        return Ok(());
    }

    sort_compare_analyses(&mut analyses, req.sort.as_deref())?;

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
}
