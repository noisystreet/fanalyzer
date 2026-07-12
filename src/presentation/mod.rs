//! 终端呈现：表格、报告与导出。

mod analysis;
mod brief;
mod comparison;
mod fetch;
mod portfolio;
mod query_output;
mod screen_output;
mod structured;

use crate::models::FundAnalysis;
pub use analysis::{export_analysis_json, render_analysis};
pub use brief::{print_brief_separator, render_brief_terminal, write_brief_markdown};
use comfy_table::*;
pub use comparison::{export_comparison_csv, export_comparison_json, render_comparison};
pub use fetch::print_fetch_result;
pub use portfolio::{export_portfolio_json, print_portfolio_report, render_portfolio};
pub use query_output::{print_fund_overview, print_holdings, print_industry, print_ranking_table};
pub use screen_output::{
    ScreenHeaderContext, print_deep_limit_hint, print_filter_hint, print_insufficient_candidates,
    print_rank_prefilter, print_screen_header, print_screen_passed,
};
use std::fs::File;
use std::io::Write;
pub use structured::{
    AnalysisMeta, BaseMeta, BatchMeta, BatchPayload, CodedError, ENVELOPE_VERSION, ExportMeta,
    ExportPayload, FetchPayload, HoldingsItem, ItemError, ItemsPayload, PortfolioMeta, RankMeta,
    RankPayload, ScreenMeta, ScreenPayload, SectorItem, StructuredEnvelope, StructuredError,
    StructuredFailureEnvelope, base_meta, compact_analysis_reports, compact_brief_summary,
    compact_portfolio_report, emit, error_from_anyhow, failure_envelope_json, item_error,
    item_error_failed, item_error_insufficient, print_failure_capture, print_failure_from_anyhow,
    success_envelope_json, write_file,
};

/// Rounded-corner UTF8 preset, similar to tabled's `Style::rounded()`.
const ROUNDED_UTF8: &str = "││──╞═╪╡┆╌┼├┤┬┴╭╮╰╯";

pub fn print_analysis(analysis: &FundAnalysis) {
    println!("基金分析报告");
    println!("基金名称: {}", analysis.name);
    println!("基金代码: {}", analysis.code);
    println!("分析周期: {} 天", analysis.period_days);
    println!("平均净值: {:.4}", analysis.avg_nav);
    println!("最高净值: {:.4}", analysis.max_nav);
    println!("最低净值: {:.4}", analysis.min_nav);
    println!("总收益率: {:.2}%", analysis.total_return * 100.0);
    println!("年化收益率: {:.2}%", analysis.annualized_return * 100.0);
    println!("波动率: {:.2}%", analysis.volatility * 100.0);
    println!("最大回撤: {:.2}%", analysis.max_drawdown * 100.0);
    println!("夏普比率: {:.2}", analysis.sharpe_ratio);
    println!("索提诺比率: {:.2}", analysis.sortino_ratio);
    println!("卡玛比率: {:.2}", analysis.calmar_ratio);
    println!("阿尔法 (Alpha): {:.2}%", analysis.alpha * 100.0);
    println!("贝塔 (Beta): {:.2}", analysis.beta);

    if !analysis.manager_name.is_empty() {
        println!("基金经理: {}", analysis.manager_name);
        let tenure_years = analysis.manager_tenure_days as f64 / 365.0;
        println!("经理任期: {:.1} 年", tenure_years);
        println!(
            "经理任职回报: {:.2}%",
            analysis.manager_total_return * 100.0
        );
    }

    if analysis.management_fee > 0.0 {
        println!("管理费率: {:.2}%", analysis.management_fee);
        println!("托管费率: {:.2}%", analysis.custody_fee);
    }
}

fn comparison_rows(analyses: &[FundAnalysis]) -> (Vec<String>, Vec<Vec<String>>) {
    let header = vec![
        "基金代码".into(),
        "基金名称".into(),
        "总收益率".into(),
        "年化收益率".into(),
        "波动率".into(),
        "最大回撤".into(),
        "夏普比率".into(),
        "Sortino".into(),
        "Calmar".into(),
        "Alpha".into(),
        "Beta".into(),
        "管理费".into(),
        "托管费".into(),
    ];
    let rows = analyses
        .iter()
        .map(|a| {
            let mgmt_fee = if a.management_fee > 0.0 {
                format!("{:.2}%", a.management_fee)
            } else {
                "-".to_string()
            };
            let custody_fee_str = if a.custody_fee > 0.0 {
                format!("{:.2}%", a.custody_fee)
            } else {
                "-".to_string()
            };
            vec![
                a.code.clone(),
                truncate_string(&a.name, 14),
                format!("{:.2}%", a.total_return * 100.0),
                format!("{:.2}%", a.annualized_return * 100.0),
                format!("{:.2}%", a.volatility * 100.0),
                format!("{:.2}%", a.max_drawdown * 100.0),
                format!("{:.2}", a.sharpe_ratio),
                format!("{:.2}", a.sortino_ratio),
                format!("{:.2}", a.calmar_ratio),
                format!("{:.2}%", a.alpha * 100.0),
                format!("{:.2}", a.beta),
                mgmt_fee,
                custody_fee_str,
            ]
        })
        .collect();
    (header, rows)
}

