//! 选基综合简报终端与 Markdown 输出。

use crate::models::{FundAnalysis, FundBrief, IndustryAllocation, StockHoldings};
use crate::presentation::{print_analysis, print_holdings, print_industry, truncate_string};
use std::fs;
use std::path::Path;

pub fn render_brief_terminal(b: &FundBrief) {
    println!("═══════════════════════════════════════════════════════════════");
    println!("选基综合简报");
    println!("═══════════════════════════════════════════════════════════════");
    println!("代码: {}  简称: {}", b.code, b.name);
    if !b.fund_type.is_empty() {
        println!("类型: {}", b.fund_type);
    }
    if !b.company.is_empty() {
        println!("管理人: {}", b.company);
    }
    if !b.asset_size.is_empty() {
        println!("规模: {}", b.asset_size);
    }
    println!();

    if let Some(ref a) = b.analysis {
        print_analysis(a);
    } else {
        println!("（净值分析数据不足，跳过风险收益段）");
    }
    println!();

    let mut ind = b.industry.clone();
    ind.rows.truncate(b.industry_top.max(1));
    print_industry(&b.code, &b.name, &ind);

    println!();
    let mut hold = b.holdings.clone();
    hold.rows.truncate(b.holdings_top.max(1));
    print_holdings(&b.code, &b.name, &hold);
}

pub fn print_brief_separator() {
    println!();
    println!("{}", "=".repeat(72));
    println!();
}

pub fn write_brief_markdown(b: &FundBrief, path: &Path) -> anyhow::Result<()> {
    let mut md = String::new();
    md.push_str(&format!("# 选基简报 — {} ({})\n\n", b.name, b.code));
    if !b.fund_type.is_empty() {
        md.push_str(&format!("- **类型**: {}\n", b.fund_type));
    }
    if !b.company.is_empty() {
        md.push_str(&format!("- **管理人**: {}\n", b.company));
    }
    if !b.asset_size.is_empty() {
        md.push_str(&format!("- **规模**: {}\n", b.asset_size));
    }
    md.push_str(&format!("- **分析窗口**: {} 日历天\n\n", b.days));

    md.push_str("## 风险与收益\n\n");
    if let Some(ref a) = b.analysis {
        append_analysis_md(&mut md, a);
    } else {
        md.push_str("_净值数据不足_\n\n");
    }

    md.push_str("## 行业配置（前若干项）\n\n");
    append_industry_md(&mut md, &b.industry, b.industry_top);

    md.push_str("\n## 重仓股\n\n");
    append_holdings_md(&mut md, &b.holdings, b.holdings_top);

    md.push_str("\n---\n\n_数据来源：东方财富 / 天天基金公开渠道；仅供个人研究参考，不构成投资建议。完整条款见项目 `docs/DISCLAIMER.md`。_\n");
    fs::write(path, md)?;
    Ok(())
}

fn append_analysis_md(md: &mut String, a: &FundAnalysis) {
    md.push_str("| 指标 | 数值 |\n|------|------|\n");
    let rows = [
        ("总收益率", format!("{:.2}%", a.total_return * 100.0)),
        ("年化收益率", format!("{:.2}%", a.annualized_return * 100.0)),
        ("波动率", format!("{:.2}%", a.volatility * 100.0)),
        ("最大回撤", format!("{:.2}%", a.max_drawdown * 100.0)),
        ("夏普比率", format!("{:.2}", a.sharpe_ratio)),
        ("Alpha", format!("{:.2}%", a.alpha * 100.0)),
        ("Beta", format!("{:.2}", a.beta)),
    ];
    for (k, v) in rows {
        md.push_str(&format!("| {k} | {v} |\n"));
    }
    if !a.manager_name.is_empty() {
        md.push_str(&format!(
            "\n**基金经理**: {}（任期 {:.1} 年，任职回报 {:.2}%）\n",
            a.manager_name,
            a.manager_tenure_days as f64 / 365.0,
            a.manager_total_return * 100.0
        ));
    }
    if a.management_fee > 0.0 {
        md.push_str(&format!(
            "\n**费率**: 管理 {:.2}%，托管 {:.2}%\n",
            a.management_fee, a.custody_fee
        ));
    }
    md.push('\n');
}

fn append_industry_md(md: &mut String, report: &IndustryAllocation, top: usize) {
    if report.rows.is_empty() {
        md.push_str("_暂无行业配置_\n");
        return;
    }
    if let Some(ref d) = report.as_of {
        md.push_str(&format!("报告截止: {d}\n\n"));
    }
    md.push_str("| 序号 | 行业 | 占净值 |\n|------|------|--------|\n");
    for r in report.rows.iter().take(top.max(1)) {
        md.push_str(&format!(
            "| {} | {} | {:.2}% |\n",
            r.rank,
            truncate_string(&r.industry, 40),
            r.pct_nav
        ));
    }
}

fn append_holdings_md(md: &mut String, report: &StockHoldings, top: usize) {
    if report.rows.is_empty() {
        md.push_str("_暂无重仓股_\n");
        return;
    }
    if let Some(ref d) = report.as_of {
        md.push_str(&format!("报告截止: {d}\n\n"));
    }
    md.push_str("| 序号 | 代码 | 名称 | 占净值 | 持股(万股) | 市值(万元) |\n");
    md.push_str("|------|------|------|--------|------------|------------|\n");
    for r in report.rows.iter().take(top.max(1)) {
        md.push_str(&format!(
            "| {} | {} | {} | {:.2}% | {} | {} |\n",
            r.rank,
            r.stock_code,
            truncate_string(&r.stock_name, 16),
            r.pct_nav,
            r.shares_wan
                .map(|v| format!("{v:.2}"))
                .unwrap_or_else(|| "-".into()),
            r.market_value_wan
                .map(|v| format!("{v:.2}"))
                .unwrap_or_else(|| "-".into()),
        ));
    }
}
