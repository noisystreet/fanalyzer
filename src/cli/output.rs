use crate::api::eastmoney::FundProfile;
use crate::models::{FundAnalysis, FundNav};
use std::fs::File;
use std::io::Write;

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

pub fn print_comparison(analyses: &[FundAnalysis]) {
    println!("基金对比分析");
    println!();
    println!(
        "{:<10} {:<16} {:>10} {:>12} {:>10} {:>10} {:>10} {:>10} {:>8} {:>8} {:>8}",
        "基金代码",
        "基金名称",
        "总收益率",
        "年化收益率",
        "波动率",
        "最大回撤",
        "夏普比率",
        "Alpha",
        "Beta",
        "管理费",
        "托管费"
    );
    println!("{}", "-".repeat(130));
    for a in analyses {
        let name = truncate_string(&a.name, 14);
        let mgmt_fee = if a.management_fee > 0.0 {
            format!("{:.2}%", a.management_fee)
        } else {
            "-".to_string()
        };
        let custody_fee = if a.custody_fee > 0.0 {
            format!("{:.2}%", a.custody_fee)
        } else {
            "-".to_string()
        };
        println!(
            "{:<10} {:<16} {:>9.2}% {:>11.2}% {:>9.2}% {:>9.2}% {:>10.2} {:>9.2}% {:>8.2} {:>8} {:>8}",
            a.code,
            name,
            a.total_return * 100.0,
            a.annualized_return * 100.0,
            a.volatility * 100.0,
            a.max_drawdown * 100.0,
            a.sharpe_ratio,
            a.alpha * 100.0,
            a.beta,
            mgmt_fee,
            custody_fee
        );
    }

    println!();
    println!("基金经理信息");
    println!("{}", "-".repeat(80));
    for a in analyses {
        if !a.manager_name.is_empty() {
            let tenure_years = a.manager_tenure_days as f64 / 365.0;
            println!(
                "{} {:<16} 经理: {:<10} 任期: {:>5.1}年 任职回报: {:>6.2}%",
                a.code,
                truncate_string(&a.name, 14),
                a.manager_name,
                tenure_years,
                a.manager_total_return * 100.0
            );
        }
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
