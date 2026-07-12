//! 净值序列日收益对齐与组合指标（纯计算）。

use crate::models::FundNav;
use chrono::NaiveDate;

/// 组合层风险收益摘要。
#[derive(Debug, Clone, PartialEq)]
pub struct PortfolioMetrics {
    pub total_return: f64,
    pub annualized_return: f64,
    pub volatility: f64,
    pub max_drawdown: f64,
    pub sharpe_ratio: f64,
}

pub fn nav_price(n: &FundNav) -> f64 {
    if n.acc_nav > 0.0 { n.acc_nav } else { n.nav }
}

/// 按日期升序提取日收益；键为收益归属日（当日净值相对前一日）。
pub fn daily_returns(navs: &[FundNav]) -> Vec<(NaiveDate, f64)> {
    let mut sorted: Vec<&FundNav> = navs.iter().collect();
    sorted.sort_by_key(|n| n.date);
    let mut out = Vec::new();
    for i in 1..sorted.len() {
        let prev = nav_price(sorted[i - 1]);
        let curr = nav_price(sorted[i]);
        let r = if prev > 0.0 {
            (curr - prev) / prev
        } else {
            sorted[i].daily_return.unwrap_or(0.0)
        };
        out.push((sorted[i].date, r));
    }
    out
}

/// 取各序列日期交集，按 `labels` 顺序返回对齐后的日收益矩阵。
pub fn align_daily_returns(
    series: &[(String, Vec<(NaiveDate, f64)>)],
) -> Option<(Vec<NaiveDate>, Vec<Vec<f64>>)> {
    if series.len() < 2 {
        return None;
    }
    let mut common: Option<std::collections::HashSet<NaiveDate>> = None;
    for (_, pts) in series {
        let dates: std::collections::HashSet<NaiveDate> = pts.iter().map(|(d, _)| *d).collect();
        common = Some(match common {
            None => dates,
            Some(prev) => prev.intersection(&dates).copied().collect(),
        });
    }
    let mut dates: Vec<NaiveDate> = common?.into_iter().collect();
    dates.sort();
    if dates.len() < 2 {
        return None;
    }
    let mut aligned = Vec::with_capacity(series.len());
    for (_, pts) in series {
        let map: std::collections::HashMap<NaiveDate, f64> =
            pts.iter().map(|(d, r)| (*d, *r)).collect();
        aligned.push(dates.iter().map(|d| map[d]).collect());
    }
    Some((dates, aligned))
}

pub fn pearson_correlation(a: &[f64], b: &[f64]) -> f64 {
    if a.len() < 2 || a.len() != b.len() {
        return 0.0;
    }
    let n = a.len() as f64;
    let mean_a = a.iter().sum::<f64>() / n;
    let mean_b = b.iter().sum::<f64>() / n;
    let mut cov = 0.0;
    let mut var_a = 0.0;
    let mut var_b = 0.0;
    for (x, y) in a.iter().zip(b.iter()) {
        let dx = x - mean_a;
        let dy = y - mean_b;
        cov += dx * dy;
        var_a += dx * dx;
        var_b += dy * dy;
    }
    let denom = (var_a * var_b).sqrt();
    if denom > 0.0 { cov / denom } else { 0.0 }
}

pub fn correlation_matrix(data: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let n = data.len();
    let mut out = vec![vec![0.0; n]; n];
    for i in 0..n {
        out[i][i] = 1.0;
        for j in (i + 1)..n {
            let c = pearson_correlation(&data[i], &data[j]);
            out[i][j] = c;
            out[j][i] = c;
        }
    }
    out
}

pub fn weighted_portfolio_returns(weights: &[f64], aligned: &[Vec<f64>]) -> Vec<f64> {
    let days = aligned.first().map(|v| v.len()).unwrap_or(0);
    let mut out = Vec::with_capacity(days);
    for t in 0..days {
        let mut r = 0.0;
        for (w, series) in weights.iter().zip(aligned.iter()) {
            r += w * series[t];
        }
        out.push(r);
    }
    out
}

pub fn metrics_from_daily_returns(daily: &[f64], calendar_days: u32) -> PortfolioMetrics {
    if daily.is_empty() {
        return PortfolioMetrics {
            total_return: 0.0,
            annualized_return: 0.0,
            volatility: 0.0,
            max_drawdown: 0.0,
            sharpe_ratio: 0.0,
        };
    }
    let mut price = 1.0;
    let mut prices = vec![1.0];
    for r in daily {
        price *= 1.0 + r;
        prices.push(price);
    }
    let total_return = price - 1.0;
    let days = calendar_days.max(1) as f64;
    let annualized_return = if total_return.is_finite() {
        (1.0 + total_return).powf(365.0 / days) - 1.0
    } else {
        0.0
    };
    let volatility = calc_volatility(daily);
    let max_drawdown = calc_max_drawdown(&prices);
    let sharpe_ratio = calc_sharpe(annualized_return, volatility);
    PortfolioMetrics {
        total_return,
        annualized_return,
        volatility,
        max_drawdown,
        sharpe_ratio,
    }
}

fn calc_volatility(returns: &[f64]) -> f64 {
    if returns.len() < 2 {
        return 0.0;
    }
    let mean = returns.iter().sum::<f64>() / returns.len() as f64;
    let variance =
        returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (returns.len() - 1) as f64;
    variance.sqrt() * (252.0_f64).sqrt()
}

