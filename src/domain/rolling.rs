//! 滚动指标与净值/回撤时间序列（纯计算）。

use crate::models::{FundAnalysisSeries, FundNav, PortfolioTimeSeries, SeriesPoint};
use chrono::NaiveDate;
use std::collections::HashMap;

use super::BenchmarkData;
use super::returns::{daily_returns, nav_price};

/// 默认滚动窗口（交易日，约 3 个月）。
pub const DEFAULT_ROLLING_WINDOW: u32 = 60;
/// 滚动计算所需最少样本。
const MIN_ROLLING_SAMPLES: usize = 10;
/// 滚动窗口上限（交易日）。
pub const MAX_ROLLING_WINDOW: u32 = 252;
const RISK_FREE_RATE: f64 = 0.03;
const TRADING_DAYS: f64 = 252.0;

/// 将请求的滚动窗口规范到 [10, 252]。
pub fn normalize_rolling_window(requested: u32) -> usize {
    requested.clamp(MIN_ROLLING_SAMPLES as u32, MAX_ROLLING_WINDOW) as usize
}

/// 归一化净值曲线（起点 = 1.0）。
pub fn normalized_nav_curve(navs: &[FundNav]) -> Vec<SeriesPoint> {
    let mut sorted: Vec<&FundNav> = navs.iter().collect();
    sorted.sort_by_key(|n| n.date);
    if sorted.is_empty() {
        return Vec::new();
    }
    let start = nav_price(sorted[0]);
    if start <= 0.0 {
        return Vec::new();
    }

    let mut out = vec![SeriesPoint {
        date: sorted[0].date,
        value: 1.0,
    }];
    let mut price = 1.0;
    for i in 1..sorted.len() {
        let prev = nav_price(sorted[i - 1]);
        let curr = nav_price(sorted[i]);
        let r = if prev > 0.0 {
            (curr - prev) / prev
        } else {
            sorted[i].daily_return.unwrap_or(0.0)
        };
        price *= 1.0 + r;
        out.push(SeriesPoint {
            date: sorted[i].date,
            value: price,
        });
    }
    out
}

/// 由归一化净值曲线计算逐日回撤（≤ 0）。
pub fn drawdown_series(curve: &[SeriesPoint]) -> Vec<SeriesPoint> {
    if curve.is_empty() {
        return Vec::new();
    }
    let mut peak = curve[0].value;
    curve
        .iter()
        .map(|p| {
            if p.value > peak {
                peak = p.value;
            }
            let dd = if peak > 0.0 {
                (p.value - peak) / peak
            } else {
                0.0
            };
            SeriesPoint {
                date: p.date,
                value: dd,
            }
        })
        .collect()
}

fn effective_window(requested: usize, sample_len: usize) -> Option<usize> {
    if sample_len < MIN_ROLLING_SAMPLES {
        return None;
    }
    let w = requested.min(sample_len);
    if w >= MIN_ROLLING_SAMPLES {
        Some(w)
    } else {
        None
    }
}

fn window_volatility(window: &[f64]) -> f64 {
    if window.len() < 2 {
        return 0.0;
    }
    let mean = window.iter().sum::<f64>() / window.len() as f64;
    let variance =
        window.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (window.len() - 1) as f64;
    variance.sqrt() * TRADING_DAYS.sqrt()
}

fn window_sharpe(window: &[f64]) -> f64 {
    let vol = window_volatility(window);
    if vol <= 0.0 || !vol.is_finite() {
        return 0.0;
    }
    let mean = window.iter().sum::<f64>() / window.len() as f64;
    let ann_return = mean * TRADING_DAYS;
    (ann_return - RISK_FREE_RATE) / vol
}

