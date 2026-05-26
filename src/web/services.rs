//! Web 用例：复用 application / domain 层。

use super::state::AppState;
use crate::application::{
    analyze_fund, gather_brief, gather_compare_analyses, load_fund_overview, sort_compare_analyses,
};
use crate::domain::resolve_analysis_days;
use crate::models::{FundAnalysis, FundBrief, FundOverview};
use chrono::Local;

pub async fn analyze_one(
    state: &AppState,
    code: &str,
    days: u32,
    period: Option<&str>,
) -> anyhow::Result<Option<FundAnalysis>> {
    let today = Local::now().date_naive();
    let window = resolve_analysis_days(period, days, today)?;
    let ctx = state.command_context();
    analyze_fund(&ctx.session, code.trim(), window, false).await
}

pub async fn fetch_overview(state: &AppState, code: &str) -> anyhow::Result<FundOverview> {
    let ctx = state.command_context();
    load_fund_overview(&ctx.session, code.trim()).await
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
    let ctx = state.command_context();
    gather_brief(
        &ctx.session,
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
    let ctx = state.command_context();
    let mut analyses = gather_compare_analyses(&ctx.session, codes, window, false).await;
    sort_compare_analyses(&mut analyses, sort)?;
    Ok(analyses)
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
