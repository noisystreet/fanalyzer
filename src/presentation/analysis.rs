//! 单基金分析 JSON 导出。

use crate::models::FundAnalysisReport;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub fn export_analysis_json(report: &FundAnalysisReport, path: &Path) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(report)?;
    let mut file = File::create(path)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

pub fn render_analysis(
    report: &FundAnalysisReport,
    output: Option<&Path>,
    format: &str,
) -> anyhow::Result<()> {
    if let Some(path) = output {
        match format.to_lowercase().as_str() {
            "json" => export_analysis_json(report, path),
            other => anyhow::bail!("analyze 导出格式 `{other}` 不支持，请使用 json"),
        }
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{FundAnalysis, FundAnalysisSeries, SeriesPoint};
    use chrono::NaiveDate;

    fn sample_report() -> FundAnalysisReport {
        FundAnalysisReport {
            snapshot: FundAnalysis {
                code: "000001".into(),
                name: "测试".into(),
                period_days: 90,
                avg_nav: 1.0,
                max_nav: 1.1,
                min_nav: 0.9,
                total_return: 0.05,
                annualized_return: 0.08,
                volatility: 0.12,
                max_drawdown: 0.06,
                sharpe_ratio: 1.2,
                sortino_ratio: 1.3,
                calmar_ratio: 1.1,
                alpha: 0.01,
                beta: 0.95,
                manager_name: String::new(),
                manager_tenure_days: 0,
                manager_total_return: 0.0,
                management_fee: 0.0,
                custody_fee: 0.0,
            },
            series: Some(FundAnalysisSeries {
                rolling_window: 60,
                nav_normalized: vec![SeriesPoint {
                    date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                    value: 1.0,
                }],
                drawdown: vec![],
                rolling_sharpe: vec![],
                rolling_beta: vec![],
                rolling_volatility: vec![],
            }),
            benchmark_label: Some("沪深300".into()),
        }
    }

    #[test]
    fn export_json_contains_series() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("analysis.json");
        export_analysis_json(&sample_report(), &path).unwrap();
        let text = std::fs::read_to_string(path).unwrap();
        assert!(text.contains("nav_normalized"));
        assert!(text.contains("rolling_sharpe"));
    }
}
