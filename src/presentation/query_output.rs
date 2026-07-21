//! 查询类命令终端输出（profile / 行业 / 重仓 / 排行）。

use crate::models::{FundOverview, FundRankRow, IndustryAllocation, StockHoldings};
use crate::presentation::truncate_string;
use comfy_table::*;

const ROUNDED_UTF8: &str = "││──╞═╪╡┆╌┼├┤┬┴╭╮╰╯";

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

fn fmt_pct_opt(v: Option<f64>) -> String {
    v.map(|x| format!("{x:.2}%"))
        .unwrap_or_else(|| "-".to_string())
}

fn ranking_table_rows(rows: &[FundRankRow]) -> (Vec<String>, Vec<Vec<String>>) {
    let header = vec![
        "代码".into(),
        "简称".into(),
        "近1周".into(),
        "近1月".into(),
        "近3月".into(),
        "近6月".into(),
        "近1年".into(),
        "今年来".into(),
    ];
    let data = rows
        .iter()
        .map(|r| {
            vec![
                r.code.clone(),
                truncate_string(&r.name, 20),
                fmt_pct_opt(r.pct_week),
                fmt_pct_opt(r.pct_month),
                fmt_pct_opt(r.pct_3m),
                fmt_pct_opt(r.pct_6m),
                fmt_pct_opt(r.pct_1y),
                fmt_pct_opt(r.pct_this_year),
            ]
        })
        .collect();
    (header, data)
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
    let (header, data) = ranking_table_rows(rows);
    print_rounded_table(header, data, 2);
}

fn industry_display_rows(report: &IndustryAllocation) -> (Vec<String>, Vec<Vec<String>>) {
    let header = vec![
        "序号".into(),
        "行业类别".into(),
        "占净值比例".into(),
        "市值(万元)".into(),
    ];
    let rows = report
        .rows
        .iter()
        .map(|r| {
            vec![
                r.rank.to_string(),
                truncate_string(&r.industry, 36),
                format!("{:.2}%", r.pct_nav),
                r.market_value_wan
                    .map(|v| format!("{v:.2}"))
                    .unwrap_or_else(|| "-".to_string()),
            ]
        })
        .collect();
    (header, rows)
}

fn holdings_display_rows(report: &StockHoldings) -> (Vec<String>, Vec<Vec<String>>) {
    let header = vec![
        "序号".into(),
        "股票代码".into(),
        "股票名称".into(),
        "占净值比例".into(),
        "较上期".into(),
        "持股数(万股)".into(),
        "持仓市值(万元)".into(),
    ];
    let rows = report
        .rows
        .iter()
        .map(|r| {
            vec![
                r.rank.to_string(),
                r.stock_code.clone(),
                truncate_string(&r.stock_name, 16),
                format!("{:.2}%", r.pct_nav),
                r.pct_nav_chg
                    .map(|v| format!("{v:+.2}"))
                    .unwrap_or_else(|| "-".to_string()),
                r.shares_wan
                    .map(|v| format!("{v:.2}"))
                    .unwrap_or_else(|| "-".to_string()),
                r.market_value_wan
                    .map(|v| format!("{v:.2}"))
                    .unwrap_or_else(|| "-".to_string()),
            ]
        })
        .collect();
    (header, rows)
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
    let (header, rows) = holdings_display_rows(report);
    print_rounded_table(header, rows, 3);
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
    let (header, rows) = industry_display_rows(report);
    print_rounded_table(header, rows, 2);
}

pub fn print_fund_overview(profile: &FundOverview) {
    println!("基金概况");
    println!("{}", "=".repeat(60));
    print_overview_identity(profile);
    print_peer_rank_line(&profile.peer_rank);
    print_allocation_section(profile.allocation.as_ref());

    if !profile.benchmark.is_empty() {
        println!();
        println!("业绩比较基准");
        println!("{}", "-".repeat(60));
        println!("{}", profile.benchmark);
    }

    println!();
    println!("基金经理");
    println!("{}", "-".repeat(60));
    print_managers_section(profile);

    println!("费率信息");
    println!("{}", "-".repeat(60));
    println!("管理费率: {:.2}%", profile.management_fee);
    if profile.custody_fee > 0.0 {
        println!("托管费率: {:.2}%", profile.custody_fee);
    }

    print_overview_investment_texts(profile);
}

fn print_overview_identity(profile: &FundOverview) {
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
}

fn print_overview_investment_texts(profile: &FundOverview) {
    if !profile.investment_target.is_empty() {
        println!();
        println!("投资目标");
        println!("{}", "-".repeat(60));
        println!("{}", profile.investment_target);
    }

    if profile.investment_scope.is_empty() {
        return;
    }
    println!();
    println!("投资范围");
    println!("{}", "-".repeat(60));
    let scope = &profile.investment_scope;
    if scope.len() > 80 {
        for sentence in scope.split('。').filter(|s| !s.is_empty()) {
            println!("{}", sentence.trim());
        }
    } else {
        println!("{scope}");
    }
}

fn print_peer_rank_line(peer_rank: &crate::models::PeerRankInfo) {
    let (Some(rank), Some(count)) = (peer_rank.rank, peer_rank.peer_count) else {
        return;
    };
    let mut line = format!("同类排名(近3月): {rank}/{count}");
    if let Some(pct) = peer_rank.percentile {
        line.push_str(&format!("（前 {pct:.1}%）"));
    }
    if let Some(ref as_of) = peer_rank.as_of {
        line.push_str(&format!("  截至 {as_of}"));
    }
    println!("{line}");
}

fn print_allocation_section(allocation: Option<&crate::models::AssetAllocationSnapshot>) {
    let Some(a) = allocation else {
        return;
    };
    println!();
    println!("资产配置");
    println!("{}", "-".repeat(60));
    println!("报告期: {}", a.as_of);
    println!(
        "股票 {:.2}%  债券 {:.2}%  现金 {:.2}%",
        a.stock_pct, a.bond_pct, a.cash_pct
    );
    if let Some(nav) = a.net_asset_yi {
        println!("净资产: {nav:.2} 亿元");
    }
    if let Some(chg) = a.stock_pct_chg {
        println!("股票仓位较上期: {chg:+.2} pct");
    }
    if let Some(ref summary) = a.summary {
        println!("{summary}");
    }
}

fn print_managers_section(profile: &FundOverview) {
    if profile.managers.is_empty() {
        println!("姓名: {}", profile.manager_name);
        println!("任期: {:.1} 年", profile.manager_tenure_days as f64 / 365.0);
        println!("任职回报: {:.2}%", profile.manager_total_return * 100.0);
        return;
    }
    for m in &profile.managers {
        println!("姓名: {}", m.name);
        if !m.start_date.is_empty() {
            println!("上任日期: {}", m.start_date);
        }
        if m.tenure_days > 0 {
            println!("任期: {:.1} 年", m.tenure_days as f64 / 365.0);
        }
        if m.total_return != 0.0 {
            println!("任职回报: {:.2}%", m.total_return * 100.0);
        }
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranking_table_rows_empty_ok() {
        let (_, rows) = ranking_table_rows(&[]);
        assert!(rows.is_empty());
    }
}
