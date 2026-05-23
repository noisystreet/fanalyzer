//! 对比结果终端输出与 CSV/JSON 导出。

use crate::models::FundAnalysis;
use crate::presentation::print_comparison;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub fn render_comparison(
    analyses: &[FundAnalysis],
    output: Option<&Path>,
    format: &str,
) -> anyhow::Result<()> {
    if analyses.len() < 2 {
        return Ok(());
    }
    print_comparison(analyses);

    if let Some(path) = output {
        match format.trim().to_ascii_lowercase().as_str() {
            "json" => export_comparison_json(analyses, path)?,
            _ => export_comparison_csv(analyses, path)?,
        }
        tracing::info!(path = %path.display(), "Wrote comparison export");
    }
    Ok(())
}

pub fn export_comparison_csv(analyses: &[FundAnalysis], path: &Path) -> anyhow::Result<()> {
    let mut w = csv::Writer::from_path(path)?;
    w.write_record([
        "code",
        "name",
        "period_days",
        "total_return_pct",
        "annualized_return_pct",
        "volatility_pct",
        "max_drawdown_pct",
        "sharpe_ratio",
        "sortino_ratio",
        "calmar_ratio",
        "alpha_pct",
        "beta",
        "management_fee_pct",
        "custody_fee_pct",
        "manager_name",
        "manager_tenure_days",
        "manager_total_return_pct",
    ])?;
    for a in analyses {
        w.write_record([
            a.code.clone(),
            a.name.clone(),
            a.period_days.to_string(),
            pct(a.total_return),
            pct(a.annualized_return),
            pct(a.volatility),
            pct(a.max_drawdown),
            fmt(a.sharpe_ratio),
            fmt(a.sortino_ratio),
            fmt(a.calmar_ratio),
            pct(a.alpha),
            fmt(a.beta),
            fmt(a.management_fee),
            fmt(a.custody_fee),
            a.manager_name.clone(),
            a.manager_tenure_days.to_string(),
            pct(a.manager_total_return),
        ])?;
    }
    w.flush()?;
    Ok(())
}

pub fn export_comparison_json(analyses: &[FundAnalysis], path: &Path) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(analyses)?;
    let mut file = File::create(path)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

fn pct(v: f64) -> String {
    format!("{:.4}", v * 100.0)
}

fn fmt(v: f64) -> String {
    format!("{:.4}", v)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::FundAnalysis;
    use std::io::Read;

    fn sample() -> FundAnalysis {
        FundAnalysis {
            code: "000001".into(),
            name: "t".into(),
            period_days: 30,
            avg_nav: 1.0,
            max_nav: 1.0,
            min_nav: 1.0,
            total_return: 0.05,
            annualized_return: 0.1,
            volatility: 0.12,
            max_drawdown: 0.08,
            sharpe_ratio: 1.0,
            sortino_ratio: 1.2,
            calmar_ratio: 1.25,
            alpha: 0.02,
            beta: 0.9,
            manager_name: "m".into(),
            manager_tenure_days: 365,
            manager_total_return: 0.2,
            management_fee: 1.5,
            custody_fee: 0.2,
        }
    }

    #[test]
    fn export_csv_writes_header() {
        let dir = std::env::temp_dir().join(format!("af_cmp_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("cmp.csv");
        export_comparison_csv(&[sample(), sample()], &path).unwrap();
        let mut s = String::new();
        std::fs::File::open(&path)
            .unwrap()
            .read_to_string(&mut s)
            .unwrap();
        assert!(s.contains("sortino_ratio"));
        let _ = std::fs::remove_dir_all(dir);
    }
}
