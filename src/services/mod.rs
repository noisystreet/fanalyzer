use crate::models::{FundAnalysis, FundNav};

pub struct FundAnalyzer;

#[derive(Debug, Clone)]
pub struct BenchmarkData {
    pub dates: Vec<chrono::NaiveDate>,
    pub returns: Vec<f64>,
}

impl FundAnalyzer {
    pub fn analyze(
        navs: &[FundNav],
        period_days: u32,
        name: &str,
        benchmark: Option<&BenchmarkData>,
    ) -> Option<FundAnalysis> {
        if navs.is_empty() {
            return None;
        }

        let code = navs[0].code.clone();

        let mut sorted: Vec<&FundNav> = navs.iter().collect();
        sorted.sort_by_key(|n| n.date);

        let nav_values: Vec<f64> = sorted.iter().map(|n| n.nav).collect();
        let avg_nav = nav_values.iter().sum::<f64>() / nav_values.len() as f64;
        let max_nav = nav_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let min_nav = nav_values.iter().cloned().fold(f64::INFINITY, f64::min);

        let total_return = if sorted.len() >= 2 {
            let first = sorted.first()?.nav;
            let last = sorted.last()?.nav;
            if first == 0.0 {
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

        let volatility = Self::calc_volatility(&sorted);
        let max_drawdown = Self::calc_max_drawdown(&nav_values);
        let sharpe_ratio = Self::calc_sharpe_ratio(annualized_return, volatility);

        let (aligned_fund_returns, aligned_bm_returns) = if let Some(bm) = benchmark {
            Self::align_returns_by_date(&sorted, bm)
        } else {
            (Vec::new(), Vec::new())
        };

        let (alpha, beta) = if !aligned_fund_returns.is_empty() {
            Self::calc_alpha_beta(&aligned_fund_returns, &aligned_bm_returns)
        } else {
            (0.0, 0.0)
        };

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
            alpha,
            beta,
        })
    }

    fn align_returns_by_date(navs: &[&FundNav], benchmark: &BenchmarkData) -> (Vec<f64>, Vec<f64>) {
        use std::collections::HashMap;

        let bm_map: HashMap<chrono::NaiveDate, f64> = benchmark
            .dates
            .iter()
            .zip(benchmark.returns.iter())
            .map(|(d, r)| (*d, *r))
            .collect();

        let mut fund_returns = Vec::new();
        let mut bm_returns = Vec::new();

        for nav in navs {
            if let Some(&bm_return) = bm_map.get(&nav.date) {
                if let Some(fund_return) = nav.daily_return {
                    fund_returns.push(fund_return);
                    bm_returns.push(bm_return);
                }
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
        let alpha_daily =
            fund_mean - (RISK_FREE_RATE_DAILY + beta * (bm_mean - RISK_FREE_RATE_DAILY));
        let alpha_annualized = alpha_daily * 252.0;

        (alpha_annualized, beta)
    }

    fn calc_volatility(navs: &[&FundNav]) -> f64 {
        let returns: Vec<f64> = navs.iter().filter_map(|n| n.daily_return).collect();

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
    fn test_analyze_empty() {
        let result = FundAnalyzer::analyze(&[], 30, "测试基金", None);
        assert!(result.is_none());
    }

    #[test]
    fn test_analyze_single_nav() {
        let navs = vec![make_nav("000001", "2026-01-01", 1.0, 1.0, None)];
        let result = FundAnalyzer::analyze(&navs, 1, "测试基金", None).unwrap();
        assert_eq!(result.code, "000001");
        assert_eq!(result.name, "测试基金");
        assert!((result.avg_nav - 1.0).abs() < 1e-6);
        assert!((result.max_nav - 1.0).abs() < 1e-6);
        assert!((result.min_nav - 1.0).abs() < 1e-6);
        assert!((result.total_return - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_total_return_positive() {
        let navs = vec![
            make_nav("000001", "2026-01-10", 1.2, 1.2, Some(0.0435)),
            make_nav("000001", "2026-01-05", 1.1, 1.1, Some(0.1)),
            make_nav("000001", "2026-01-01", 1.0, 1.0, None),
        ];
        let result = FundAnalyzer::analyze(&navs, 10, "测试基金", None).unwrap();
        assert!((result.total_return - 0.2).abs() < 1e-6);
    }

    #[test]
    fn test_total_return_negative() {
        let navs = vec![
            make_nav("000001", "2026-01-10", 0.8, 0.8, Some(-0.1)),
            make_nav("000001", "2026-01-05", 0.9, 0.9, Some(-0.1)),
            make_nav("000001", "2026-01-01", 1.0, 1.0, None),
        ];
        let result = FundAnalyzer::analyze(&navs, 10, "测试基金", None).unwrap();
        assert!((result.total_return - (-0.2)).abs() < 1e-6);
    }

    #[test]
    fn test_max_drawdown() {
        let navs = vec![
            make_nav("000001", "2026-01-01", 1.0, 1.0, None),
            make_nav("000001", "2026-01-05", 0.7, 0.7, None),
            make_nav("000001", "2026-01-10", 0.9, 0.9, None),
        ];
        let result = FundAnalyzer::analyze(&navs, 10, "测试基金", None).unwrap();
        assert!((result.max_drawdown - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_max_drawdown_no_drawdown() {
        let navs = vec![
            make_nav("000001", "2026-01-01", 1.0, 1.0, None),
            make_nav("000001", "2026-01-05", 1.2, 1.2, None),
            make_nav("000001", "2026-01-10", 1.3, 1.3, None),
        ];
        let result = FundAnalyzer::analyze(&navs, 10, "测试基金", None).unwrap();
        assert!((result.max_drawdown - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_max_drawdown_reverse_order() {
        let navs = vec![
            make_nav("000001", "2026-01-10", 0.9, 0.9, None),
            make_nav("000001", "2026-01-05", 0.7, 0.7, None),
            make_nav("000001", "2026-01-01", 1.0, 1.0, None),
        ];
        let result = FundAnalyzer::analyze(&navs, 10, "测试基金", None).unwrap();
        assert!((result.max_drawdown - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_volatility_zero() {
        let navs = vec![
            make_nav("000001", "2026-01-05", 1.0, 1.0, Some(0.0)),
            make_nav("000001", "2026-01-01", 1.0, 1.0, None),
        ];
        let result = FundAnalyzer::analyze(&navs, 5, "测试基金", None).unwrap();
        assert!((result.volatility - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_avg_max_min_nav() {
        let navs = vec![
            make_nav("000001", "2026-01-10", 1.5, 1.5, None),
            make_nav("000001", "2026-01-05", 0.5, 0.5, None),
            make_nav("000001", "2026-01-01", 1.0, 1.0, None),
        ];
        let result = FundAnalyzer::analyze(&navs, 10, "测试基金", None).unwrap();
        assert!((result.avg_nav - 1.0).abs() < 1e-6);
        assert!((result.max_nav - 1.5).abs() < 1e-6);
        assert!((result.min_nav - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_sharpe_ratio() {
        let sharpe = FundAnalyzer::calc_sharpe_ratio(0.15, 0.2);
        assert!((sharpe - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_sharpe_ratio_zero_volatility() {
        let sharpe = FundAnalyzer::calc_sharpe_ratio(0.15, 0.0);
        assert!((sharpe - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_sharpe_ratio_negative_return() {
        let sharpe = FundAnalyzer::calc_sharpe_ratio(-0.05, 0.1);
        assert!((sharpe - (-0.8)).abs() < 1e-6);
    }
}
