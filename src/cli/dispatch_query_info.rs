//! 数据查询类子命令分派（info / rank / sectors / holdings）。

use super::Commands;
use crate::application::{
    CommandContext, HoldingsRequest, InfoRequest, RankRequest, SectorsRequest, run_holdings,
    run_info, run_rank, run_sectors,
};

pub async fn dispatch(ctx: &CommandContext<'_>, cmd: Commands) -> anyhow::Result<()> {
    match cmd {
        Commands::Info {
            fund_code,
            pick_watchlist,
        } => {
            run_info(
                ctx,
                InfoRequest {
                    code: fund_code.resolve()?,
                    pick_watchlist,
                },
            )
            .await
        }
        Commands::Rank { kind, top, sort } => run_rank(ctx, RankRequest { kind, top, sort }).await,
        Commands::Sectors {
            fund_code,
            pick_watchlist,
        } => {
            run_sectors(
                ctx,
                SectorsRequest {
                    code: fund_code.resolve()?,
                    pick_watchlist,
                },
            )
            .await
        }
        Commands::Holdings {
            fund_code,
            pick_watchlist,
            top,
        } => {
            run_holdings(
                ctx,
                HoldingsRequest {
                    code: fund_code.resolve()?,
                    pick_watchlist,
                    top,
                },
            )
            .await
        }
        _ => anyhow::bail!("unexpected command for dispatch_query_info"),
    }
}
