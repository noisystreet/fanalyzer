//! Web 用例：复用 application / domain 层。

use super::state::AppState;
use crate::application::analyze_fund;
use crate::domain::{parse_sort_key, resolve_analysis_days, sort_analyses, AnalysisSortKey};
use crate::models::FundAnalysis;
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
    let mut analyses = Vec::new();
    for code in codes {
        match analyze_fund(&ctx.session, code.trim(), window, false).await {
            Ok(Some(a)) => analyses.push(a),
            Ok(None) => {}
            Err(e) => tracing::warn!(code = %code, error = %e, "compare skip"),
        }
    }
    if let Some(raw) = sort.filter(|s| !s.is_empty()) {
        let key = parse_sort_key(raw)?;
        sort_analyses(&mut analyses, key, key.default_desc());
    } else {
        sort_analyses(&mut analyses, AnalysisSortKey::Code, false);
    }
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
