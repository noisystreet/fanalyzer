//! 从全市场排行池中按风险/费率规则筛选候选基金。

use super::context::{require_online, CommandContext};
use super::fund_service::analyze_fund;
use crate::api::fund_ranking::{rank_return_for_sort, FundRankEntry};
use crate::domain::{
    days_for_rank_sort, parse_sort_key, passes_screen, rank_ft_code, resolve_analysis_days,
    sort_analyses, AnalysisSortKey, ScreenFilters,
};
use crate::models::FundAnalysis;
use crate::presentation::{
    print_deep_limit_hint, print_filter_hint, print_insufficient_candidates, print_rank_prefilter,
    print_screen_header, print_screen_passed, render_comparison, ScreenHeaderContext,
};
use chrono::Local;
use std::path::PathBuf;
use std::time::Duration as StdDuration;

pub struct ScreenRequest {
    pub kind: String,
    pub sort: String,
    pub rank_top: u32,
    pub days: Option<u32>,
    pub period: Option<String>,
    pub filters: ScreenFilters,
    pub deep_limit: u32,
    pub full_scan: bool,
    pub sort_by: Option<String>,
    pub limit: u32,
    pub output: Option<PathBuf>,
    pub format: String,
}

fn resolve_screen_days(
    period: Option<&str>,
    days: Option<u32>,
    sort: &str,
    today: chrono::NaiveDate,
) -> anyhow::Result<u32> {
    if let Some(p) = period {
        return resolve_analysis_days(Some(p), days.unwrap_or(365), today);
    }
    if let Some(d) = days {
        return Ok(d);
    }
    Ok(days_for_rank_sort(sort, today))
}

fn filter_rank_pool<'a>(
    rows: &'a [FundRankEntry],
    sort: &str,
    min_rank_return_pct: Option<f64>,
) -> Vec<&'a FundRankEntry> {
    let mut out: Vec<_> = rows.iter().collect();
    if let Some(min_rr) = min_rank_return_pct {
        out.retain(|row| {
            rank_return_for_sort(row, sort)
                .map(|v| v >= min_rr)
                .unwrap_or(false)
        });
    }
    out
}

async fn deep_analyze_candidates(
    session: &super::context::Session<'_>,
    candidates: &[&FundRankEntry],
    to_analyze: usize,
    days: u32,
    filters: &ScreenFilters,
) -> Vec<FundAnalysis> {
    let mut passed = Vec::new();
    for (i, row) in candidates.iter().take(to_analyze).enumerate() {
        match analyze_fund(
            session,
            &row.code,
            days,
            false,
            crate::domain::DEFAULT_ROLLING_WINDOW,
        )
        .await
        {
            Ok(Some(r)) if passes_screen(&r.snapshot, filters) => passed.push(r.snapshot),
            Ok(Some(_)) | Ok(None) => {}
            Err(e) => tracing::warn!(code = %row.code, error = %e, "分析失败，跳过"),
        }
        if i + 1 < to_analyze {
            tokio::time::sleep(StdDuration::from_millis(200)).await;
        }
    }
    passed
}

fn sort_passed(analyses: &mut [FundAnalysis], sort_by: Option<&str>) -> anyhow::Result<()> {
    if let Some(raw) = sort_by {
        let key = parse_sort_key(raw)?;
        sort_analyses(analyses, key, key.default_desc());
    } else {
        sort_analyses(analyses, AnalysisSortKey::Sharpe, true);
    }
    Ok(())
}

pub async fn run_screen(ctx: &CommandContext<'_>, req: ScreenRequest) -> anyhow::Result<()> {
    require_online(ctx.offline, "screen")?;
    let today = Local::now().date_naive();
    let ft = rank_ft_code(&req.kind)?;
    let sc = req.sort.trim();
    if sc.is_empty() {
        anyhow::bail!("`--sort` 不能为空");
    }
    let pool = req.rank_top.clamp(5, 100);
    let show = req.limit.clamp(2, 30);
    let deep_cap = if req.full_scan {
        pool
    } else {
        req.deep_limit.clamp(3, pool)
    };
    let days = resolve_screen_days(req.period.as_deref(), req.days, sc, today)?;

    let page = ctx
        .session
        .client
        .fetch_fund_ranking_top(ft, sc, pool)
        .await?;
    print_screen_header(&ScreenHeaderContext {
        kind: &req.kind,
        sort: sc,
        pool_len: page.rows.len(),
        days,
        period_user_specified: req.period.is_some() || req.days.is_some(),
    });
    print_filter_hint(&req.filters, sc);

    let candidates = filter_rank_pool(&page.rows, sc, req.filters.min_rank_return_pct);
    if let Some(min_rr) = req.filters.min_rank_return_pct {
        print_rank_prefilter(sc, min_rr, candidates.len());
    }

    let to_analyze = if req.full_scan {
        candidates.len()
    } else {
        candidates.len().min(deep_cap as usize)
    };
    if to_analyze < candidates.len() {
        print_deep_limit_hint(to_analyze, candidates.len());
    }

    let mut passed =
        deep_analyze_candidates(&ctx.session, &candidates, to_analyze, days, &req.filters).await;

    sort_passed(&mut passed, req.sort_by.as_deref())?;
    passed.truncate(show as usize);

    if passed.len() < 2 {
        let samples: Vec<_> = passed
            .iter()
            .map(|a| {
                (
                    a.code.clone(),
                    a.name.clone(),
                    a.total_return,
                    a.max_drawdown,
                    a.sharpe_ratio,
                )
            })
            .collect();
        print_insufficient_candidates(passed.len(), &samples);
        return Ok(());
    }

    print_screen_passed(passed.len());
    render_comparison(&passed, req.output.as_deref(), &req.format)
}
