//! Web 用例：复用 application / domain 层。

use super::state::AppState;
use crate::application::{
    analyze_fund, gather_brief, gather_compare_analyses, gather_portfolio_report,
    load_fund_overview, sort_compare_analyses, PortfolioGatherRequest,
};
use crate::domain::resolve_analysis_days;
use crate::models::{FundAnalysis, FundAnalysisReport, FundBrief, FundOverview, PortfolioReport};
use crate::portfolio::PortfolioDefinition;
use chrono::Local;

pub async fn analyze_one(
    state: &AppState,
    code: &str,
    days: u32,
    period: Option<&str>,
    rolling_window: u32,
) -> anyhow::Result<Option<FundAnalysisReport>> {
    let today = Local::now().date_naive();
    let window = resolve_analysis_days(period, days, today)?;
    analyze_fund(&state.session(), code.trim(), window, false, rolling_window).await
}

pub async fn fetch_overview(state: &AppState, code: &str) -> anyhow::Result<FundOverview> {
    load_fund_overview(&state.session(), code.trim()).await
}

pub async fn build_brief(
    state: &AppState,
    code: &str,
    days: u32,
    period: Option<&str>,
    industry_top: u32,
    holdings_top: u32,
) -> anyhow::Result<FundBrief> {
    let today = Local::now().date_naive();
    let window = resolve_analysis_days(period, days, today)?;
    gather_brief(
        &state.session(),
        code.trim(),
        window,
        holdings_top,
        industry_top,
    )
    .await
}

pub async fn compare_funds(
    state: &AppState,
    codes: &[String],
    days: u32,
    period: Option<&str>,
    sort: Option<&str>,
) -> anyhow::Result<Vec<FundAnalysis>> {
    let today = Local::now().date_naive();
    let window = resolve_analysis_days(period, days, today)?;
    let mut gather = gather_compare_analyses(&state.session(), codes, window, false).await;
    sort_compare_analyses(&mut gather.items, sort)?;
    Ok(gather.items)
}

pub async fn analyze_portfolio(
    state: &AppState,
    def: &PortfolioDefinition,
    days: u32,
    period: Option<&str>,
    holdings_top: u32,
    rolling_window: u32,
) -> anyhow::Result<PortfolioReport> {
    gather_portfolio_report(
        &state.session(),
        def,
        PortfolioGatherRequest {
            days,
            period: period.map(str::to_string),
            holdings_top,
            rolling_window,
            offline: false,
        },
    )
    .await
}

pub fn parse_code_list(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_codes_splits_commas() {
        assert_eq!(
            parse_code_list("000001, 110011 ,"),
            vec!["000001".to_string(), "110011".to_string()]
        );
    }
}
