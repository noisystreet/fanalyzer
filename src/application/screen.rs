//! 从全市场排行池中按风险/费率规则筛选候选基金。

use super::context::{require_online, CommandContext};
use super::fund_service::analyze_fund;
use crate::api::fund_ranking::{rank_return_for_sort, FundRankEntry};
use crate::domain::{
    days_for_rank_sort, parse_sort_key, passes_screen, rank_ft_code, resolve_analysis_days,
    sort_analyses, AnalysisSortKey, ScreenFilters,
};
use crate::models::FundAnalysis;
use crate::presentation::render_comparison;
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

fn resolve_screen_days(period: Option<&str>, days: Option<u32>, sort: &str) -> anyhow::Result<u32> {
    if let Some(p) = period {
        return resolve_analysis_days(Some(p), days.unwrap_or(365));
    }
    if let Some(d) = days {
        return Ok(d);
    }
    Ok(days_for_rank_sort(sort))
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
        match analyze_fund(session, &row.code, days, false).await {
            Ok(Some(a)) if passes_screen(&a, filters) => passed.push(a),
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

fn print_insufficient(passed: &[FundAnalysis]) {
    println!(
        "筛选后有效样本 {} 只（需 ≥2 才能对比）；可放宽筛选条件或增大 --rank-top / --deep-limit",
        passed.len()
    );
    for a in passed {
        println!(
            "  {} {}  收益 {:.2}%  回撤 {:.2}%  夏普 {:.2}",
            a.code,
            a.name,
            a.total_return * 100.0,
            a.max_drawdown * 100.0,
            a.sharpe_ratio
        );
    }
}

pub async fn run_screen(ctx: &CommandContext<'_>, req: ScreenRequest) -> anyhow::Result<()> {
    require_online(ctx.offline, "screen")?;
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
    let days = resolve_screen_days(req.period.as_deref(), req.days, sc)?;

    let page = ctx
        .session
        .client
        .fetch_fund_ranking_top(ft, sc, pool)
        .await?;
    print_screen_header(&req, sc, page.rows.len(), days);
    print_filter_hint(&req.filters, sc);

    let candidates = filter_rank_pool(&page.rows, sc, req.filters.min_rank_return_pct);
    if let Some(min_rr) = req.filters.min_rank_return_pct {
        println!(
            "排行预筛（{} 区间收益 ≥ {:.2}%）：剩余 {} 只",
            sc,
            min_rr,
            candidates.len()
        );
        println!();
    }

    let to_analyze = if req.full_scan {
        candidates.len()
    } else {
        candidates.len().min(deep_cap as usize)
    };
    if to_analyze < candidates.len() {
        println!(
            "deep 分析前 {} 只（共 {} 只候选；加 --full-scan 扫描全部）",
            to_analyze,
            candidates.len()
        );
        println!();
    }

    let mut passed =
        deep_analyze_candidates(&ctx.session, &candidates, to_analyze, days, &req.filters).await;

    sort_passed(&mut passed, req.sort_by.as_deref())?;
    passed.truncate(show as usize);

    if passed.len() < 2 {
        print_insufficient(&passed);
        return Ok(());
    }

    println!(
        "通过筛选 {} 只，展示前 {} 只对比：",
        passed.len(),
        passed.len()
    );
    println!();
    render_comparison(&passed, req.output.as_deref(), &req.format)
}

fn print_screen_header(req: &ScreenRequest, sc: &str, pool_len: usize, days: u32) {
    println!(
        "候选池：{} 类型排行前 {}（sc={}），deep 分析窗口 {} 天（{}）",
        req.kind,
        pool_len,
        sc,
        days,
        if req.period.is_some() || req.days.is_some() {
            "用户指定"
        } else {
            "与 sort 对齐"
        }
    );
}

fn print_filter_hint(f: &ScreenFilters, sort: &str) {
    let mut parts = Vec::new();
    if let Some(rr) = f.min_rank_return_pct {
        parts.push(format!("排行 {sort} 收益 ≥ {rr:.2}%"));
    }
    if let Some(dd) = f.max_drawdown_pct {
        parts.push(format!("最大回撤 ≤ {dd:.1}%"));
    }
    if let Some(s) = f.min_sharpe {
        parts.push(format!("夏普 ≥ {s:.2}"));
    }
    if let Some(fee) = f.max_mgmt_fee_pct {
        parts.push(format!("管理费 ≤ {fee:.2}%"));
    }
    if let Some(a) = f.min_alpha_pct {
        parts.push(format!("Alpha ≥ {a:.2}%"));
    }
    if let Some(v) = f.max_volatility_pct {
        parts.push(format!("波动率 ≤ {v:.1}%"));
    }
    if let Some(r) = f.min_total_return_pct {
        parts.push(format!("区间总收益 ≥ {r:.2}%"));
    }
    if parts.is_empty() {
        println!("筛选条件：（无额外约束，deep 分析后全部展示）");
    } else {
        println!("筛选条件：{}", parts.join("；"));
    }
    println!();
}
