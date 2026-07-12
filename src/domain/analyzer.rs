//! 净值序列分析：优先累计净值口径，并计算 Sortino / Calmar。

use super::{BenchmarkData, FundMetaInfo};
use crate::models::{FundAnalysis, FundNav};

pub struct FundAnalyzer;

impl FundAnalyzer {
    pub fn analyze(
        navs: &[FundNav],
        period_days: u32,
        name: &str,
        benchmark: Option<&BenchmarkData>,
        meta: Option<&FundMetaInfo>,
    ) -> Option<FundAnalysis> {
        if navs.is_empty() {
            return None;
        }

        let code = navs[0].code.clone();
        let mut sorted: Vec<&FundNav> = navs.iter().collect();
        sorted.sort_by_key(|n| n.date);

        let prices: Vec<f64> = sorted.iter().map(|n| price_at(n)).collect();
        let use_acc = sorted
            .iter()
            .any(|n| n.acc_nav > 0.0 && (n.acc_nav - n.nav).abs() > 1e-6);

        let avg_nav = prices.iter().sum::<f64>() / prices.len() as f64;
        let max_nav = prices.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let min_nav = prices.iter().cloned().fold(f64::INFINITY, f64::min);

        let total_return = if sorted.len() >= 2 {
            let first = prices.first()?;
            let last = prices.last()?;
            if *first == 0.0 {
                0.0
            } else {
                (last - first) / first
            }
        } else {
            0.0
        };

        let actual_days = if sorted.len() >= 2 {
            let first_date = sorted.first()?.date;
            let last_date = sorted.last()?.date;
            (last_date - first_date).num_days().max(1) as u32
        } else {
            1
        };

        let annualized_return = if actual_days > 0 && total_return.is_finite() {
            (1.0 + total_return).powf(365.0 / actual_days as f64) - 1.0
        } else {
            0.0
        };

        let daily_returns = daily_returns_from_prices(&prices, &sorted);
        let volatility = calc_volatility_from_returns(&daily_returns);
        let max_drawdown = calc_max_drawdown(&prices);
        let sharpe_ratio = calc_sharpe_ratio(annualized_return, volatility);
        let sortino_ratio = calc_sortino_ratio(annualized_return, &daily_returns);
        let calmar_ratio = calc_calmar_ratio(annualized_return, max_drawdown);

        let (aligned_fund_returns, aligned_bm_returns) = if let Some(bm) = benchmark {
            align_returns_by_date(&sorted, &daily_returns, bm)
        } else {
            (Vec::new(), Vec::new())
        };

        let (alpha, beta) = if !aligned_fund_returns.is_empty() {
            calc_alpha_beta(&aligned_fund_returns, &aligned_bm_returns)
        } else {
            (0.0, 0.0)
        };

        let (manager_name, manager_tenure_days, manager_total_return, management_fee, custody_fee) =
            if let Some(m) = meta {
                (
                    m.manager_name.clone(),
                    m.manager_tenure_days,
                    m.manager_total_return,
                    m.management_fee,
                    m.custody_fee,
                )
            } else {
                (String::new(), 0, 0.0, 0.0, 0.0)
            };

        let _ = use_acc;

        Some(FundAnalysis {
            code,
            name: name.to_string(),
            period_days,
            avg_nav,
            max_nav,
            min_nav,
            total_return,
            annualized_return,
            volatility,
            max_drawdown,
            sharpe_ratio,
            sortino_ratio,
            calmar_ratio,
            alpha,
            beta,
            manager_name,
            manager_tenure_days,
            manager_total_return,
            management_fee,
            custody_fee,
        })
    }
}

fn price_at(n: &FundNav) -> f64 {
    if n.acc_nav > 0.0 { n.acc_nav } else { n.nav }
}

fn daily_returns_from_prices(prices: &[f64], sorted: &[&FundNav]) -> Vec<f64> {
    let mut out = Vec::new();
    for i in 1..prices.len() {
        let prev = prices[i - 1];
        let curr = prices[i];
        if prev > 0.0 {
            out.push((curr - prev) / prev);
        } else if let Some(r) = sorted[i].daily_return {
            out.push(r);
        }
    }
    out
}

fn align_returns_by_date(
    navs: &[&FundNav],
    fund_daily: &[f64],
    benchmark: &BenchmarkData,
) -> (Vec<f64>, Vec<f64>) {
    use std::collections::HashMap;

    let bm_map: HashMap<chrono::NaiveDate, f64> = benchmark
        .dates
        .iter()
        .zip(benchmark.returns.iter())
        .map(|(d, r)| (*d, *r))
        .collect();

    let mut fund_returns = Vec::new();
    let mut bm_returns = Vec::new();

    for (i, nav) in navs.iter().enumerate().skip(1) {
        let idx = i - 1;
        if let Some(&bm_return) = bm_map.get(&nav.date)
            && let Some(&fund_return) = fund_daily.get(idx)
        {
            fund_returns.push(fund_return);
            bm_returns.push(bm_return);
        }
    }

    (fund_returns, bm_returns)
}

