//! 排行筛选规则（纯函数）。

use crate::models::FundAnalysis;

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

    #[test]
    fn screen_default_passes_all() {
        let f = ScreenFilters::default();
        assert!(passes_screen(&sample(0.5, -10.0, -1.0, 0.5, -0.5), &f));
    }

    #[test]
    fn screen_rejects_drawdown() {
        let f = ScreenFilters {
            max_drawdown_pct: Some(10.0),
            ..Default::default()
        };
        assert!(passes_screen(&sample(0.08, 0.0, 0.0, 0.0, 0.0), &f));
        assert!(!passes_screen(&sample(0.15, 0.0, 0.0, 0.0, 0.0), &f));
    }

    #[test]
    fn screen_rejects_sharpe() {
        let f = ScreenFilters {
            min_sharpe: Some(1.0),
            ..Default::default()
        };
        assert!(passes_screen(&sample(0.0, 1.5, 0.0, 0.0, 0.0), &f));
        assert!(!passes_screen(&sample(0.0, 0.5, 0.0, 0.0, 0.0), &f));
    }

    #[test]
    fn screen_rejects_volatility() {
        let f = ScreenFilters {
            max_volatility_pct: Some(20.0),
            ..Default::default()
        };
        assert!(passes_screen(&sample(0.0, 0.0, 0.0, 0.18, 0.0), &f));
        assert!(!passes_screen(&sample(0.0, 0.0, 0.0, 0.25, 0.0), &f));
    }

    #[test]
    fn screen_rejects_return() {
        let f = ScreenFilters {
            min_total_return_pct: Some(10.0),
            ..Default::default()
        };
        assert!(passes_screen(&sample(0.0, 0.0, 0.0, 0.0, 0.15), &f));
        assert!(!passes_screen(&sample(0.0, 0.0, 0.0, 0.0, 0.05), &f));
    }

    #[test]
    fn screen_all_filters_combined_boundary() {
        let f = ScreenFilters {
            max_drawdown_pct: Some(10.0),
            min_sharpe: Some(0.5),
            min_alpha_pct: Some(0.0),
            max_volatility_pct: Some(20.0),
            min_total_return_pct: Some(5.0),
            ..Default::default()
        };
        // Exactly at boundary values
        let a = sample(0.10, 0.5, 0.0, 0.20, 0.05);
        assert!(passes_screen(&a, &f));

        // Slightly above max_drawdown threshold
        assert!(!passes_screen(&sample(0.101, 0.5, 0.0, 0.20, 0.05), &f));
        // Slightly below min_sharpe threshold
        assert!(!passes_screen(&sample(0.10, 0.499, 0.0, 0.20, 0.05), &f));
    }
}