fn manager_compare_rows(analyses: &[FundAnalysis]) -> (Vec<String>, Vec<Vec<String>>) {
    let header = vec![
        "代码".into(),
        "简称".into(),
        "经理".into(),
        "任期(年)".into(),
        "任职回报".into(),
    ];
    let rows = analyses
        .iter()
        .filter(|a| !a.manager_name.is_empty())
        .map(|a| {
            let tenure_years = a.manager_tenure_days as f64 / 365.0;
            vec![
                a.code.clone(),
                truncate_string(&a.name, 14),
                a.manager_name.clone(),
                format!("{tenure_years:.1}"),
                format!("{:.2}%", a.manager_total_return * 100.0),
            ]
        })
        .collect();
    (header, rows)
}

fn print_rounded_table(header: Vec<String>, rows: Vec<Vec<String>>, right_align_from_col: usize) {
    let mut table = Table::new();
    table.load_preset(ROUNDED_UTF8);
    table.set_header(header);
    for row in rows {
        table.add_row(row);
    }
    for (idx, col) in table.column_iter_mut().enumerate() {
        if idx >= right_align_from_col {
            col.set_cell_alignment(CellAlignment::Right);
        }
    }
    println!("{table}");
}

pub fn print_comparison(analyses: &[FundAnalysis]) {
    println!("基金对比分析");
    println!();
    let (header, rows) = comparison_rows(analyses);
    print_rounded_table(header, rows, 2);
    println!();
    println!("基金经理信息");
    let (mgr_header, mgr_rows) = manager_compare_rows(analyses);
    if mgr_rows.is_empty() {
        println!("（无经理信息）");
    } else {
        print_rounded_table(mgr_header, mgr_rows, 3);
    }
}

pub fn truncate_string(s: &str, max_chars: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max_chars {
        s.to_string()
    } else {
        format!("{}..", chars[..max_chars].iter().collect::<String>())
    }
}

pub fn export_csv(navs: &[crate::models::FundNav], path: &str) -> anyhow::Result<()> {
    let mut writer = csv::Writer::from_path(path)?;
    writer.write_record(["date", "code", "nav", "acc_nav", "daily_return"])?;
    for nav in navs {
        writer.write_record([
            nav.date.to_string(),
            nav.code.clone(),
            nav.nav.to_string(),
            nav.acc_nav.to_string(),
            nav.daily_return.map(|r| r.to_string()).unwrap_or_default(),
        ])?;
    }
    writer.flush()?;
    Ok(())
}

pub fn export_json(navs: &[crate::models::FundNav], path: &str) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(navs)?;
    let mut file = File::create(path)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::FundAnalysis;

    fn sample_analysis() -> FundAnalysis {
        FundAnalysis {
            code: "000001".into(),
            name: "测试基金".into(),
            period_days: 30,
            avg_nav: 1.0,
            max_nav: 1.1,
            min_nav: 0.9,
            total_return: 0.01,
            annualized_return: 0.02,
            volatility: 0.03,
            max_drawdown: -0.04,
            sharpe_ratio: 1.5,
            sortino_ratio: 1.6,
            calmar_ratio: 1.4,
            alpha: 0.001,
            beta: 0.9,
            manager_name: "张三".into(),
            manager_tenure_days: 365,
            manager_total_return: 0.05,
            management_fee: 1.2,
            custody_fee: 0.2,
        }
    }

    #[test]
    fn comparison_rows_formats_percent_columns() {
        let (_, rows) = comparison_rows(&[sample_analysis()]);
        assert_eq!(rows.len(), 1);
        assert!(rows[0][2].ends_with('%'));
        let (_, mgr_rows) = manager_compare_rows(&[sample_analysis()]);
        assert_eq!(mgr_rows.len(), 1);
    }

    #[test]
    fn rounded_style_emits_unicode_table() {
        let (header, rows) = comparison_rows(&[sample_analysis()]);
        let mut table = Table::new();
        table.load_preset(ROUNDED_UTF8);
        table.set_header(header);
        for row in rows {
            table.add_row(row);
        }
        let rendered = table.to_string();
        assert!(
            rendered.contains('╭') && rendered.contains('╰'),
            "expected rounded corners in table output"
        );
    }
}
