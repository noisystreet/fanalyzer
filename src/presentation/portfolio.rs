//! 组合分析终端输出与导出。

use crate::models::{CorrelationMatrix, OverlapPair, PortfolioInterpretation, PortfolioReport};
use comfy_table::*;
use std::fs::File;
use std::io::Write;
use std::path::Path;

const ROUNDED_UTF8: &str = "││──╞═╪╡┆╌┼├┤┬┴╭╮╰╯";

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

fn print_members_table(s: &crate::models::PortfolioSummary) {
    println!("成分基金与静态贡献（weight × 单基总收益）");
    let header: Vec<String> = vec![
        "代码".into(),
        "名称".into(),
        "权重".into(),
        "总收益".into(),
        "波动率".into(),
        "最大回撤".into(),
        "夏普".into(),
        "收益贡献".into(),
    ];
    let rows: Vec<Vec<String>> = s
        .members
        .iter()
        .map(|m| {
            vec![
                m.code.clone(),
                m.name.clone(),
                pct_ratio(m.weight),
                pct(m.total_return),
                pct(m.volatility),
                pct(m.max_drawdown),
                fmt(m.sharpe_ratio),
                pct(m.return_contribution),
            ]
        })
        .collect();
    let mut table = Table::new();
    table.load_preset(ROUNDED_UTF8);
    table.set_header(header);
    for row in rows {
        table.add_row(row);
    }
    table
        .column_iter_mut()
        .for_each(|col| col.set_cell_alignment(CellAlignment::Right));
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

fn print_overlap_table(pairs: &[OverlapPair]) {
    println!("重仓股加权重叠（前 N 大重仓，min(占净值%) 之和）");
    let header: Vec<String> = vec![
        "基金A".into(),
        "基金B".into(),
        "加权重叠%".into(),
        "共同持仓数".into(),
    ];
    let rows: Vec<Vec<String>> = pairs
        .iter()
        .map(|p| {
            vec![
                format!("{} {}", p.fund_a_code, p.fund_a_name),
                format!("{} {}", p.fund_b_code, p.fund_b_name),
                format!("{:.2}", p.overlap_pct),
                p.shared_count.to_string(),
            ]
        })
        .collect();
    let mut table = Table::new();
    table.load_preset(ROUNDED_UTF8);
    table.set_header(header);
    for row in rows {
        table.add_row(row);
    }
    table
        .column_iter_mut()
        .for_each(|col| col.set_cell_alignment(CellAlignment::Right));
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
