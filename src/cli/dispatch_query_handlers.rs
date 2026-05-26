//! 查询类子命令 → application Request 映射。

use super::Commands;
use crate::application::{
    run_analyze, run_compare, run_export, run_fetch, run_portfolio, AnalyzeRequest, CommandContext,
    CompareRequest, ExportRequest, FetchRequest, PortfolioRequest,
};

pub async fn dispatch(ctx: &CommandContext<'_>, cmd: Commands) -> anyhow::Result<()> {
    match cmd {
        Commands::Fetch { .. } | Commands::Analyze { .. } | Commands::Compare { .. } => {
            dispatch_fund(ctx, cmd).await
        }
        Commands::Portfolio { .. } | Commands::Export { .. } => {
            dispatch_portfolio_export(ctx, cmd).await
        }
        _ => super::dispatch_query_info::dispatch(ctx, cmd).await,
    }
}

async fn dispatch_fund(ctx: &CommandContext<'_>, cmd: Commands) -> anyhow::Result<()> {
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
            output,
            format,
        } => {
            run_analyze(
                ctx,
                AnalyzeRequest {
                    code,
                    pick_watchlist,
                    days,
                    period,
                    output,
                    format,
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
        _ => unreachable!("dispatch_fund"),
    }
}

async fn dispatch_portfolio_export(ctx: &CommandContext<'_>, cmd: Commands) -> anyhow::Result<()> {
    match cmd {
        Commands::Portfolio {
            portfolio_file,
            days,
            period,
            holdings_top,
            output,
            format,
        } => {
            run_portfolio(
                ctx,
                PortfolioRequest {
                    portfolio_path: portfolio_file,
                    days,
                    period,
                    holdings_top,
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
        _ => unreachable!("dispatch_portfolio_export"),
    }
}
