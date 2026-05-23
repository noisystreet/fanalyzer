//! 子命令分派（从 `handlers::execute` 拆出以满足行数门控）。

use super::{brief, route_handlers, screen, Cli, Commands};
use crate::api::eastmoney::EastMoneyClient;
use crate::cache::FundCache;
use crate::nav_cache::NavCache;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn route_command(
    cmd: Commands,
    cli: &Cli,
    client: &EastMoneyClient,
    cache: &Arc<Mutex<FundCache>>,
    nav_store: &NavCache,
) -> anyhow::Result<()> {
    match cmd {
        Commands::Brief {
            code,
            pick_watchlist,
            days,
            industry_top,
            holdings_top,
            output,
        } => {
            brief::run_brief(
                cli,
                client,
                cache,
                nav_store,
                brief::BriefOpts {
                    code,
                    pick_watchlist,
                    days,
                    industry_top,
                    holdings_top,
                    output,
                },
            )
            .await
        }
        Commands::Screen {
            kind,
            sort,
            rank_top,
            days,
            max_drawdown,
            min_sharpe,
            max_mgmt_fee,
            limit,
        } => {
            screen::run_screen(
                cli,
                client,
                cache,
                nav_store,
                screen::ScreenOpts {
                    kind,
                    sort,
                    rank_top,
                    days,
                    filters: screen::ScreenFilters {
                        max_drawdown_pct: max_drawdown,
                        min_sharpe,
                        max_mgmt_fee_pct: max_mgmt_fee,
                    },
                    limit,
                },
            )
            .await
        }
        other => route_handlers::dispatch(other, cli, client, cache, nav_store).await,
    }
}
