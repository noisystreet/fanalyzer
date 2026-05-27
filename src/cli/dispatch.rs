//! 子命令分派（CLI 薄层 → application 用例）。

use super::Commands;
use crate::api::eastmoney::EastMoneyClient;
use crate::application::{CommandContext, StructuredOutput};
use crate::cache::FundCache;
use crate::nav_cache::NavCache;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn dispatch_with_command(
    cmd: Commands,
    client: &EastMoneyClient,
    name_cache: &Arc<Mutex<FundCache>>,
    nav_store: &NavCache,
    offline: bool,
    watchlist_path: &Path,
    structured_output: StructuredOutput,
) -> anyhow::Result<()> {
    let ctx = CommandContext::new(
        client,
        name_cache,
        nav_store,
        offline,
        watchlist_path,
        structured_output,
    );

    match cmd {
        Commands::Brief { .. } | Commands::Screen { .. } => {
            super::dispatch_workflow::dispatch(&ctx, cmd).await
        }
        Commands::Json { .. } => unreachable!("json handled in cli::run"),
        Commands::Serve { .. } => unreachable!("serve handled in cli::run"),
        Commands::Schema { .. } | Commands::Mcp { .. } => {
            unreachable!("schema/mcp handled in cli::run")
        }
        Commands::WatchlistList
        | Commands::WatchlistAdd { .. }
        | Commands::WatchlistRemove { .. }
        | Commands::PortfolioConfig { .. } => super::dispatch_agent::dispatch(&ctx, cmd).await,
        other => super::dispatch_query::dispatch_core(&ctx, other).await,
    }
}
