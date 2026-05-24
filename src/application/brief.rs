//! 单基金选基综合简报：分析 + 行业 + 重仓。

use super::context::{require_online, resolve_fund_ids, CommandContext};
use super::fund_service::{analyze_fund, resolve_fund_identifier};
use super::mappers::{map_holdings, map_industry};
use crate::domain::resolve_analysis_days;
use crate::models::FundBrief;
use crate::presentation::{print_brief_separator, render_brief_terminal, write_brief_markdown};
use chrono::Local;

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
    let today = Local::now().date_naive();
    let days = resolve_analysis_days(req.period.as_deref(), req.days, today)?;
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
            print_brief_separator();
        }
    }
    Ok(())
}

pub async fn gather_brief(
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

    let industry_api = session
        .client
        .fetch_fund_industry_allocation(&code)
        .await
        .unwrap_or_default();
    let holdings_api = session
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
        industry: map_industry(&industry_api),
        holdings: map_holdings(&holdings_api),
        industry_top: industry_top as usize,
        holdings_top: holdings_top as usize,
    })
}
