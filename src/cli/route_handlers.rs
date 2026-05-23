//! 既有子命令分派（fetch / analyze / compare 等）。

use super::analyze::AnalyzeOpts;
use super::compare::{run_compare, CompareOpts};
use super::{Cli, Commands};
use crate::api::eastmoney::EastMoneyClient;
use crate::cache::FundCache;
use crate::cli::handlers::{
    identifiers_many_or_watchlist, run_analyze, run_export_all, run_fetch, run_holdings, run_info,
    run_rank, run_sectors, ExportInvocation,
};
use crate::nav_cache::NavCache;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn dispatch(
    cmd: Commands,
    cli: &Cli,
    client: &EastMoneyClient,
    cache: &Arc<Mutex<FundCache>>,
    nav_store: &NavCache,
) -> anyhow::Result<()> {
    match cmd {
        Commands::Fetch {
            code,
            pick_watchlist,
            limit,
        } => run_fetch(cli, client, cache, code, pick_watchlist, limit).await,
        Commands::Analyze {
            code,
            pick_watchlist,
            days,
            period,
        } => {
            run_analyze(
                cli,
                client,
                cache,
                nav_store,
                AnalyzeOpts {
                    code,
                    pick_watchlist,
                    days,
                    period,
                },
            )
            .await
        }
        Commands::Compare {
            codes,
            pick_watchlist,
            days,
            period,
            sort,
            output,
            format,
        } => {
            let ids = identifiers_many_or_watchlist(codes, pick_watchlist, &cli.watchlist_file)?;
            run_compare(
                cli,
                client,
                cache,
                nav_store,
                CompareOpts {
                    codes: ids,
                    days,
                    period,
                    sort,
                    output,
                    format,
                },
            )
            .await
        }
        Commands::Export {
            code,
            pick_watchlist,
            days,
            output,
            output_dir,
            format,
        } => {
            let export = ExportInvocation {
                code,
                pick_watchlist,
                days,
                output,
                output_dir,
                format,
            };
            run_export_all(cli, client, cache, nav_store, export).await
        }
        Commands::Info {
            code,
            pick_watchlist,
        } => run_info(cli, client, cache, code, pick_watchlist).await,
        Commands::Rank { kind, top, sort } => run_rank(cli, client, kind, top, sort).await,
        Commands::Sectors {
            code,
            pick_watchlist,
        } => run_sectors(cli, client, cache, code, pick_watchlist).await,
        Commands::Holdings {
            code,
            pick_watchlist,
            top,
        } => run_holdings(cli, client, cache, code, pick_watchlist, top).await,
        Commands::Brief { .. } | Commands::Screen { .. } => {
            unreachable!("workflow commands handled in route.rs")
        }
    }
}
