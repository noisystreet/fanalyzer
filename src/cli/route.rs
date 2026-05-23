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
            period,
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
                    period,
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
            period,
            min_rank_return,
            max_drawdown,
            min_sharpe,
            max_mgmt_fee,
            min_alpha,
            max_volatility,
            min_total_return,
            deep_limit,
            full_scan,
            sort_by,
            limit,
            output,
            format,
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
                    period,
                    filters: screen::ScreenFilters {
                        min_rank_return_pct: min_rank_return,
                        max_drawdown_pct: max_drawdown,
                        min_sharpe,
                        max_mgmt_fee_pct: max_mgmt_fee,
                        min_alpha_pct: min_alpha,
                        max_volatility_pct: max_volatility,
                        min_total_return_pct: min_total_return,
                    },
                    deep_limit,
                    full_scan,
                    sort_by,
                    limit,
                    output,
                    format,
                },
            )
            .await
        }
        other => route_handlers::dispatch(other, cli, client, cache, nav_store).await,
    }
}
