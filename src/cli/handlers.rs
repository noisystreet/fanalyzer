use super::fund_session::{
    analyze_fund, fetch_nav_series, get_benchmark_data, get_fund_meta, resolve_fund_identifier,
};
use super::output::{
    export_csv, export_json, print_analysis, print_comparison, print_fund_profile,
    print_holdings_report, print_industry_report, print_ranking_table,
};
use super::rank_kind::rank_ft_code;
use super::route;
use super::Cli;
use crate::api::eastmoney::{EastMoneyClient, EastMoneyError};
use crate::cache::FundCache;
use crate::models::FundAnalysis;
use crate::nav_cache::NavCache;
use crate::services::{BenchmarkData, FundAnalyzer};
use clap::Parser;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

pub fn map_client_err(e: EastMoneyError) -> anyhow::Error {
    anyhow::Error::msg(e.to_string())
}

pub fn no_offline(offline: bool, cmd: &str) -> anyhow::Result<()> {
    if offline {
        anyhow::bail!("`{cmd}` 需要访问网络，勿使用 `--offline`");
    }
    Ok(())
}

pub(crate) fn identifiers_one_or_watchlist(
    code: Option<String>,
    pick_watchlist: bool,
    path: &Path,
    flag_hint: &str,
) -> anyhow::Result<Vec<String>> {
    if pick_watchlist {
        let v = crate::watchlist::load_watchlist(path)?;
        if v.is_empty() {
            anyhow::bail!("自选列表为空或无有效项：{}", path.display());
        }
        Ok(v)
    } else {
        let c = code.ok_or_else(|| anyhow::anyhow!("请指定 `{flag_hint}`"))?;
        Ok(vec![c])
    }
}

pub fn identifiers_many_or_watchlist(
    codes: Vec<String>,
    pick_watchlist: bool,
    path: &Path,
) -> anyhow::Result<Vec<String>> {
    if pick_watchlist {
        let v = crate::watchlist::load_watchlist(path)?;
        if v.is_empty() {
            anyhow::bail!("自选列表为空或无有效项：{}", path.display());
        }
        Ok(v)
    } else if codes.is_empty() {
        anyhow::bail!("请提供 --codes 或使用 --watchlist")
    } else {
        Ok(codes)
    }
}

pub async fn execute(
    mut cli: Cli,
    client: &EastMoneyClient,
    cache: &Arc<Mutex<FundCache>>,
    nav_store: &NavCache,
) -> anyhow::Result<()> {
    let Some(cmd) = cli.command.take() else {
        Cli::parse_from(["analysis_fund", "--help"]);
        return Ok(());
    };
    route::route_command(cmd, &cli, client, cache, nav_store).await
}

pub(crate) async fn run_fetch(
    cli: &Cli,
    client: &EastMoneyClient,
    cache: &Arc<Mutex<FundCache>>,
    code: Option<String>,
    pick_watchlist: bool,
    limit: u32,
) -> anyhow::Result<()> {
    no_offline(cli.offline, "fetch")?;
    let ids = identifiers_one_or_watchlist(
        code,
        pick_watchlist,
        &cli.watchlist_file,
        "--code/--watchlist",
    )?;
    for id in ids {
        cmd_fetch(client, cache, id, limit).await?;
    }
    Ok(())
}

pub(crate) async fn run_analyze(
    cli: &Cli,
    client: &EastMoneyClient,
    cache: &Arc<Mutex<FundCache>>,
    nav_store: &NavCache,
    code: Option<String>,
    pick_watchlist: bool,
    days: u32,
) -> anyhow::Result<()> {
    let ids = identifiers_one_or_watchlist(
        code,
        pick_watchlist,
        &cli.watchlist_file,
        "--code/--watchlist",
    )?;
    for id in ids {
        cmd_analyze(client, cache, nav_store, cli.offline, id, days).await?;
    }
    Ok(())
}

pub(crate) struct ExportInvocation {
    pub code: Option<String>,
    pub pick_watchlist: bool,
    pub days: u32,
    pub output: Option<String>,
    pub output_dir: Option<String>,
    pub format: String,
}

pub(crate) async fn run_export_all(
    cli: &Cli,
    client: &EastMoneyClient,
    cache: &Arc<Mutex<FundCache>>,
    nav_store: &NavCache,
    export: ExportInvocation,
) -> anyhow::Result<()> {
    let sess = ExportSession {
        client,
        cache,
        nav_store,
        offline: cli.offline,
        days: export.days,
    };

    if export.pick_watchlist {
        let dir = export
            .output_dir
            .ok_or_else(|| anyhow::anyhow!("自选导出需要指定 --output-dir"))?;
        let ids = crate::watchlist::load_watchlist(&cli.watchlist_file)?;
        if ids.is_empty() {
            anyhow::bail!("自选列表为空或无有效项：{}", cli.watchlist_file.display());
        }
        for id in ids {
            let (resolved_code, name) =
                resolve_fund_identifier(client, cache, &id, cli.offline).await?;
            tracing::info!(code = %resolved_code, name = %name, "Export batch");
            let path = Path::new(&dir).join(match export.format.as_str() {
                "csv" => format!("{resolved_code}.csv"),
                "json" => format!("{resolved_code}.json"),
                other => anyhow::bail!("不支持的导出格式：{other}"),
            });
            sess.export_to_path(&resolved_code, path, &export.format)
                .await?;
        }
        Ok(())
    } else {
        let oc = export
            .code
            .ok_or_else(|| anyhow::anyhow!("请指定 --code 或使用 --watchlist"))?;
        let out = export
            .output
            .ok_or_else(|| anyhow::anyhow!("单基金导出需要指定 --output"))?;
        sess.export_to_path(oc.trim(), PathBuf::from(out), &export.format)
            .await
    }
}

