//! 自选列表结构化命令。

use super::context::CommandContext;
use crate::presentation::{BatchMeta, BatchPayload, base_meta, emit};
use crate::watchlist::{add_to_watchlist, load_watchlist, remove_from_watchlist};
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Debug, Serialize, JsonSchema)]
pub struct WatchlistItem {
    pub code: String,
}

pub async fn run_watchlist_list(ctx: &CommandContext<'_>) -> anyhow::Result<()> {
    let funds = load_watchlist(ctx.watchlist_path)?;
    let items: Vec<WatchlistItem> = funds
        .into_iter()
        .map(|code| WatchlistItem { code })
        .collect();
    let meta = BatchMeta {
        base: base_meta(ctx),
        requested: items.len(),
        succeeded: items.len(),
    };
    emit(
        ctx,
        "watchlist",
        &BatchPayload {
            items,
            errors: vec![],
        },
        Some(&meta),
        None,
    )
}

pub async fn run_watchlist_add(ctx: &CommandContext<'_>, codes: Vec<String>) -> anyhow::Result<()> {
    let funds = add_to_watchlist(ctx.watchlist_path, &codes)?;
    let items: Vec<WatchlistItem> = funds
        .into_iter()
        .map(|code| WatchlistItem { code })
        .collect();
    let meta = BatchMeta {
        base: base_meta(ctx),
        requested: codes.len(),
        succeeded: items.len(),
    };
    emit(
        ctx,
        "watchlist",
        &BatchPayload {
            items,
            errors: vec![],
        },
        Some(&meta),
        None,
    )
}

pub async fn run_watchlist_remove(
    ctx: &CommandContext<'_>,
    codes: Vec<String>,
) -> anyhow::Result<()> {
    let funds = remove_from_watchlist(ctx.watchlist_path, &codes)?;
    let items: Vec<WatchlistItem> = funds
        .into_iter()
        .map(|code| WatchlistItem { code })
        .collect();
    let meta = BatchMeta {
        base: base_meta(ctx),
        requested: codes.len(),
        succeeded: items.len(),
    };
    emit(
        ctx,
        "watchlist",
        &BatchPayload {
            items,
            errors: vec![],
        },
        Some(&meta),
        None,
    )
}
