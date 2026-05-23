//! 数据查询类子命令分派。

use super::Commands;
use crate::application::context::resolve_many_fund_ids;
use crate::application::{
    run_analyze, run_compare, run_export, run_fetch, run_holdings, run_info, run_rank, run_sectors,
    AnalyzeRequest, CommandContext, CompareRequest, ExportRequest,
};

pub async fn dispatch(ctx: &CommandContext<'_>, cmd: Commands) -> anyhow::Result<()> {
    match cmd {
        Commands::Fetch {
            code,
            pick_watchlist,
            limit,
        } => run_fetch(ctx, code, pick_watchlist, limit).await,
        Commands::Analyze {
            code,
            pick_watchlist,
            days,
            period,
        } => {
            run_analyze(
                ctx,
                AnalyzeRequest {
                    code,
                    pick_watchlist,
                    days,
                    period,
                },
            )
            .await
        }
        Commands::Compare {
            codes,
            pick_watchlist,
            days,
            period,
            sort,
            output,
            format,
        } => {
            let ids = resolve_many_fund_ids(codes, pick_watchlist, ctx.watchlist_path)?;
            run_compare(
                ctx,
                CompareRequest {
                    codes: ids,
                    days,
                    period,
                    sort,
                    output,
                    format,
                },
            )
            .await
        }
        Commands::Export {
            code,
            pick_watchlist,
            days,
            output,
            output_dir,
            format,
        } => {
            run_export(
                ctx,
                ExportRequest {
                    code,
                    pick_watchlist,
                    days,
                    output,
                    output_dir,
                    format,
                },
            )
            .await
        }
        Commands::Info {
            code,
            pick_watchlist,
        } => run_info(ctx, code, pick_watchlist).await,
        Commands::Rank { kind, top, sort } => run_rank(ctx, kind, top, sort).await,
        Commands::Sectors {
            code,
            pick_watchlist,
        } => run_sectors(ctx, code, pick_watchlist).await,
        Commands::Holdings {
            code,
            pick_watchlist,
            top,
        } => run_holdings(ctx, code, pick_watchlist, top).await,
        Commands::Brief { .. } | Commands::Screen { .. } => {
            unreachable!("workflow commands handled in dispatch_workflow")
        }
    }
}