fn window_beta(fund: &[f64], bm: &[f64]) -> f64 {
    if fund.len() < 2 || fund.len() != bm.len() {
        return 0.0;
    }
    let fund_mean = fund.iter().sum::<f64>() / fund.len() as f64;
    let bm_mean = bm.iter().sum::<f64>() / bm.len() as f64;
    let covariance = fund
        .iter()
        .zip(bm.iter())
        .map(|(f, b)| (f - fund_mean) * (b - bm_mean))
        .sum::<f64>()
        / (fund.len() - 1) as f64;
    let bm_variance = bm.iter().map(|b| (b - bm_mean).powi(2)).sum::<f64>() / (bm.len() - 1) as f64;
    if bm_variance > 0.0 {
        covariance / bm_variance
    } else {
        0.0
    }
}

/// 滚动年化波动率序列。
pub fn rolling_volatility_series(returns: &[(NaiveDate, f64)], window: usize) -> Vec<SeriesPoint> {
    let w = match effective_window(window, returns.len()) {
        Some(w) => w,
        None => return Vec::new(),
    };
    (w - 1..returns.len())
        .map(|i| {
            let slice: Vec<f64> = returns[i + 1 - w..=i].iter().map(|(_, r)| *r).collect();
            SeriesPoint {
                date: returns[i].0,
                value: window_volatility(&slice),
            }
        })
        .collect()
}

/// 滚动夏普比率序列。
pub fn rolling_sharpe_series(returns: &[(NaiveDate, f64)], window: usize) -> Vec<SeriesPoint> {
    let w = match effective_window(window, returns.len()) {
        Some(w) => w,
        None => return Vec::new(),
    };
    (w - 1..returns.len())
        .map(|i| {
            let slice: Vec<f64> = returns[i + 1 - w..=i].iter().map(|(_, r)| *r).collect();
            SeriesPoint {
                date: returns[i].0,
                value: window_sharpe(&slice),
            }
        })
        .collect()
}

fn align_fund_benchmark(
    fund: &[(NaiveDate, f64)],
    benchmark: &BenchmarkData,
) -> (Vec<NaiveDate>, Vec<f64>, Vec<f64>) {
    let bm_map: HashMap<NaiveDate, f64> = benchmark
        .dates
        .iter()
        .zip(benchmark.returns.iter())
        .map(|(d, r)| (*d, *r))
        .collect();
    let mut dates = Vec::new();
    let mut fund_r = Vec::new();
    let mut bm_r = Vec::new();
    for (d, r) in fund {
        if let Some(&br) = bm_map.get(d) {
            dates.push(*d);
            fund_r.push(*r);
            bm_r.push(br);
        }
    }
    (dates, fund_r, bm_r)
}

/// 滚动 Beta 序列（与基准按日期对齐）。
pub fn rolling_beta_series(
    fund: &[(NaiveDate, f64)],
    benchmark: &BenchmarkData,
    window: usize,
) -> Vec<SeriesPoint> {
    let (dates, fund_r, bm_r) = align_fund_benchmark(fund, benchmark);
    let w = match effective_window(window, dates.len()) {
        Some(w) => w,
        None => return Vec::new(),
    };
    (w - 1..dates.len())
        .map(|i| {
            let f_slice = &fund_r[i + 1 - w..=i];
            let b_slice = &bm_r[i + 1 - w..=i];
            SeriesPoint {
                date: dates[i],
                value: window_beta(f_slice, b_slice),
            }
        })
        .collect()
}

/// 由净值序列构建单基金时间序列。
pub fn build_fund_analysis_series(
    navs: &[FundNav],
    benchmark: Option<&BenchmarkData>,
    window: usize,
) -> Option<FundAnalysisSeries> {
    let returns = daily_returns(navs);
    let effective = effective_window(window, returns.len())?;
    let nav_normalized = normalized_nav_curve(navs);
    if nav_normalized.is_empty() {
        return None;
    }
    let drawdown = drawdown_series(&nav_normalized);
    let rolling_volatility = rolling_volatility_series(&returns, effective);
    let rolling_sharpe = rolling_sharpe_series(&returns, effective);
    let rolling_beta = benchmark
        .map(|bm| rolling_beta_series(&returns, bm, effective))
        .unwrap_or_default();

    Some(FundAnalysisSeries {
        rolling_window: effective as u32,
        nav_normalized,
        drawdown,
        rolling_sharpe,
        rolling_beta,
        rolling_volatility,
    })
}

