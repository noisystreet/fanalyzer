//! 组合解读阈值配置（TOML，缺失时使用默认值）。

use serde::Deserialize;
use std::path::Path;

/// 解读规则阈值。
#[derive(Debug, Clone, Deserialize)]
pub struct PortfolioInsightThresholds {
    #[serde(default = "default_high_correlation")]
    pub high_correlation: f64,
    #[serde(default = "default_elevated_correlation")]
    pub elevated_correlation: f64,
    #[serde(default = "default_high_overlap_pct")]
    pub high_overlap_pct: f64,
    #[serde(default = "default_elevated_overlap_pct")]
    pub elevated_overlap_pct: f64,
    #[serde(default = "default_concentrated_weight")]
    pub concentrated_weight: f64,
    #[serde(default = "default_drawdown_caution")]
    pub drawdown_caution: f64,
    #[serde(default = "default_sharpe_good")]
    pub sharpe_good: f64,
    #[serde(default = "default_sharpe_weak")]
    pub sharpe_weak: f64,
    #[serde(default = "default_aligned_days_ratio_caution")]
    pub aligned_days_ratio_caution: f64,
    #[serde(default = "default_equal_weight_sharpe_delta")]
    pub equal_weight_sharpe_delta: f64,
}

#[derive(Debug, Deserialize)]
struct InsightConfigToml {
    #[serde(default)]
    thresholds: PortfolioInsightThresholds,
}

impl Default for PortfolioInsightThresholds {
    fn default() -> Self {
        Self {
            high_correlation: default_high_correlation(),
            elevated_correlation: default_elevated_correlation(),
            high_overlap_pct: default_high_overlap_pct(),
            elevated_overlap_pct: default_elevated_overlap_pct(),
            concentrated_weight: default_concentrated_weight(),
            drawdown_caution: default_drawdown_caution(),
            sharpe_good: default_sharpe_good(),
            sharpe_weak: default_sharpe_weak(),
            aligned_days_ratio_caution: default_aligned_days_ratio_caution(),
            equal_weight_sharpe_delta: default_equal_weight_sharpe_delta(),
        }
    }
}

fn default_high_correlation() -> f64 {
    0.85
}
fn default_elevated_correlation() -> f64 {
    0.70
}
fn default_high_overlap_pct() -> f64 {
    15.0
}
fn default_elevated_overlap_pct() -> f64 {
    8.0
}
fn default_concentrated_weight() -> f64 {
    0.40
}
fn default_drawdown_caution() -> f64 {
    0.20
}
fn default_sharpe_good() -> f64 {
    1.0
}
fn default_sharpe_weak() -> f64 {
    0.5
}
fn default_aligned_days_ratio_caution() -> f64 {
    0.50
}
fn default_equal_weight_sharpe_delta() -> f64 {
    0.05
}

/// 读取解读阈值；文件不存在或解析失败时回退默认值。
pub fn load_portfolio_insights(path: &Path) -> PortfolioInsightThresholds {
    if !path.exists() {
        return PortfolioInsightThresholds::default();
    }
    match std::fs::read_to_string(path) {
        Ok(raw) => match toml::from_str::<InsightConfigToml>(&raw) {
            Ok(cfg) => cfg.thresholds,
            Err(e) => {
                tracing::warn!(path = %path.display(), error = %e, "解读阈值配置解析失败，使用默认值");
                PortfolioInsightThresholds::default()
            }
        },
        Err(e) => {
            tracing::warn!(path = %path.display(), error = %e, "解读阈值配置读取失败，使用默认值");
            PortfolioInsightThresholds::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn load_missing_uses_defaults() {
        let t = load_portfolio_insights(Path::new("/nonexistent/portfolio_insights.toml"));
        assert!((t.high_correlation - 0.85).abs() < 1e-9);
    }

    #[test]
    fn load_custom_thresholds() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(
            f,
            r#"
[thresholds]
high_correlation = 0.90
concentrated_weight = 0.35
"#
        )
        .unwrap();
        let t = load_portfolio_insights(f.path());
        assert!((t.high_correlation - 0.90).abs() < 1e-9);
        assert!((t.concentrated_weight - 0.35).abs() < 1e-9);
        assert!((t.sharpe_good - 1.0).abs() < 1e-9);
    }
}