pub(crate) async fn run_holdings(
    cli: &Cli,
    client: &EastMoneyClient,
    cache: &Arc<Mutex<FundCache>>,
    code: Option<String>,
    pick_watchlist: bool,
    top: u32,
) -> anyhow::Result<()> {
    no_offline(cli.offline, "holdings")?;
    let top = top.clamp(1, 50);
    let ids = identifiers_one_or_watchlist(
        code,
        pick_watchlist,
        &cli.watchlist_file,
        "--code/--watchlist",
    )?;
    for id in ids {
        cmd_holdings(client, cache, id, top).await?;
    }
    Ok(())
}

async fn cmd_holdings(
    client: &EastMoneyClient,
    cache: &Arc<Mutex<FundCache>>,
    code: String,
    top: u32,
) -> anyhow::Result<()> {
    let (resolved_code, name) = resolve_fund_identifier(client, cache, &code, false).await?;
    tracing::info!(code = %resolved_code, top = top, "Fetching stock holdings");
    let report = client
        .fetch_fund_stock_holdings(&resolved_code, top)
        .await
        .map_err(|e| anyhow::anyhow!("重仓股接口失败：{e}"))?;
    print_holdings_report(&resolved_code, &name, &report);
    Ok(())
}

pub(crate) async fn run_sectors(
    cli: &Cli,
    client: &EastMoneyClient,
    cache: &Arc<Mutex<FundCache>>,
    code: Option<String>,
    pick_watchlist: bool,
) -> anyhow::Result<()> {
    no_offline(cli.offline, "sectors")?;
    let ids = identifiers_one_or_watchlist(
        code,
        pick_watchlist,
        &cli.watchlist_file,
        "--code/--watchlist",
    )?;
    for id in ids {
        cmd_sectors(client, cache, id).await?;
    }
    Ok(())
}

async fn cmd_sectors(
    client: &EastMoneyClient,
    cache: &Arc<Mutex<FundCache>>,
    code: String,
) -> anyhow::Result<()> {
    let (resolved_code, name) = resolve_fund_identifier(client, cache, &code, false).await?;
    tracing::info!(code = %resolved_code, "Fetching industry allocation");
    let report = client
        .fetch_fund_industry_allocation(&resolved_code)
        .await
        .map_err(|e| anyhow::anyhow!("行业配置获取失败：{e}"))?;
    print_industry_report(&resolved_code, &name, &report);
    Ok(())
}

pub(crate) async fn run_rank(
    cli: &Cli,
    client: &EastMoneyClient,
    kind: String,
    top: u32,
    sort: String,
) -> anyhow::Result<()> {
    no_offline(cli.offline, "rank")?;
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
    let page = client.fetch_fund_ranking_top(ft, sc, top).await?;
    print_ranking_table(&page.rows, ft, sc, page.total_records);
    Ok(())
}

pub(crate) async fn run_info(
    cli: &Cli,
    client: &EastMoneyClient,
    cache: &Arc<Mutex<FundCache>>,
    code: Option<String>,
    pick_watchlist: bool,
) -> anyhow::Result<()> {
    no_offline(cli.offline, "info")?;
    if pick_watchlist {
        let ids = crate::watchlist::load_watchlist(&cli.watchlist_file)?;
        if ids.is_empty() {
            anyhow::bail!("自选列表为空或无有效项：{}", cli.watchlist_file.display());
        }
        for id in ids {
            cmd_info(client, cache, id).await?;
        }
    } else {
        let c = code.ok_or_else(|| anyhow::anyhow!("请指定 --code 或使用 --watchlist"))?;
        cmd_info(client, cache, c).await?;
    }
    Ok(())
}

struct ExportSession<'a> {
    client: &'a EastMoneyClient,
    cache: &'a Arc<Mutex<FundCache>>,
    nav_store: &'a NavCache,
    offline: bool,
    days: u32,
}

