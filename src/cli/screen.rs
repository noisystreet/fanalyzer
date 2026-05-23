//! 从全市场排行池中按风险/费率规则筛选候选基金。

use super::analysis_sort::{parse_sort_key, sort_analyses, AnalysisSortKey};
use super::compare_output::render_comparison;
use super::fund_session::analyze_fund;
use super::rank_kind::rank_ft_code;
use super::Cli;
use crate::analysis_period::{days_for_rank_sort, resolve_analysis_days};
use crate::api::eastmoney::EastMoneyClient;
use crate::api::fund_ranking::{rank_return_for_sort, FundRankEntry};
use crate::cache::FundCache;
use crate::models::FundAnalysis;
use crate::nav_cache::NavCache;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration as StdDuration;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Default)]
pub struct ScreenFilters {
    pub max_drawdown_pct: Option<f64>,
    pub min_sharpe: Option<f64>,
    pub max_mgmt_fee_pct: Option<f64>,
    pub min_alpha_pct: Option<f64>,
    pub max_volatility_pct: Option<f64>,
    pub min_total_return_pct: Option<f64>,
    pub min_rank_return_pct: Option<f64>,
}

pub struct ScreenOpts {
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

pub fn passes_screen(a: &FundAnalysis, f: &ScreenFilters) -> bool {
    if let Some(max_dd) = f.max_drawdown_pct {
        if a.max_drawdown * 100.0 > max_dd {
            return false;
        }
    }
    if let Some(min_s) = f.min_sharpe {
        if a.sharpe_ratio < min_s {
            return false;
        }
    }
    if let Some(max_fee) = f.max_mgmt_fee_pct {
        if a.management_fee > 0.0 && a.management_fee > max_fee {
            return false;
        }
    }
    if let Some(min_a) = f.min_alpha_pct {
        if a.alpha * 100.0 < min_a {
            return false;
        }
    }
    if let Some(max_vol) = f.max_volatility_pct {
        if a.volatility * 100.0 > max_vol {
            return false;
        }
    }
    if let Some(min_ret) = f.min_total_return_pct {
        if a.total_return * 100.0 < min_ret {
            return false;
        }
    }
    true
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
    client: &EastMoneyClient,
    cache: &Arc<Mutex<FundCache>>,
    nav_store: &NavCache,
    candidates: &[&FundRankEntry],
    to_analyze: usize,
    days: u32,
    filters: &ScreenFilters,
) -> Vec<FundAnalysis> {
    let mut passed = Vec::new();
    for (i, row) in candidates.iter().take(to_analyze).enumerate() {
        match analyze_fund(client, cache, nav_store, &row.code, days, false).await {
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

pub async fn run_screen(
    cli: &Cli,
    client: &EastMoneyClient,
    cache: &Arc<Mutex<FundCache>>,
    nav_store: &NavCache,
    opts: ScreenOpts,
) -> anyhow::Result<()> {
    crate::cli::handlers::no_offline(cli.offline, "screen")?;
    let ft = rank_ft_code(&opts.kind)?;
    let sc = opts.sort.trim();
    if sc.is_empty() {
        anyhow::bail!("`--sort` 不能为空");
    }
    let pool = opts.rank_top.clamp(5, 100);
    let show = opts.limit.clamp(2, 30);
    let deep_cap = if opts.full_scan {
        pool
    } else {
        opts.deep_limit.clamp(3, pool)
    };
    let days = resolve_screen_days(opts.period.as_deref(), opts.days, sc)?;

    let page = client.fetch_fund_ranking_top(ft, sc, pool).await?;
    print_screen_header(&opts, sc, page.rows.len(), days);
    print_filter_hint(&opts.filters, sc);

    let candidates = filter_rank_pool(&page.rows, sc, opts.filters.min_rank_return_pct);
    if let Some(min_rr) = opts.filters.min_rank_return_pct {
        println!(
            "排行预筛（{} 区间收益 ≥ {:.2}%）：剩余 {} 只",
            sc,
            min_rr,
            candidates.len()
        );
        println!();
    }

    let to_analyze = if opts.full_scan {
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

    let mut passed = deep_analyze_candidates(
        client,
        cache,
        nav_store,
        &candidates,
        to_analyze,
        days,
        &opts.filters,
    )
    .await;

    sort_passed(&mut passed, opts.sort_by.as_deref())?;
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
    render_comparison(&passed, opts.output.as_deref(), &opts.format)
}

fn print_screen_header(opts: &ScreenOpts, sc: &str, pool_len: usize, days: u32) {
    println!(
        "候选池：{} 类型排行前 {}（sc={}），deep 分析窗口 {} 天（{}）",
        opts.kind,
        pool_len,
        sc,
        days,
        if opts.period.is_some() || opts.days.is_some() {
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

#[cfg(test)]
mod tests {
    use super::{passes_screen, ScreenFilters};
    use crate::models::FundAnalysis;

    fn sample(max_dd: f64, sharpe: f64, alpha: f64, vol: f64, ret: f64) -> FundAnalysis {
        FundAnalysis {
            code: "000001".into(),
            name: "x".into(),
            period_days: 90,
            avg_nav: 1.0,
            max_nav: 1.0,
            min_nav: 1.0,
            total_return: ret,
            annualized_return: ret,
            volatility: vol,
            max_drawdown: max_dd,
            sharpe_ratio: sharpe,
            sortino_ratio: sharpe,
            calmar_ratio: sharpe,
            alpha,
            beta: 1.0,
            manager_name: String::new(),
            manager_tenure_days: 0,
            manager_total_return: 0.0,
            management_fee: 1.0,
            custody_fee: 0.0,
        }
    }

    #[test]
    fn screen_filters_extended() {
        let f = ScreenFilters {
            max_drawdown_pct: Some(20.0),
            min_sharpe: Some(0.5),
            min_alpha_pct: Some(1.0),
            max_volatility_pct: Some(15.0),
            min_total_return_pct: Some(5.0),
            ..Default::default()
        };
        assert!(passes_screen(&sample(0.15, 0.6, 0.02, 0.12, 0.08), &f));
        assert!(!passes_screen(&sample(0.15, 0.6, 0.005, 0.12, 0.08), &f));
    }
}
