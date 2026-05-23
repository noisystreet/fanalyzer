//! 查询类命令终端输出（profile / 行业 / 重仓 / 排行）。

use crate::models::{FundOverview, FundRankRow, IndustryAllocation, StockHoldings};
use crate::presentation::truncate_string;
use tabled::settings::{object::Columns, Alignment, Style};
use tabled::{Table, Tabled};

fn print_rounded_table<T: Tabled>(rows: &[T], right_align_from_col: usize) {
    let mut table = Table::new(rows);
    table.with(Style::rounded());
    table.modify(Columns::new(right_align_from_col..), Alignment::right());
    println!("{table}");
}

fn fmt_pct_opt(v: Option<f64>) -> String {
    v.map(|x| format!("{x:.2}%"))
        .unwrap_or_else(|| "-".to_string())
}

#[derive(Tabled)]
struct RankingTableRow {
    #[tabled(rename = "代码")]
    code: String,
    #[tabled(rename = "简称")]
    name: String,
    #[tabled(rename = "近1周")]
    week: String,
    #[tabled(rename = "近1月")]
    month: String,
    #[tabled(rename = "近3月")]
    three_m: String,
    #[tabled(rename = "近6月")]
    six_m: String,
    #[tabled(rename = "近1年")]
    one_y: String,
    #[tabled(rename = "今年来")]
    ytd: String,
}

fn ranking_table_rows(rows: &[FundRankRow]) -> Vec<RankingTableRow> {
    rows.iter()
        .map(|r| RankingTableRow {
            code: r.code.clone(),
            name: truncate_string(&r.name, 20),
            week: fmt_pct_opt(r.pct_week),
            month: fmt_pct_opt(r.pct_month),
            three_m: fmt_pct_opt(r.pct_3m),
            six_m: fmt_pct_opt(r.pct_6m),
            one_y: fmt_pct_opt(r.pct_1y),
            ytd: fmt_pct_opt(r.pct_this_year),
        })
        .collect()
}

pub fn print_ranking_table(rows: &[FundRankRow], kind: &str, sort: &str, universe_total: u32) {
    println!(
        "开放式基金排行（ft={}，排序 sc={}，官网该类型约 {} 只，下列 {} 条）",
        kind,
        sort,
        universe_total,
        rows.len()
    );
    println!();
    print_rounded_table(&ranking_table_rows(rows), 2);
}

#[derive(Tabled)]
struct IndustryTableRow {
    #[tabled(rename = "序号")]
    rank: u32,
    #[tabled(rename = "行业类别")]
    industry: String,
    #[tabled(rename = "占净值比例")]
    pct_nav: String,
    #[tabled(rename = "市值(万元)")]
    market_value_wan: String,
}

fn industry_display_rows(report: &IndustryAllocation) -> Vec<IndustryTableRow> {
    report
        .rows
        .iter()
        .map(|r| IndustryTableRow {
            rank: r.rank,
            industry: truncate_string(&r.industry, 36),
            pct_nav: format!("{:.2}%", r.pct_nav),
            market_value_wan: r
                .market_value_wan
                .map(|v| format!("{v:.2}"))
                .unwrap_or_else(|| "-".to_string()),
        })
        .collect()
}

#[derive(Tabled)]
struct HoldingTableRow {
    #[tabled(rename = "序号")]
    rank: u32,
    #[tabled(rename = "股票代码")]
    stock_code: String,
    #[tabled(rename = "股票名称")]
    stock_name: String,
    #[tabled(rename = "占净值比例")]
    pct_nav: String,
    #[tabled(rename = "持股数(万股)")]
    shares_wan: String,
    #[tabled(rename = "持仓市值(万元)")]
    market_value_wan: String,
}

fn holdings_display_rows(report: &StockHoldings) -> Vec<HoldingTableRow> {
    report
        .rows
        .iter()
        .map(|r| HoldingTableRow {
            rank: r.rank,
            stock_code: r.stock_code.clone(),
            stock_name: truncate_string(&r.stock_name, 16),
            pct_nav: format!("{:.2}%", r.pct_nav),
            shares_wan: r
                .shares_wan
                .map(|v| format!("{v:.2}"))
                .unwrap_or_else(|| "-".to_string()),
            market_value_wan: r
                .market_value_wan
                .map(|v| format!("{v:.2}"))
                .unwrap_or_else(|| "-".to_string()),
        })
        .collect()
}

pub fn print_holdings(code: &str, name: &str, report: &StockHoldings) {
    println!("重仓股（股票投资明细，季报披露）");
    println!("基金代码: {code}  简称: {name}");
    if let Some(ref d) = report.as_of {
        println!("报告截止: {d}");
    }
    println!();
    if report.rows.is_empty() {
        println!("暂无重仓股数据（常见于债券型、货币型或当季未持股）。");
        return;
    }
    print_rounded_table(&holdings_display_rows(report), 3);
}

pub fn print_industry(code: &str, name: &str, report: &IndustryAllocation) {
    println!("行业配置（板块 — 证监会行业分类）");
    println!("基金代码: {code}  简称: {name}");
    if let Some(ref d) = report.as_of {
        println!("报告截止: {d}");
    }
    println!();
    if report.rows.is_empty() {
        println!("暂无行业配置数据（常见于债券型、货币型或极低股票仓位）。");
        return;
    }
    print_rounded_table(&industry_display_rows(report), 2);
}

pub fn print_fund_overview(profile: &FundOverview) {
    println!("基金概况");
    println!("{}", "=".repeat(60));

    if !profile.full_name.is_empty() {
        println!("基金全称: {}", profile.full_name);
    }
    println!("基金简称: {}", profile.name);
    println!("基金代码: {}", profile.code);
    if !profile.fund_type.is_empty() {
        println!("基金类型: {}", profile.fund_type);
    }
    if !profile.establishment_date.is_empty() {
        println!("成立日期: {}", profile.establishment_date);
    }
    if !profile.asset_size.is_empty() {
        println!("资产规模: {}", profile.asset_size);
    }
    if !profile.company.is_empty() {
        println!("管理公司: {}", profile.company);
    }

    if !profile.benchmark.is_empty() {
        println!();
        println!("业绩比较基准");
        println!("{}", "-".repeat(60));
        println!("{}", profile.benchmark);
    }

    println!();
    println!("基金经理");
    println!("{}", "-".repeat(60));
    println!("姓名: {}", profile.manager_name);
    let tenure_years = profile.manager_tenure_days as f64 / 365.0;
    println!("任期: {:.1} 年", tenure_years);
    println!("任职回报: {:.2}%", profile.manager_total_return * 100.0);

    println!();
    println!("费率信息");
    println!("{}", "-".repeat(60));
    println!("管理费率: {:.2}%", profile.management_fee);
    if profile.custody_fee > 0.0 {
        println!("托管费率: {:.2}%", profile.custody_fee);
    }

    if !profile.investment_target.is_empty() {
        println!();
        println!("投资目标");
        println!("{}", "-".repeat(60));
        println!("{}", profile.investment_target);
    }

    if !profile.investment_scope.is_empty() {
        println!();
        println!("投资范围");
        println!("{}", "-".repeat(60));
        let scope = &profile.investment_scope;
        if scope.len() > 80 {
            for sentence in scope.split('。').filter(|s| !s.is_empty()) {
                println!("{}", sentence.trim());
            }
        } else {
            println!("{}", scope);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranking_table_rows_empty_ok() {
        assert!(ranking_table_rows(&[]).is_empty());
    }
}
