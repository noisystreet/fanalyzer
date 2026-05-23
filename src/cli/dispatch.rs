//! 子命令分派（CLI 薄层 → application 用例）。

use super::{Cli, Commands};
use crate::api::eastmoney::EastMoneyClient;
use crate::application::CommandContext;
use crate::cache::FundCache;
use crate::nav_cache::NavCache;
use clap::Parser;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn dispatch(
    mut cli: Cli,
    client: &EastMoneyClient,
    name_cache: &Arc<Mutex<FundCache>>,
    nav_store: &NavCache,
) -> anyhow::Result<()> {
    let Some(cmd) = cli.command.take() else {
        Cli::parse_from(["analysis_fund", "--help"]);
        return Ok(());
    };

    let ctx = CommandContext::new(
        client,
        name_cache,
        nav_store,
        cli.offline,
        &cli.watchlist_file,
    );

    match cmd {
        Commands::Brief { .. } | Commands::Screen { .. } => {
            super::dispatch_workflow::dispatch(&ctx, cmd).await
        }
        other => super::dispatch_query::dispatch(&ctx, other).await,
    }
}