/// 由对齐后的组合日收益构建时间序列。
pub fn build_portfolio_series(
    dates: &[NaiveDate],
    daily: &[f64],
    window: usize,
) -> Option<PortfolioTimeSeries> {
    if dates.len() != daily.len() || dates.is_empty() {
        return None;
    }
    let returns: Vec<(NaiveDate, f64)> = dates
        .iter()
        .zip(daily.iter())
        .map(|(d, r)| (*d, *r))
        .collect();
    let effective = effective_window(window, returns.len())?;

    let mut price = 1.0;
    let start_date = dates[0].pred_opt().unwrap_or(dates[0]);
    let mut nav_normalized = vec![SeriesPoint {
        date: start_date,
        value: 1.0,
    }];
    for (d, r) in dates.iter().zip(daily.iter()) {
        price *= 1.0 + r;
        nav_normalized.push(SeriesPoint {
            date: *d,
            value: price,
        });
    }

    let drawdown = drawdown_series(&nav_normalized);
    let rolling_volatility = rolling_volatility_series(&returns, effective);
    let rolling_sharpe = rolling_sharpe_series(&returns, effective);

    Some(PortfolioTimeSeries {
        rolling_window: effective as u32,
        nav_normalized,
        drawdown,
        rolling_sharpe,
        rolling_volatility,
    })
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

    fn make_upward_navs(n: usize) -> Vec<FundNav> {
        (0..n)
            .map(|i| {
                let d =
                    NaiveDate::from_ymd_opt(2026, 1, 1).unwrap() + chrono::Duration::days(i as i64);
                nav(
                    "000001",
                    &d.format("%Y-%m-%d").to_string(),
                    1.0 + i as f64 * 0.001,
                )
            })
            .collect()
    }

    #[test]
    fn normalized_nav_starts_at_one() {
        let navs = vec![
            nav("000001", "2026-01-01", 1.0),
            nav("000001", "2026-01-02", 1.1),
        ];
        let curve = normalized_nav_curve(&navs);
        assert_eq!(curve.len(), 2);
        assert!((curve[0].value - 1.0).abs() < 1e-9);
        assert!((curve[1].value - 1.1).abs() < 1e-6);
    }

    #[test]
    fn drawdown_non_positive() {
        let curve = vec![
            SeriesPoint {
                date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                value: 1.0,
            },
            SeriesPoint {
                date: NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(),
                value: 1.2,
            },
            SeriesPoint {
                date: NaiveDate::from_ymd_opt(2026, 1, 3).unwrap(),
                value: 1.0,
            },
        ];
        let dd = drawdown_series(&curve);
        assert!(dd[2].value <= 0.0);
        assert!((dd[2].value - (-1.0 / 6.0)).abs() < 1e-6);
    }

    #[test]
    fn rolling_volatility_produces_points() {
        let navs = make_upward_navs(65);
        let series =
            build_fund_analysis_series(&navs, None, DEFAULT_ROLLING_WINDOW as usize).unwrap();
        assert!(!series.rolling_volatility.is_empty());
        assert_eq!(series.rolling_window, 60);
    }

    #[test]
    fn portfolio_series_from_daily() {
        let dates: Vec<NaiveDate> = (0..65)
            .map(|i| {
                NaiveDate::from_ymd_opt(2026, 1, 1).unwrap() + chrono::Duration::days(i as i64)
            })
            .collect();
        let daily = vec![0.001; 65];
        let s = build_portfolio_series(&dates, &daily, DEFAULT_ROLLING_WINDOW as usize).unwrap();
        assert!(!s.nav_normalized.is_empty());
        assert!(!s.rolling_sharpe.is_empty());
    }

    #[test]
    fn normalize_rolling_window_clamps() {
        assert_eq!(normalize_rolling_window(5), 10);
        assert_eq!(normalize_rolling_window(60), 60);
        assert_eq!(normalize_rolling_window(500), 252);
    }
}
