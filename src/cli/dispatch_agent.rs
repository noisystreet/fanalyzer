//! Agent 专用子命令分派（自选 / 组合配置）。

use super::Commands;
use crate::application::{
    run_portfolio_config, run_watchlist_add, run_watchlist_list, run_watchlist_remove,
    CommandContext,
};

pub async fn dispatch(ctx: &CommandContext<'_>, cmd: Commands) -> anyhow::Result<()> {
    match cmd {
        Commands::WatchlistList => run_watchlist_list(ctx).await,
        Commands::WatchlistAdd { codes } => run_watchlist_add(ctx, codes).await,
        Commands::WatchlistRemove { codes } => run_watchlist_remove(ctx, codes).await,
        Commands::PortfolioConfig { portfolio_file } => {
            run_portfolio_config(ctx, portfolio_file).await
        }
        _ => unreachable!("agent dispatch called with wrong command"),
    }
}
