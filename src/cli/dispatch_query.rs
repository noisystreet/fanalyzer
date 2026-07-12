//! 数据查询类子命令分派（fetch / analyze / compare / export / portfolio）。

use super::Commands;
use super::dispatch_query_handlers;
use crate::application::CommandContext;

pub async fn dispatch_core(ctx: &CommandContext<'_>, cmd: Commands) -> anyhow::Result<()> {
    dispatch_query_handlers::dispatch(ctx, cmd).await
}
