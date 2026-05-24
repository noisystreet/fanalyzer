//! fetch / info / rank / sectors / holdings 查询用例。

use super::context::{require_online, resolve_fund_ids, CommandContext};
use super::fund_service::resolve_fund_identifier;
use super::mappers::{map_holdings, map_industry, map_profile, map_rank_rows};
use crate::domain::rank_ft_code;
use crate::presentation::{
    print_fetch_result, print_fund_overview, print_holdings, print_industry, print_ranking_table,
};

pub struct FetchRequest {
    pub code: Option<String>,
    pub pick_watchlist: bool,
    pub limit: u32,
}

pub struct InfoRequest {
    pub code: Option<String>,
    pub pick_watchlist: bool,
}

pub struct RankRequest {
    pub kind: String,
    pub top: u32,
    pub sort: String,
}

pub struct SectorsRequest {
    pub code: Option<String>,
    pub pick_watchlist: bool,
}

pub struct HoldingsRequest {
    pub code: Option<String>,
    pub pick_watchlist: bool,
    pub top: u32,
}

pub async fn run_fetch(ctx: &CommandContext<'_>, req: FetchRequest) -> anyhow::Result<()> {
    require_online(ctx.offline, "fetch")?;
    let ids = resolve_fund_ids(
        req.code,
        req.pick_watchlist,
        ctx.watchlist_path,
        "--code/--watchlist",
    )?;
    for id in ids {
        fetch_one(&ctx.session, id, req.limit).await?;
    }
    Ok(())
}

async fn fetch_one(
    session: &super::context::Session<'_>,
    code: String,
    limit: u32,
) -> anyhow::Result<()> {
    let (resolved_code, name) = resolve_fund_identifier(session, &code, false).await?;
    tracing::info!(code = %resolved_code, name = %name, limit = limit, "Fetching fund nav history");
    match session
        .client
        .fetch_nav_history(&resolved_code, 1, limit)
        .await
    {
        Ok((nav_list, total)) => {
            tracing::info!(total = total, fetched = nav_list.len(), "Fetched nav data");
            print_fetch_result(&resolved_code, &name, &nav_list, total);
        }
        Err(e) => tracing::error!(error = %e, "Failed to fetch nav history"),
    }
    Ok(())
}

pub async fn load_fund_overview(
    session: &super::context::Session<'_>,
    code: &str,
) -> anyhow::Result<crate::models::FundOverview> {
    let (resolved_code, _name) = resolve_fund_identifier(session, code, false).await?;
    tracing::info!(code = %resolved_code, "Fetching fund info");
    let profile = session
        .client
        .fetch_fund_profile(&resolved_code)
        .await
        .map_err(|e| anyhow::anyhow!("获取基金概况失败：{e}"))?;
    Ok(map_profile(&profile))
}

pub async fn run_info(ctx: &CommandContext<'_>, req: InfoRequest) -> anyhow::Result<()> {
    require_online(ctx.offline, "info")?;
    let ids = resolve_fund_ids(
        req.code,
        req.pick_watchlist,
        ctx.watchlist_path,
        "--code/--watchlist",
    )?;
    for id in ids {
        info_one(&ctx.session, id).await?;
    }
    Ok(())
}

async fn info_one(session: &super::context::Session<'_>, code: String) -> anyhow::Result<()> {
    match load_fund_overview(session, &code).await {
        Ok(profile) => print_fund_overview(&profile),
        Err(e) => tracing::error!(error = %e, "Failed to fetch fund info"),
    }
    Ok(())
}

pub async fn run_rank(ctx: &CommandContext<'_>, req: RankRequest) -> anyhow::Result<()> {
    require_online(ctx.offline, "rank")?;
    if req.top == 0 {
        anyhow::bail!("`--top` 须 ≥ 1");
    }
    if req.top > 500 {
        anyhow::bail!("`--top` 上限为 500");
    }
    let ft = rank_ft_code(&req.kind)?;
    let sc = req.sort.trim();
    if sc.is_empty() {
        anyhow::bail!("`--sort` 不能为空（默认可用 1n）");
    }
    tracing::info!(ft = ft, top = req.top, sort = %sc, "Fetching fund ranking");
    let page = ctx
        .session
        .client
        .fetch_fund_ranking_top(ft, sc, req.top)
        .await?;
    let rows = map_rank_rows(&page.rows);
    print_ranking_table(&rows, ft, sc, page.total_records);
    Ok(())
}

pub async fn run_sectors(ctx: &CommandContext<'_>, req: SectorsRequest) -> anyhow::Result<()> {
    require_online(ctx.offline, "sectors")?;
    let ids = resolve_fund_ids(
        req.code,
        req.pick_watchlist,
        ctx.watchlist_path,
        "--code/--watchlist",
    )?;
    for id in ids {
        sectors_one(&ctx.session, id).await?;
    }
    Ok(())
}

async fn sectors_one(session: &super::context::Session<'_>, code: String) -> anyhow::Result<()> {
    let (resolved_code, name) = resolve_fund_identifier(session, &code, false).await?;
    tracing::info!(code = %resolved_code, "Fetching industry allocation");
    let report = session
        .client
        .fetch_fund_industry_allocation(&resolved_code)
        .await
        .map_err(|e| anyhow::anyhow!("行业配置获取失败：{e}"))?;
    print_industry(&resolved_code, &name, &map_industry(&report));
    Ok(())
}

pub async fn run_holdings(ctx: &CommandContext<'_>, req: HoldingsRequest) -> anyhow::Result<()> {
    require_online(ctx.offline, "holdings")?;
    let top = req.top.clamp(1, 50);
    let ids = resolve_fund_ids(
        req.code,
        req.pick_watchlist,
        ctx.watchlist_path,
        "--code/--watchlist",
    )?;
    for id in ids {
        holdings_one(&ctx.session, id, top).await?;
    }
    Ok(())
}

async fn holdings_one(
    session: &super::context::Session<'_>,
    code: String,
    top: u32,
) -> anyhow::Result<()> {
    let (resolved_code, name) = resolve_fund_identifier(session, &code, false).await?;
    tracing::info!(code = %resolved_code, top = top, "Fetching stock holdings");
    let report = session
        .client
        .fetch_fund_stock_holdings(&resolved_code, top)
        .await
        .map_err(|e| anyhow::anyhow!("重仓股接口失败：{e}"))?;
    print_holdings(&resolved_code, &name, &map_holdings(&report));
    Ok(())
}