fn calc_max_drawdown(prices: &[f64]) -> f64 {
    if prices.len() < 2 {
        return 0.0;
    }
    let mut peak = prices[0];
    let mut max_dd = 0.0;
    for &p in &prices[1..] {
        if p > peak {
            peak = p;
        }
        let dd = if peak > 0.0 { (peak - p) / peak } else { 0.0 };
        if dd > max_dd {
            max_dd = dd;
        }
    }
    max_dd
}

fn calc_sharpe(annualized_return: f64, volatility: f64) -> f64 {
    const RISK_FREE_RATE: f64 = 0.03;
    if volatility <= 0.0 || !volatility.is_finite() {
        0.0
    } else {
        (annualized_return - RISK_FREE_RATE) / volatility
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn nav(code: &str, date: &str, acc: f64) -> FundNav {
        FundNav {
            code: code.to_string(),
            date: NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap(),
            nav: acc,
            acc_nav: acc,
            daily_return: None,
        }
    }

    #[test]
    fn align_daily_returns_intersection() {
        let a = (
            "A".into(),
            vec![
                (NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(), 0.01),
                (NaiveDate::from_ymd_opt(2026, 1, 3).unwrap(), 0.02),
            ],
        );
        let b = (
            "B".into(),
            vec![
                (NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(), 0.005),
                (NaiveDate::from_ymd_opt(2026, 1, 3).unwrap(), -0.01),
            ],
        );
        let (dates, aligned) = align_daily_returns(&[a, b]).unwrap();
        assert_eq!(dates.len(), 2);
        assert_eq!(aligned.len(), 2);
    }

    #[test]
    fn correlation_perfect_positive() {
        let a = vec![0.01, 0.02, -0.01, 0.015];
        let b = vec![0.02, 0.04, -0.02, 0.03];
        assert!((pearson_correlation(&a, &b) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn weighted_portfolio_equal_weight() {
        let aligned = vec![vec![0.10, 0.0], vec![0.0, 0.10]];
        let daily = weighted_portfolio_returns(&[0.5, 0.5], &aligned);
        assert!((daily[0] - 0.05).abs() < 1e-9);
        assert!((daily[1] - 0.05).abs() < 1e-9);
    }

    #[test]
    fn metrics_from_flat_returns() {
        let daily = vec![0.01; 20];
        let m = metrics_from_daily_returns(&daily, 20);
        assert!(m.total_return > 0.0);
        assert!(m.volatility >= 0.0);
    }

    #[test]
    fn daily_returns_from_navs() {
        let navs = vec![
            nav("000001", "2026-01-01", 1.0),
            nav("000001", "2026-01-02", 1.1),
        ];
        let dr = daily_returns(&navs);
        assert_eq!(dr.len(), 1);
        assert!((dr[0].1 - 0.1).abs() < 1e-9);
    }

    #[test]
    fn correlation_perfect_negative() {
        let a = vec![0.01, 0.02, -0.01, 0.015];
        let b = vec![-0.01, -0.02, 0.01, -0.015];
        assert!((pearson_correlation(&a, &b) - (-1.0)).abs() < 1e-6);
    }

    #[test]
    fn correlation_uncorrelated() {
        let a = vec![0.01, -0.01, 0.01, -0.01];
        let b = vec![1.0, -1.0, 1.0, -1.0];
        // Perfectly correlated (b = 100 * a)
        assert!((pearson_correlation(&a, &b).abs() - 1.0) < 1e-6);
    }

    #[test]
    fn correlation_matrix_two_funds() {
        let aligned = vec![vec![0.01, 0.02, -0.01], vec![-0.01, -0.02, 0.01]];
        let mat = correlation_matrix(&aligned);
        assert_eq!(mat.len(), 2);
        assert_eq!(mat[0].len(), 2);
        assert!((mat[0][0] - 1.0).abs() < 1e-6);
        assert!((mat[1][1] - 1.0).abs() < 1e-6);
        assert!((mat[0][1] - (-1.0)).abs() < 1e-6);
        assert!((mat[1][0] - (-1.0)).abs() < 1e-6);
    }

    #[test]
    fn nav_price_uses_acc_nav_first() {
        let nav = FundNav {
            code: "000001".into(),
            date: NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(),
            nav: 1.0,
            acc_nav: 1.5,
            daily_return: None,
        };
        assert!((nav_price(&nav) - 1.5).abs() < 1e-9);
    }

    #[test]
    fn nav_price_falls_back_to_nav() {
        let nav = FundNav {
            code: "000001".into(),
            date: NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(),
            nav: 1.0,
            acc_nav: 0.0,
            daily_return: None,
        };
        assert!((nav_price(&nav) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn align_daily_returns_unequal_length() {
        let a = (
            "A".into(),
            vec![
                (NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(), 0.01),
                (NaiveDate::from_ymd_opt(2026, 1, 3).unwrap(), 0.02),
                (NaiveDate::from_ymd_opt(2026, 1, 4).unwrap(), 0.03),
            ],
        );
        let b = (
            "B".into(),
            vec![
                (NaiveDate::from_ymd_opt(2026, 1, 3).unwrap(), 0.005),
                (NaiveDate::from_ymd_opt(2026, 1, 4).unwrap(), -0.01),
            ],
        );
        let (dates, aligned) = align_daily_returns(&[a, b]).unwrap();
        assert_eq!(dates.len(), 2); // Only common dates
        assert_eq!(aligned.len(), 2);
        assert_eq!(aligned[0].len(), 2);
        assert_eq!(aligned[1].len(), 2);
    }
}
