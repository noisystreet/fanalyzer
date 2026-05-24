//! 子命令分派（CLI 薄层 → application 用例）。

use super::Commands;
use crate::api::eastmoney::EastMoneyClient;
use crate::application::CommandContext;
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
) -> anyhow::Result<()> {
    let ctx = CommandContext::new(client, name_cache, nav_store, offline, watchlist_path);

    match cmd {
        Commands::Brief { .. } | Commands::Screen { .. } => {
            super::dispatch_workflow::dispatch(&ctx, cmd).await
        }
        Commands::Serve { .. } => unreachable!("serve handled in cli::run"),
        other => super::dispatch_query::dispatch_core(&ctx, other).await,
    }
}
