//! Agent 专用子命令分派（自选 / 组合配置）。

use super::Commands;
use crate::application::{
    CommandContext, run_portfolio_config, run_watchlist_add, run_watchlist_list,
    run_watchlist_remove,
};
use std::path::PathBuf;

pub async fn dispatch(ctx: &CommandContext<'_>, cmd: Commands) -> anyhow::Result<()> {
    match cmd {
        Commands::WatchlistList => run_watchlist_list(ctx).await,
        Commands::WatchlistAdd { codes } => run_watchlist_add(ctx, codes).await,
        Commands::WatchlistRemove { codes } => run_watchlist_remove(ctx, codes).await,
        Commands::PortfolioConfig { portfolio_file } => {
            let path: PathBuf =
                portfolio_file.unwrap_or_else(crate::portfolio::default_portfolio_path);
            run_portfolio_config(ctx, path).await
        }
        _ => unreachable!("agent dispatch called with wrong command"),
    }
}
