//! 组合分析终端输出与导出。

use crate::models::{CorrelationMatrix, OverlapPair, PortfolioInterpretation, PortfolioReport};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tabled::settings::{object::Columns, Alignment, Style};
use tabled::{Table, Tabled};

pub fn render_portfolio(
    report: &PortfolioReport,
    output: Option<&Path>,
    _format: &str,
) -> anyhow::Result<()> {
    print_portfolio_report(report);

    if let Some(path) = output {
        export_portfolio_json(report, path)?;
        tracing::info!(path = %path.display(), "Wrote portfolio export");
    }
    Ok(())
}

pub fn print_portfolio_report(report: &PortfolioReport) {
    let s = &report.summary;
    println!("组合分析报告");
    println!("组合名称: {}", s.name);
    println!(
        "分析窗口: {} 天（对齐交易日 {} 天）",
        s.period_days, s.aligned_days
    );
    println!("组合总收益率: {:.2}%", s.total_return * 100.0);
    println!("组合年化收益率: {:.2}%", s.annualized_return * 100.0);
    println!("组合波动率: {:.2}%", s.volatility * 100.0);
    println!("组合最大回撤: {:.2}%", s.max_drawdown * 100.0);
    println!("组合夏普比率: {:.2}", s.sharpe_ratio);
    println!();

    if let Some(ref interp) = report.interpretation {
        print_portfolio_interpretation(interp);
        println!();
    }

    print_members_table(s);
    println!();
    print_correlation_table(&report.correlation);
    if !report.overlaps.is_empty() {
        println!();
        print_overlap_table(&report.overlaps);
    }
}

pub fn print_portfolio_interpretation(interp: &PortfolioInterpretation) {
    println!("── 分析解读 ──");
    println!("{}", interp.headline);
    for item in &interp.insights {
        let tag = match item.level {
            crate::models::InsightLevel::Positive => "✓",
            crate::models::InsightLevel::Info => "·",
            crate::models::InsightLevel::Caution => "!",
        };
        println!("  [{tag}] {}", item.message);
    }
}

#[derive(Tabled)]
struct MemberRow {
    #[tabled(rename = "代码")]
    code: String,
    #[tabled(rename = "名称")]
    name: String,
    #[tabled(rename = "权重")]
    weight: String,
    #[tabled(rename = "总收益")]
    total_return: String,
    #[tabled(rename = "波动率")]
    volatility: String,
    #[tabled(rename = "最大回撤")]
    max_drawdown: String,
    #[tabled(rename = "夏普")]
    sharpe: String,
    #[tabled(rename = "收益贡献")]
    contribution: String,
}

fn print_members_table(s: &crate::models::PortfolioSummary) {
    println!("成分基金与静态贡献（weight × 单基总收益）");
    let rows: Vec<MemberRow> = s
        .members
        .iter()
        .map(|m| MemberRow {
            code: m.code.clone(),
            name: m.name.clone(),
            weight: pct_ratio(m.weight),
            total_return: pct(m.total_return),
            volatility: pct(m.volatility),
            max_drawdown: pct(m.max_drawdown),
            sharpe: fmt(m.sharpe_ratio),
            contribution: pct(m.return_contribution),
        })
        .collect();
    let mut table = Table::new(rows);
    table
        .with(Style::rounded())
        .modify(Columns::new(..), Alignment::right());
    println!("{table}");
}

fn print_correlation_table(matrix: &CorrelationMatrix) {
    println!("日收益 Pearson 相关矩阵");
    print!("{:>8}", "");
    for label in &matrix.labels {
        print!(" {:>8}", label);
    }
    println!();
    for (i, row) in matrix.values.iter().enumerate() {
        print!("{:>8}", matrix.labels[i]);
        for v in row {
            print!(" {:>8.3}", v);
        }
        println!();
    }
}

#[derive(Tabled)]
struct OverlapRow {
    #[tabled(rename = "基金A")]
    fund_a: String,
    #[tabled(rename = "基金B")]
    fund_b: String,
    #[tabled(rename = "加权重叠%")]
    overlap: String,
    #[tabled(rename = "共同持仓数")]
    shared: String,
}

fn print_overlap_table(pairs: &[OverlapPair]) {
    println!("重仓股加权重叠（前 N 大重仓，min(占净值%) 之和）");
    let rows: Vec<OverlapRow> = pairs
        .iter()
        .map(|p| OverlapRow {
            fund_a: format!("{} {}", p.fund_a_code, p.fund_a_name),
            fund_b: format!("{} {}", p.fund_b_code, p.fund_b_name),
            overlap: format!("{:.2}", p.overlap_pct),
            shared: p.shared_count.to_string(),
        })
        .collect();
    let mut table = Table::new(rows);
    table
        .with(Style::rounded())
        .modify(Columns::new(..), Alignment::right());
    println!("{table}");
}

pub fn export_portfolio_json(report: &PortfolioReport, path: &Path) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(report)?;
    let mut file = File::create(path)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

fn pct(v: f64) -> String {
    format!("{:.2}%", v * 100.0)
}

fn pct_ratio(v: f64) -> String {
    format!("{:.1}%", v * 100.0)
}

fn fmt(v: f64) -> String {
    format!("{:.2}", v)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CorrelationMatrix, PortfolioMember, PortfolioReport, PortfolioSummary};

    fn sample_report() -> PortfolioReport {
        PortfolioReport {
            summary: PortfolioSummary {
                name: "demo".into(),
                period_days: 90,
                aligned_days: 60,
                total_return: 0.05,
                annualized_return: 0.08,
                volatility: 0.12,
                max_drawdown: 0.04,
                sharpe_ratio: 0.9,
                members: vec![PortfolioMember {
                    code: "000001".into(),
                    name: "t".into(),
                    weight: 1.0,
                    total_return: 0.05,
                    volatility: 0.1,
                    max_drawdown: 0.03,
                    sharpe_ratio: 1.0,
                    return_contribution: 0.05,
                }],
            },
            correlation: CorrelationMatrix {
                labels: vec!["000001".into()],
                values: vec![vec![1.0]],
            },
            overlaps: vec![],
            interpretation: None,
            series: None,
        }
    }

    #[test]
    fn export_json_writes_file() {
        let dir = std::env::temp_dir().join(format!("af_pf_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("pf.json");
        export_portfolio_json(&sample_report(), &path).unwrap();
        let s = std::fs::read_to_string(&path).unwrap();
        assert!(s.contains("correlation"));
        let _ = std::fs::remove_dir_all(dir);
    }
}
