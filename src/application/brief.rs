//! 单基金选基综合简报：分析 + 行业 + 重仓。

use super::context::{require_online, resolve_fund_ids, CommandContext};
use super::fund_service::{analyze_fund, resolve_fund_identifier};
use crate::api::fund_holdings::FundStockHoldingsReport;
use crate::api::fund_industry::FundIndustryReport;
use crate::domain::resolve_analysis_days;
use crate::models::FundAnalysis;
use crate::presentation::{
    print_analysis, print_holdings_report, print_industry_report, truncate_string,
};
use std::fs;
use std::path::Path;

/// 综合简报数据（供终端与 Markdown 共用）。
#[derive(Debug, Clone)]
pub struct FundBrief {
    pub code: String,
    pub name: String,
    pub fund_type: String,
    pub company: String,
    pub asset_size: String,
    pub days: u32,
    pub analysis: Option<FundAnalysis>,
    pub industry: FundIndustryReport,
    pub holdings: FundStockHoldingsReport,
    pub industry_top: usize,
    pub holdings_top: usize,
}

/// `brief` 请求参数。
pub struct BriefRequest {
    pub code: Option<String>,
    pub pick_watchlist: bool,
    pub days: u32,
    pub period: Option<String>,
    pub industry_top: u32,
    pub holdings_top: u32,
    pub output: Option<std::path::PathBuf>,
}

pub async fn run_brief(ctx: &CommandContext<'_>, req: BriefRequest) -> anyhow::Result<()> {
    require_online(ctx.offline, "brief")?;
    let days = resolve_analysis_days(req.period.as_deref(), req.days)?;
    let ids = resolve_fund_ids(
        req.code,
        req.pick_watchlist,
        ctx.watchlist_path,
        "--code/--watchlist",
    )?;
    let multi = ids.len() > 1;
    for id in ids {
        let brief =
            gather_brief(&ctx.session, &id, days, req.holdings_top, req.industry_top).await?;
        render_brief_terminal(&brief);
        if let Some(ref path) = req.output {
            write_brief_markdown(&brief, path)?;
            tracing::info!(path = %path.display(), "Wrote brief markdown");
        }
        if multi {
            println!();
            println!("{}", "=".repeat(72));
            println!();
        }
    }
    Ok(())
}

async fn gather_brief(
    session: &super::context::Session<'_>,
    identifier: &str,
    days: u32,
    holdings_top: u32,
    industry_top: u32,
) -> anyhow::Result<FundBrief> {
    let (code, name) = resolve_fund_identifier(session, identifier, false).await?;
    tracing::info!(code = %code, days = days, "Building fund brief");

    let analysis = analyze_fund(session, &code, days, false).await?;

    let profile = session.client.fetch_fund_profile(&code).await.ok();
    let fund_type = profile
        .as_ref()
        .map(|p| p.fund_type.clone())
        .unwrap_or_default();
    let company = profile
        .as_ref()
        .map(|p| p.company.clone())
        .unwrap_or_default();
    let asset_size = profile
        .as_ref()
        .map(|p| p.asset_size.clone())
        .unwrap_or_default();
    let display_name = profile
        .as_ref()
        .map(|p| p.name.clone())
        .filter(|n| !n.is_empty())
        .unwrap_or(name);

    let industry = session
        .client
        .fetch_fund_industry_allocation(&code)
        .await
        .unwrap_or_default();
    let holdings = session
        .client
        .fetch_fund_stock_holdings(&code, holdings_top.clamp(1, 50))
        .await
        .unwrap_or_default();

    Ok(FundBrief {
        code,
        name: display_name,
        fund_type,
        company,
        asset_size,
        days,
        analysis,
        industry,
        holdings,
        industry_top: industry_top as usize,
        holdings_top: holdings_top as usize,
    })
}

fn render_brief_terminal(b: &FundBrief) {
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
    print_industry_report(&b.code, &b.name, &ind);

    println!();
    let mut hold = b.holdings.clone();
    hold.rows.truncate(b.holdings_top.max(1));
    print_holdings_report(&b.code, &b.name, &hold);
}

fn write_brief_markdown(b: &FundBrief, path: &Path) -> anyhow::Result<()> {
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

    md.push_str("\n---\n\n_数据来源：天天基金；仅供研究参考，不构成投资建议。_\n");
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

fn append_industry_md(md: &mut String, report: &FundIndustryReport, top: usize) {
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

fn append_holdings_md(md: &mut String, report: &FundStockHoldingsReport, top: usize) {
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
