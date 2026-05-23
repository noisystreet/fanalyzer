//! 分析结果排序键（compare / screen 共用）。

use crate::models::FundAnalysis;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnalysisSortKey {
    Code,
    TotalReturn,
    AnnualizedReturn,
    Sharpe,
    Sortino,
    Calmar,
    MaxDrawdown,
    Alpha,
    Volatility,
}

impl AnalysisSortKey {
    /// 默认是否降序（收益类降序，风险类升序）。
    pub fn default_desc(self) -> bool {
        !matches!(self, Self::MaxDrawdown | Self::Volatility | Self::Code)
    }

    pub fn metric_value(self, a: &FundAnalysis) -> f64 {
        match self {
            Self::Code => 0.0,
            Self::TotalReturn => a.total_return,
            Self::AnnualizedReturn => a.annualized_return,
            Self::Sharpe => a.sharpe_ratio,
            Self::Sortino => a.sortino_ratio,
            Self::Calmar => a.calmar_ratio,
            Self::MaxDrawdown => a.max_drawdown,
            Self::Alpha => a.alpha,
            Self::Volatility => a.volatility,
        }
    }
}

pub fn parse_sort_key(raw: &str) -> anyhow::Result<AnalysisSortKey> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "code" => Ok(AnalysisSortKey::Code),
        "total-return" | "return" | "total_return" => Ok(AnalysisSortKey::TotalReturn),
        "annualized-return" | "annualized" | "annualized_return" => {
            Ok(AnalysisSortKey::AnnualizedReturn)
        }
        "sharpe" | "sharpe-ratio" => Ok(AnalysisSortKey::Sharpe),
        "sortino" | "sortino-ratio" => Ok(AnalysisSortKey::Sortino),
        "calmar" | "calmar-ratio" => Ok(AnalysisSortKey::Calmar),
        "max-drawdown" | "drawdown" | "max_drawdown" => Ok(AnalysisSortKey::MaxDrawdown),
        "alpha" => Ok(AnalysisSortKey::Alpha),
        "volatility" | "vol" => Ok(AnalysisSortKey::Volatility),
        other => anyhow::bail!(
            "未知 sort 键 `{other}`；可用 sharpe/sortino/calmar/total-return/max-drawdown/alpha/volatility"
        ),
    }
}

pub fn sort_analyses(analyses: &mut [FundAnalysis], key: AnalysisSortKey, desc: bool) {
    analyses.sort_by(|a, b| {
        let cmp = key
            .metric_value(a)
            .partial_cmp(&key.metric_value(b))
            .unwrap_or(std::cmp::Ordering::Equal);
        if key == AnalysisSortKey::Code {
            a.code.cmp(&b.code)
        } else if desc {
            cmp.reverse()
        } else {
            cmp
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::FundAnalysis;

    fn sample(code: &str, sharpe: f64, dd: f64) -> FundAnalysis {
        FundAnalysis {
            code: code.into(),
            name: code.into(),
            period_days: 90,
            avg_nav: 1.0,
            max_nav: 1.0,
            min_nav: 1.0,
            total_return: sharpe * 0.1,
            annualized_return: 0.1,
            volatility: 0.1,
            max_drawdown: dd,
            sharpe_ratio: sharpe,
            sortino_ratio: sharpe,
            calmar_ratio: sharpe,
            alpha: 0.0,
            beta: 1.0,
            manager_name: String::new(),
            manager_tenure_days: 0,
            manager_total_return: 0.0,
            management_fee: 0.0,
            custody_fee: 0.0,
        }
    }

    #[test]
    fn sort_sharpe_desc() {
        let mut v = vec![sample("b", 0.5, 0.1), sample("a", 1.2, 0.1)];
        sort_analyses(&mut v, AnalysisSortKey::Sharpe, true);
        assert_eq!(v[0].code, "a");
    }
}
