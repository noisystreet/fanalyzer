//! fetch / info / rank / sectors / holdings 查询用例。

use super::context::{require_online, resolve_fund_ids, CommandContext};
use super::fund_service::resolve_fund_identifier;
use crate::domain::rank_ft_code;
use crate::presentation::{
    print_fund_profile, print_holdings_report, print_industry_report, print_ranking_table,
};

pub async fn run_fetch(
    ctx: &CommandContext<'_>,
    code: Option<String>,
    pick_watchlist: bool,
    limit: u32,
) -> anyhow::Result<()> {
    require_online(ctx.offline, "fetch")?;
    let ids = resolve_fund_ids(
        code,
        pick_watchlist,
        ctx.watchlist_path,
        "--code/--watchlist",
    )?;
    for id in ids {
        fetch_one(&ctx.session, id, limit).await?;
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
            println!(
                "Fetched {} records (total: {}) for fund {} ({})",
                nav_list.len(),
                total,
                resolved_code,
                name
            );
            for nav in &nav_list {
                println!(
                    "  {}  NAV: {:.4}  AccNAV: {:.4}  DailyReturn: {}",
                    nav.date,
                    nav.nav,
                    nav.acc_nav,
                    nav.daily_return
                        .map(|r| format!("{:.2}%", r * 100.0))
                        .unwrap_or_else(|| "N/A".to_string())
                );
            }
        }
        Err(e) => tracing::error!(error = %e, "Failed to fetch nav history"),
    }
    Ok(())
}

pub async fn run_info(
    ctx: &CommandContext<'_>,
    code: Option<String>,
    pick_watchlist: bool,
) -> anyhow::Result<()> {
    require_online(ctx.offline, "info")?;
    if pick_watchlist {
        let ids = crate::watchlist::load_watchlist(ctx.watchlist_path)?;
        if ids.is_empty() {
            anyhow::bail!("自选列表为空或无有效项：{}", ctx.watchlist_path.display());
        }
        for id in ids {
            info_one(&ctx.session, id).await?;
        }
    } else {
        let c = code.ok_or_else(|| anyhow::anyhow!("请指定 --code 或使用 --watchlist"))?;
        info_one(&ctx.session, c).await?;
    }
    Ok(())
}

async fn info_one(session: &super::context::Session<'_>, code: String) -> anyhow::Result<()> {
    let (resolved_code, _name) = resolve_fund_identifier(session, &code, false).await?;
    tracing::info!(code = %resolved_code, "Fetching fund info");
    match session.client.fetch_fund_profile(&resolved_code).await {
        Ok(profile) => print_fund_profile(&profile),
        Err(e) => tracing::error!(error = %e, "Failed to fetch fund info"),
    }
    Ok(())
}

pub async fn run_rank(
    ctx: &CommandContext<'_>,
    kind: String,
    top: u32,
    sort: String,
) -> anyhow::Result<()> {
    require_online(ctx.offline, "rank")?;
    if top == 0 {
        anyhow::bail!("`--top` 须 ≥ 1");
    }
    if top > 500 {
        anyhow::bail!("`--top` 上限为 500");
    }
    let ft = rank_ft_code(&kind)?;
    let sc = sort.trim();
    if sc.is_empty() {
        anyhow::bail!("`--sort` 不能为空（默认可用 1n）");
    }
    tracing::info!(ft = ft, top = top, sort = %sc, "Fetching fund ranking");
    let page = ctx
        .session
        .client
        .fetch_fund_ranking_top(ft, sc, top)
        .await?;
    print_ranking_table(&page.rows, ft, sc, page.total_records);
    Ok(())
}

pub async fn run_sectors(
    ctx: &CommandContext<'_>,
    code: Option<String>,
    pick_watchlist: bool,
) -> anyhow::Result<()> {
    require_online(ctx.offline, "sectors")?;
    let ids = resolve_fund_ids(
        code,
        pick_watchlist,
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
    print_industry_report(&resolved_code, &name, &report);
    Ok(())
}

pub async fn run_holdings(
    ctx: &CommandContext<'_>,
    code: Option<String>,
    pick_watchlist: bool,
    top: u32,
) -> anyhow::Result<()> {
    require_online(ctx.offline, "holdings")?;
    let top = top.clamp(1, 50);
    let ids = resolve_fund_ids(
        code,
        pick_watchlist,
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
    print_holdings_report(&resolved_code, &name, &report);
    Ok(())
}
