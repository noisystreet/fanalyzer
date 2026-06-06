//! fetch / info / rank / sectors / holdings 查询用例。

use super::concurrency::{map_concurrent, FUND_CONCURRENCY};
use super::context::{require_online, resolve_fund_ids, CommandContext};
use super::fund_service::resolve_fund_identifier;
use super::mappers::{map_holdings, map_industry, map_profile, map_rank_rows};
use crate::domain::rank_ft_code;
use crate::presentation::{
    base_meta, emit, item_error_failed, print_fetch_result, print_fund_overview, print_holdings,
    print_industry, print_ranking_table, BatchMeta, BatchPayload, FetchPayload, HoldingsItem,
    RankMeta, RankPayload, SectorItem,
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

async fn collect_batch<T, F, Fut>(
    ctx: &CommandContext<'_>,
    ids: Vec<String>,
    op: F,
) -> anyhow::Result<(Vec<T>, Vec<crate::presentation::ItemError>)>
where
    F: Fn(String) -> Fut,
    Fut: std::future::Future<Output = anyhow::Result<T>>,
{
    let requested = ids.len();
    if ctx.structured() {
        let results = map_concurrent(&ids, FUND_CONCURRENCY, |id| async {
            let result = op(id.clone()).await;
            (id, result)
        })
        .await;
        let mut items = Vec::with_capacity(requested);
        let mut errors = Vec::new();
        for (id, result) in results {
            match result {
                Ok(value) => items.push(value),
                Err(e) => errors.push(item_error_failed(&id, e)),
            }
        }
        if items.is_empty() {
            anyhow::bail!("全部条目处理失败");
        }
        return Ok((items, errors));
    }

    let mut items = Vec::with_capacity(requested);
    for id in ids {
        match op(id.clone()).await {
            Ok(value) => items.push(value),
            Err(e) => return Err(e),
        }
    }
    Ok((items, Vec::new()))
}

fn batch_meta(ctx: &CommandContext<'_>, requested: usize, succeeded: usize) -> BatchMeta {
    BatchMeta {
        base: base_meta(ctx),
        requested,
        succeeded,
    }
}

fn warn_partial(ctx: &CommandContext<'_>, errors: &[crate::presentation::ItemError]) {
    if !errors.is_empty() {
        ctx.warn(format!("{} 只条目处理失败", errors.len()));
    }
}

pub async fn run_fetch(ctx: &CommandContext<'_>, req: FetchRequest) -> anyhow::Result<()> {
    require_online(ctx.offline, "fetch")?;
    let ids = resolve_fund_ids(
        req.code,
        req.pick_watchlist,
        ctx.watchlist_path,
        "--code/--watchlist",
    )?;
    let requested = ids.len();
    let limit = req.limit;
    let (items, errors) = collect_batch(ctx, ids, |id| {
        fetch_one(&ctx.session, id, limit, ctx.structured())
    })
    .await?;
    if ctx.structured() {
        warn_partial(ctx, &errors);
        let succeeded = items.len();
        emit(
            ctx,
            "fetch",
            &BatchPayload { items, errors },
            Some(&batch_meta(ctx, requested, succeeded)),
            None,
        )?;
    }
    Ok(())
}

async fn fetch_one(
    session: &super::context::Session<'_>,
    code: String,
    limit: u32,
    structured: bool,
) -> anyhow::Result<FetchPayload> {
    let (resolved_code, name) = resolve_fund_identifier(session, &code, false).await?;
    tracing::info!(code = %resolved_code, name = %name, limit = limit, "Fetching fund nav history");
    let (nav_list, total) = session
        .source
        .fetch_nav_history(&resolved_code, 1, limit)
        .await
        .map_err(|e| anyhow::anyhow!("拉取净值失败：{e}"))?;
    tracing::info!(total = total, fetched = nav_list.len(), "Fetched nav data");
    if !structured {
        print_fetch_result(&resolved_code, &name, &nav_list, total);
    }
    Ok(FetchPayload {
        code: resolved_code,
        name,
        total,
        fetched: nav_list.len(),
        nav: nav_list,
    })
}

pub async fn load_fund_overview(
    session: &super::context::Session<'_>,
    code: &str,
) -> anyhow::Result<crate::models::FundOverview> {
    let (resolved_code, _name) = resolve_fund_identifier(session, code, false).await?;
    load_fund_overview_resolved(session, &resolved_code).await
}

pub async fn load_fund_overview_resolved(
    session: &super::context::Session<'_>,
    resolved_code: &str,
) -> anyhow::Result<crate::models::FundOverview> {
    tracing::info!(code = %resolved_code, "Fetching fund info");
    let profile = session
        .source
        .fetch_fund_profile(resolved_code)
        .await
        .map_err(|e| anyhow::anyhow!("获取基金概况失败：{e}"))?;
    Ok(map_profile(&profile))
}

pub async fn load_fund_holdings(
    session: &super::context::Session<'_>,
    code: &str,
    top: u32,
) -> anyhow::Result<crate::models::StockHoldings> {
    let (resolved_code, name) = resolve_fund_identifier(session, code, false).await?;
    let item = load_fund_holdings_resolved(session, &resolved_code, &name, top).await?;
    Ok(item.holdings)
}

pub async fn load_fund_holdings_resolved(
    session: &super::context::Session<'_>,
    resolved_code: &str,
    name: &str,
    top: u32,
) -> anyhow::Result<HoldingsItem> {
    let top = top.clamp(1, 50);
    tracing::info!(code = %resolved_code, top = top, "Fetching stock holdings");
    let report = session
        .source
        .fetch_fund_stock_holdings(resolved_code, top)
        .await
        .map_err(|e| anyhow::anyhow!("重仓股接口失败：{e}"))?;
    Ok(HoldingsItem {
        code: resolved_code.to_string(),
        name: name.to_string(),
        holdings: map_holdings(&report),
    })
}

pub async fn load_sectors_resolved(
    session: &super::context::Session<'_>,
    resolved_code: &str,
    name: &str,
) -> anyhow::Result<SectorItem> {
    tracing::info!(code = %resolved_code, "Fetching industry allocation");
    let report = session
        .source
        .fetch_fund_industry_allocation(resolved_code)
        .await
        .map_err(|e| anyhow::anyhow!("行业配置获取失败：{e}"))?;
    Ok(SectorItem {
        code: resolved_code.to_string(),
        name: name.to_string(),
        industry: map_industry(&report),
    })
}

pub async fn run_info(ctx: &CommandContext<'_>, req: InfoRequest) -> anyhow::Result<()> {
    require_online(ctx.offline, "info")?;
    let ids = resolve_fund_ids(
        req.code,
        req.pick_watchlist,
        ctx.watchlist_path,
        "--code/--watchlist",
    )?;
    let requested = ids.len();
    let (items, errors) =
        collect_batch(ctx, ids, |id| info_one(&ctx.session, id, ctx.structured())).await?;
    if ctx.structured() {
        warn_partial(ctx, &errors);
        let succeeded = items.len();
        emit(
            ctx,
            "info",
            &BatchPayload { items, errors },
            Some(&batch_meta(ctx, requested, succeeded)),
            None,
        )?;
    }
    Ok(())
}

async fn info_one(
    session: &super::context::Session<'_>,
    code: String,
    structured: bool,
) -> anyhow::Result<crate::models::FundOverview> {
    let profile = load_fund_overview(session, &code).await?;
    if !structured {
        print_fund_overview(&profile);
    }
    Ok(profile)
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
        .source
        .fetch_fund_ranking_top(ft, sc, req.top)
        .await?;
    let rows = map_rank_rows(&page.rows);
    if ctx.structured() {
        let payload = RankPayload {
            kind: req.kind.clone(),
            sort: sc.to_string(),
            total_records: page.total_records,
            rows,
        };
        let meta = RankMeta {
            base: base_meta(ctx),
            kind: req.kind,
            sort: sc.to_string(),
            top: req.top,
        };
        emit(ctx, "rank", &payload, Some(&meta), None)?;
    } else {
        print_ranking_table(&rows, ft, sc, page.total_records);
    }
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
    let requested = ids.len();
    let (items, errors) = collect_batch(ctx, ids, |id| {
        sectors_one(&ctx.session, id, ctx.structured())
    })
    .await?;
    if ctx.structured() {
        warn_partial(ctx, &errors);
        let succeeded = items.len();
        emit(
            ctx,
            "sectors",
            &BatchPayload { items, errors },
            Some(&batch_meta(ctx, requested, succeeded)),
            None,
        )?;
    }
    Ok(())
}

async fn sectors_one(
    session: &super::context::Session<'_>,
    code: String,
    structured: bool,
) -> anyhow::Result<SectorItem> {
    let (resolved_code, name) = resolve_fund_identifier(session, &code, false).await?;
    let item = load_sectors_resolved(session, &resolved_code, &name).await?;
    if !structured {
        print_industry(&resolved_code, &name, &item.industry);
    }
    Ok(item)
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
    let requested = ids.len();
    let (items, errors) = collect_batch(ctx, ids, |id| {
        holdings_one(&ctx.session, id, top, ctx.structured())
    })
    .await?;
    if ctx.structured() {
        warn_partial(ctx, &errors);
        let succeeded = items.len();
        emit(
            ctx,
            "holdings",
            &BatchPayload { items, errors },
            Some(&batch_meta(ctx, requested, succeeded)),
            None,
        )?;
    }
    Ok(())
}

async fn holdings_one(
    session: &super::context::Session<'_>,
    code: String,
    top: u32,
    structured: bool,
) -> anyhow::Result<HoldingsItem> {
    let (resolved_code, name) = resolve_fund_identifier(session, &code, false).await?;
    let item = load_fund_holdings_resolved(session, &resolved_code, &name, top).await?;
    if !structured {
        print_holdings(&resolved_code, &name, &item.holdings);
    }
    Ok(item)
}
