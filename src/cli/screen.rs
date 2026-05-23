//! 从全市场排行池中按风险/费率规则筛选候选基金。

use super::fund_session::analyze_fund;
use super::output::print_comparison;
use super::rank_kind::rank_ft_code;
use super::Cli;
use crate::api::eastmoney::EastMoneyClient;
use crate::cache::FundCache;
use crate::models::FundAnalysis;
use crate::nav_cache::NavCache;
use std::sync::Arc;
use std::time::Duration as StdDuration;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Default)]
pub struct ScreenFilters {
    /// 最大回撤上限（百分点，如 25 表示剔除回撤 >25% 的基金）
    pub max_drawdown_pct: Option<f64>,
    pub min_sharpe: Option<f64>,
    /// 管理费率上限（百分点，如 1.5）
    pub max_mgmt_fee_pct: Option<f64>,
}

/// `screen` 子命令参数。
pub struct ScreenOpts {
    pub kind: String,
    pub sort: String,
    pub rank_top: u32,
    pub days: u32,
    pub filters: ScreenFilters,
    pub limit: u32,
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
    true
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

    tracing::info!(ft = ft, pool = pool, sort = %sc, "Screening from rank pool");
    let page = client.fetch_fund_ranking_top(ft, sc, pool).await?;
    println!(
        "候选池：{} 类型排行前 {}（sc={}），分析窗口 {} 天",
        opts.kind,
        page.rows.len(),
        sc,
        opts.days
    );
    print_filter_hint(&opts.filters);
    println!();

    let mut passed: Vec<FundAnalysis> = Vec::new();
    for (i, row) in page.rows.iter().enumerate() {
        match analyze_fund(client, cache, nav_store, &row.code, opts.days, false).await {
            Ok(Some(a)) if passes_screen(&a, &opts.filters) => passed.push(a),
            Ok(Some(_)) => {}
            Ok(None) => tracing::warn!(code = %row.code, "分析数据不足，跳过"),
            Err(e) => tracing::warn!(code = %row.code, error = %e, "分析失败，跳过"),
        }
        if i + 1 < page.rows.len() {
            tokio::time::sleep(StdDuration::from_millis(200)).await;
        }
    }

    passed.truncate(show as usize);
    if passed.len() < 2 {
        println!(
            "筛选后有效样本 {} 只（需 ≥2 才能对比）；可放宽 --max-drawdown / --min-sharpe / --max-mgmt-fee 或增大 --rank-top",
            passed.len()
        );
        for a in &passed {
            println!(
                "  {} {}  回撤 {:.2}%  夏普 {:.2}",
                a.code,
                a.name,
                a.max_drawdown * 100.0,
                a.sharpe_ratio
            );
        }
        return Ok(());
    }

    println!(
        "通过筛选 {} 只，展示前 {} 只对比：",
        passed.len(),
        passed.len()
    );
    println!();
    print_comparison(&passed);
    Ok(())
}

fn print_filter_hint(f: &ScreenFilters) {
    let mut parts = Vec::new();
    if let Some(dd) = f.max_drawdown_pct {
        parts.push(format!("最大回撤 ≤ {dd:.1}%"));
    }
    if let Some(s) = f.min_sharpe {
        parts.push(format!("夏普 ≥ {s:.2}"));
    }
    if let Some(fee) = f.max_mgmt_fee_pct {
        parts.push(format!("管理费 ≤ {fee:.2}%"));
    }
    if parts.is_empty() {
        println!("筛选条件：（无额外约束，仅按排行池顺序分析）");
    } else {
        println!("筛选条件：{}", parts.join("；"));
    }
}

#[cfg(test)]
mod tests {
    use super::{passes_screen, ScreenFilters};
    use crate::models::FundAnalysis;

    fn sample_analysis(max_dd: f64, sharpe: f64, mgmt: f64) -> FundAnalysis {
        FundAnalysis {
            code: "000001".into(),
            name: "x".into(),
            period_days: 90,
            avg_nav: 1.0,
            max_nav: 1.0,
            min_nav: 1.0,
            total_return: 0.1,
            annualized_return: 0.1,
            volatility: 0.1,
            max_drawdown: max_dd,
            sharpe_ratio: sharpe,
            alpha: 0.0,
            beta: 1.0,
            manager_name: String::new(),
            manager_tenure_days: 0,
            manager_total_return: 0.0,
            management_fee: mgmt,
            custody_fee: 0.0,
        }
    }

    #[test]
    fn screen_filters_drawdown_and_sharpe() {
        let f = ScreenFilters {
            max_drawdown_pct: Some(20.0),
            min_sharpe: Some(0.5),
            max_mgmt_fee_pct: None,
        };
        assert!(passes_screen(&sample_analysis(0.15, 0.6, 1.0), &f));
        assert!(!passes_screen(&sample_analysis(0.25, 0.6, 1.0), &f));
        assert!(!passes_screen(&sample_analysis(0.15, 0.3, 1.0), &f));
    }
}
