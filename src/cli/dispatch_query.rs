//! 数据查询类子命令分派（fetch / analyze / compare / export）。

use super::Commands;
use crate::application::{
    run_analyze, run_compare, run_export, run_fetch, AnalyzeRequest, CommandContext,
    CompareRequest, ExportRequest, FetchRequest,
};

pub async fn dispatch_core(ctx: &CommandContext<'_>, cmd: Commands) -> anyhow::Result<()> {
    match cmd {
        Commands::Fetch {
            code,
            pick_watchlist,
            limit,
        } => {
            run_fetch(
                ctx,
                FetchRequest {
                    code,
                    pick_watchlist,
                    limit,
                },
            )
            .await
        }
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
            run_compare(
                ctx,
                CompareRequest {
                    codes,
                    pick_watchlist,
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
        _ => super::dispatch_query_info::dispatch(ctx, cmd).await,
    }
}
