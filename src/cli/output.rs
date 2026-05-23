use crate::api::eastmoney::FundProfile;
use crate::api::fund_holdings::FundStockHoldingsReport;
use crate::api::fund_industry::FundIndustryReport;
use crate::api::fund_ranking::FundRankEntry;
use crate::models::{FundAnalysis, FundNav};
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

/// 圆角边框表格；从第 `right_align_from_col` 列起右对齐（0-based，含该列）。
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

pub fn export_csv(navs: &[FundNav], path: &str) -> anyhow::Result<()> {
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

pub fn export_json(navs: &[FundNav], path: &str) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(navs)?;
    let mut file = File::create(path)?;
    file.write_all(json.as_bytes())?;
    Ok(())
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

fn ranking_table_rows(rows: &[FundRankEntry]) -> Vec<RankingTableRow> {
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

/// 打印官网开放式基金排行简表（百分点列）。
pub fn print_ranking_table(rows: &[FundRankEntry], kind: &str, sort: &str, universe_total: u32) {
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

fn industry_display_rows(report: &FundIndustryReport) -> Vec<IndustryTableRow> {
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

fn holdings_display_rows(report: &FundStockHoldingsReport) -> Vec<HoldingTableRow> {
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

/// 打印季报股票投资明细（重仓）。
pub fn print_holdings_report(code: &str, name: &str, report: &FundStockHoldingsReport) {
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

/// 打印 F10 披露的行业配置（证监会行业分类）。
pub fn print_industry_report(code: &str, name: &str, report: &FundIndustryReport) {
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

pub fn print_fund_profile(profile: &FundProfile) {
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

    #[test]
    fn ranking_table_rows_empty_ok() {
        assert!(ranking_table_rows(&[]).is_empty());
    }
}