impl ExportSession<'_> {
    async fn export_to_path(
        &self,
        identifier: &str,
        output: PathBuf,
        format: &str,
    ) -> anyhow::Result<()> {
        let (resolved_code, name) =
            resolve_fund_identifier(self.client, self.cache, identifier, self.offline).await?;
        tracing::info!(code = %resolved_code, name = %name, days = self.days, output = ?output, "Export");

        let navs = fetch_nav_series(
            self.client,
            self.nav_store,
            &resolved_code,
            self.days,
            self.offline,
        )
        .await?;

        if navs.is_empty() {
            tracing::warn!("No nav data {}", resolved_code);
            return Ok(());
        }
        let out_str = output.to_string_lossy();
        match format {
            "csv" => {
                export_csv(&navs, out_str.as_ref())?;
                tracing::info!(path = %out_str, "Exported CSV");
            }
            "json" => {
                export_json(&navs, out_str.as_ref())?;
                tracing::info!(path = %out_str, "Exported JSON");
            }
            other => anyhow::bail!("Unsupported format `{other}`"),
        }

        Ok(())
    }
}

async fn cmd_fetch(
    client: &EastMoneyClient,
    cache: &Arc<Mutex<FundCache>>,
    code: String,
    limit: u32,
) -> anyhow::Result<()> {
    let (resolved_code, name) = resolve_fund_identifier(client, cache, &code, false).await?;
    tracing::info!(code = %resolved_code, name = %name, limit = limit, "Fetching fund nav history");
    match client.fetch_nav_history(&resolved_code, 1, limit).await {
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
        Err(e) => {
            tracing::error!(error = %e, "Failed to fetch nav history");
        }
    }
    Ok(())
}

async fn cmd_analyze(
    client: &EastMoneyClient,
    cache: &Arc<Mutex<FundCache>>,
    nav_store: &NavCache,
    offline: bool,
    code: String,
    days: u32,
) -> anyhow::Result<()> {
    tracing::info!(code = %code, days = days, "Analyzing fund");
    match analyze_fund(client, cache, nav_store, &code, days, offline).await {
        Ok(Some(analysis)) => print_analysis(&analysis),
        Ok(None) => tracing::warn!("Insufficient data for analysis"),
        Err(e) => {
            tracing::error!(error = %e, "Failed to analyze");
            return Err(e);
        }
    }
    Ok(())
}

struct CompareProbe<'a> {
    client: &'a EastMoneyClient,
    cache: &'a Arc<Mutex<FundCache>>,
    nav_store: &'a NavCache,
    offline: bool,
    days: u32,
    benchmark: Option<&'a BenchmarkData>,
}

async fn try_push_compare_analysis(
    analyses: &mut Vec<FundAnalysis>,
    identifier: &str,
    ctx: &CompareProbe<'_>,
) -> anyhow::Result<()> {
    let (resolved_code, name) =
        match resolve_fund_identifier(ctx.client, ctx.cache, identifier, ctx.offline).await {
            Ok(p) => p,
            Err(e) => {
                tracing::warn!(identifier = identifier, error = %e, "跳过该标的");
                return Ok(());
            }
        };

    let meta = if ctx.offline {
        None
    } else {
        get_fund_meta(ctx.client, &resolved_code).await
    };

    let navs = fetch_nav_series(
        ctx.client,
        ctx.nav_store,
        &resolved_code,
        ctx.days,
        ctx.offline,
    )
    .await?;

    if let Some(a) = FundAnalyzer::analyze(&navs, ctx.days, &name, ctx.benchmark, meta.as_ref()) {
        analyses.push(a);
    } else {
        tracing::warn!("Insufficient data for fund {}", resolved_code);
    }
    Ok(())
}

pub(crate) async fn cmd_compare(
    client: &EastMoneyClient,
    cache: &Arc<Mutex<FundCache>>,
    nav_store: &NavCache,
    offline: bool,
    codes: Vec<String>,
    days: u32,
) -> anyhow::Result<()> {
    if codes.len() < 2 {
        anyhow::bail!("对比至少需要 2 只基金（当前 {} 条）", codes.len());
    }
    tracing::info!(codes = ?codes, days = days, "Comparing funds");
    let benchmark_holder = if offline {
        None
    } else {
        get_benchmark_data(client, days).await
    };

    let ctx = CompareProbe {
        client,
        cache,
        nav_store,
        offline,
        days,
        benchmark: benchmark_holder.as_ref(),
    };

    let mut analyses = Vec::new();
    for identifier in codes {
        try_push_compare_analysis(&mut analyses, &identifier, &ctx).await?;
    }

    if analyses.len() >= 2 {
        print_comparison(&analyses);
    } else {
        tracing::warn!("有效样本不足（需要≥2）；请检查离线缓存或数据源");
    }
    Ok(())
}

async fn cmd_info(
    client: &EastMoneyClient,
    cache: &Arc<Mutex<FundCache>>,
    code: String,
) -> anyhow::Result<()> {
    let (resolved_code, _name) = resolve_fund_identifier(client, cache, &code, false).await?;
    tracing::info!(code = %resolved_code, "Fetching fund info");
    match client.fetch_fund_profile(&resolved_code).await {
        Ok(profile) => print_fund_profile(&profile),
        Err(e) => {
            tracing::error!(error = %e, "Failed to fetch fund info");
        }
    }
    Ok(())
}