fn calc_alpha_beta(fund_returns: &[f64], benchmark_returns: &[f64]) -> (f64, f64) {
    if fund_returns.len() < 2 || benchmark_returns.len() < 2 {
        return (0.0, 0.0);
    }

    let fund_mean = fund_returns.iter().sum::<f64>() / fund_returns.len() as f64;
    let bm_mean = benchmark_returns.iter().sum::<f64>() / benchmark_returns.len() as f64;

    let covariance: f64 = fund_returns
        .iter()
        .zip(benchmark_returns.iter())
        .map(|(f, b)| (f - fund_mean) * (b - bm_mean))
        .sum::<f64>()
        / (fund_returns.len() - 1) as f64;

    let bm_variance: f64 = benchmark_returns
        .iter()
        .map(|b| (b - bm_mean).powi(2))
        .sum::<f64>()
        / (benchmark_returns.len() - 1) as f64;

    let beta = if bm_variance > 0.0 {
        covariance / bm_variance
    } else {
        0.0
    };

    const RISK_FREE_RATE_DAILY: f64 = 0.03 / 252.0;
    let alpha_daily = fund_mean - (RISK_FREE_RATE_DAILY + beta * (bm_mean - RISK_FREE_RATE_DAILY));
    let alpha_annualized = alpha_daily * 252.0;

    (alpha_annualized, beta)
}

fn calc_volatility_from_returns(returns: &[f64]) -> f64 {
    if returns.len() < 2 {
        return 0.0;
    }

    let mean = returns.iter().sum::<f64>() / returns.len() as f64;
    let variance =
        returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (returns.len() - 1) as f64;

    variance.sqrt() * (252.0_f64).sqrt()
}

fn calc_max_drawdown(nav_values: &[f64]) -> f64 {
    if nav_values.len() < 2 {
        return 0.0;
    }

    let mut peak = nav_values[0];
    let mut max_dd = 0.0;

    for &nav in &nav_values[1..] {
        if nav > peak {
            peak = nav;
        }
        let dd = (peak - nav) / peak;
        if dd > max_dd {
            max_dd = dd;
        }
    }

    max_dd
}

fn calc_sharpe_ratio(annualized_return: f64, volatility: f64) -> f64 {
    const RISK_FREE_RATE: f64 = 0.03;

    if volatility == 0.0 || !volatility.is_finite() {
        return 0.0;
    }

    (annualized_return - RISK_FREE_RATE) / volatility
}

fn calc_sortino_ratio(annualized_return: f64, daily_returns: &[f64]) -> f64 {
    const RISK_FREE_RATE: f64 = 0.03;
    if daily_returns.len() < 2 {
        return 0.0;
    }
    let rf_daily = RISK_FREE_RATE / 252.0;
    let downside: Vec<f64> = daily_returns
        .iter()
        .map(|r| (r - rf_daily).min(0.0))
        .collect();
    let n = downside.len() as f64;
    let downside_var = downside.iter().map(|d| d.powi(2)).sum::<f64>() / n;
    let downside_dev = downside_var.sqrt() * (252.0_f64).sqrt();
    if downside_dev <= 0.0 || !downside_dev.is_finite() {
        return 0.0;
    }
    (annualized_return - RISK_FREE_RATE) / downside_dev
}

fn calc_calmar_ratio(annualized_return: f64, max_drawdown: f64) -> f64 {
    if max_drawdown <= 0.0 || !max_drawdown.is_finite() {
        return 0.0;
    }
    annualized_return / max_drawdown
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn make_nav(
        code: &str,
        date: &str,
        nav: f64,
        acc_nav: f64,
        daily_return: Option<f64>,
    ) -> FundNav {
        FundNav {
            code: code.to_string(),
            date: NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap(),
            nav,
            acc_nav,
            daily_return,
        }
    }

    #[test]
    fn acc_nav_total_return() {
        let navs = vec![
            make_nav("000001", "2026-01-01", 1.0, 1.0, None),
            make_nav("000001", "2026-01-10", 1.0, 1.2, None),
        ];
        let r = FundAnalyzer::analyze(&navs, 10, "t", None, None).unwrap();
        assert!((r.total_return - 0.2).abs() < 1e-6);
    }

    #[test]
    fn sortino_positive() {
        let navs = vec![
            make_nav("000001", "2026-01-01", 1.0, 1.0, None),
            make_nav("000001", "2026-01-02", 1.01, 1.01, Some(0.01)),
            make_nav("000001", "2026-01-03", 1.02, 1.02, Some(0.0099)),
            make_nav("000001", "2026-01-04", 1.03, 1.03, Some(0.0098)),
        ];
        let r = FundAnalyzer::analyze(&navs, 4, "t", None, None).unwrap();
        assert!(r.sortino_ratio >= 0.0);
        assert!(r.calmar_ratio >= 0.0);
    }
}
