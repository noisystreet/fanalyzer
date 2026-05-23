//! 终端呈现：表格、报告与导出。

mod brief;
mod comparison;
mod fetch;
mod query_output;
mod screen_output;

use crate::models::FundAnalysis;
pub use brief::{print_brief_separator, render_brief_terminal, write_brief_markdown};
pub use comparison::{export_comparison_csv, export_comparison_json, render_comparison};
pub use fetch::print_fetch_result;
pub use query_output::{print_fund_overview, print_holdings, print_industry, print_ranking_table};
pub use screen_output::{
    print_deep_limit_hint, print_filter_hint, print_insufficient_candidates, print_rank_prefilter,
    print_screen_header, print_screen_passed, ScreenHeaderContext,
};
use std::fs::File;
use std::io::Write;
use tabled::settings::{object::Columns, Alignment, Style};
use tabled::{Table, Tabled};

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

#[derive(Tabled)]
struct ComparisonTableRow {
    #[tabled(rename = "基金代码")]
    code: String,
    #[tabled(rename = "基金名称")]
    name: String,
    #[tabled(rename = "总收益率")]
    total_return: String,
    #[tabled(rename = "年化收益率")]
    annualized_return: String,
    #[tabled(rename = "波动率")]
    volatility: String,
    #[tabled(rename = "最大回撤")]
    max_drawdown: String,
    #[tabled(rename = "夏普比率")]
    sharpe_ratio: String,
    #[tabled(rename = "Sortino")]
    sortino_ratio: String,
    #[tabled(rename = "Calmar")]
    calmar_ratio: String,
    #[tabled(rename = "Alpha")]
    alpha: String,
    #[tabled(rename = "Beta")]
    beta: String,
    #[tabled(rename = "管理费")]
    management_fee: String,
    #[tabled(rename = "托管费")]
    custody_fee: String,
}

#[derive(Tabled)]
struct ManagerCompareRow {
    #[tabled(rename = "代码")]
    code: String,
    #[tabled(rename = "简称")]
    name: String,
    #[tabled(rename = "经理")]
    manager_name: String,
    #[tabled(rename = "任期(年)")]
    tenure_years: String,
    #[tabled(rename = "任职回报")]
    tenure_return: String,
}

fn comparison_rows(analyses: &[FundAnalysis]) -> Vec<ComparisonTableRow> {
    analyses
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
            ComparisonTableRow {
                code: a.code.clone(),
                name: truncate_string(&a.name, 14),
                total_return: format!("{:.2}%", a.total_return * 100.0),
                annualized_return: format!("{:.2}%", a.annualized_return * 100.0),
                volatility: format!("{:.2}%", a.volatility * 100.0),
                max_drawdown: format!("{:.2}%", a.max_drawdown * 100.0),
                sharpe_ratio: format!("{:.2}", a.sharpe_ratio),
                sortino_ratio: format!("{:.2}", a.sortino_ratio),
                calmar_ratio: format!("{:.2}", a.calmar_ratio),
                alpha: format!("{:.2}%", a.alpha * 100.0),
                beta: format!("{:.2}", a.beta),
                management_fee: mgmt_fee,
                custody_fee: custody_fee_str,
            }
        })
        .collect()
}

fn manager_compare_rows(analyses: &[FundAnalysis]) -> Vec<ManagerCompareRow> {
    analyses
        .iter()
        .filter(|a| !a.manager_name.is_empty())
        .map(|a| {
            let tenure_years = a.manager_tenure_days as f64 / 365.0;
            ManagerCompareRow {
                code: a.code.clone(),
                name: truncate_string(&a.name, 14),
                manager_name: a.manager_name.clone(),
                tenure_years: format!("{tenure_years:.1}"),
                tenure_return: format!("{:.2}%", a.manager_total_return * 100.0),
            }
        })
        .collect()
}

fn print_rounded_table<T: Tabled>(rows: &[T], right_align_from_col: usize) {
    let mut table = Table::new(rows);
    table.with(Style::rounded());
    table.modify(Columns::new(right_align_from_col..), Alignment::right());
    println!("{table}");
}

pub fn print_comparison(analyses: &[FundAnalysis]) {
    println!("基金对比分析");
    println!();
    print_rounded_table(&comparison_rows(analyses), 2);
    println!();
    println!("基金经理信息");
    let mgr_rows = manager_compare_rows(analyses);
    if mgr_rows.is_empty() {
        println!("（无经理信息）");
    } else {
        print_rounded_table(&mgr_rows, 3);
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
        let rows = comparison_rows(&[sample_analysis()]);
        assert_eq!(rows.len(), 1);
        assert!(rows[0].total_return.ends_with('%'));
        assert_eq!(manager_compare_rows(&[sample_analysis()]).len(), 1);
    }

    #[test]
    fn rounded_style_emits_unicode_table() {
        let mut table = Table::new(comparison_rows(&[sample_analysis()]));
        table.with(Style::rounded());
        let rendered = table.to_string();
        assert!(
            rendered.contains('╭') && rendered.contains('╰'),
            "expected rounded corners in table output"
        );
    }
}
