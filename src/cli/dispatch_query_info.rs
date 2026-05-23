//! 数据查询类子命令分派（info / rank / sectors / holdings）。

use super::Commands;
use crate::application::{
    run_holdings, run_info, run_rank, run_sectors, CommandContext, HoldingsRequest, InfoRequest,
    RankRequest, SectorsRequest,
};

pub async fn dispatch(ctx: &CommandContext<'_>, cmd: Commands) -> anyhow::Result<()> {
    match cmd {
        Commands::Info {
            code,
            pick_watchlist,
        } => {
            run_info(
                ctx,
                InfoRequest {
                    code,
                    pick_watchlist,
                },
            )
            .await
        }
        Commands::Rank { kind, top, sort } => run_rank(ctx, RankRequest { kind, top, sort }).await,
        Commands::Sectors {
            code,
            pick_watchlist,
        } => {
            run_sectors(
                ctx,
                SectorsRequest {
                    code,
                    pick_watchlist,
                },
            )
            .await
        }
        Commands::Holdings {
            code,
            pick_watchlist,
            top,
        } => {
            run_holdings(
                ctx,
                HoldingsRequest {
                    code,
                    pick_watchlist,
                    top,
                },
            )
            .await
        }
        _ => anyhow::bail!("unexpected command for dispatch_query_info"),
    }
}
