//! 选基工作流子命令分派。

use super::Commands;
use crate::application::{run_brief, run_screen, BriefRequest, CommandContext, ScreenRequest};
use crate::domain::ScreenFilters;

pub async fn dispatch(ctx: &CommandContext<'_>, cmd: Commands) -> anyhow::Result<()> {
    match cmd {
        Commands::Brief {
            code,
            pick_watchlist,
            days,
            period,
            industry_top,
            holdings_top,
            output,
        } => {
            run_brief(
                ctx,
                BriefRequest {
                    code,
                    pick_watchlist,
                    days,
                    period,
                    industry_top,
                    holdings_top,
                    output,
                },
            )
            .await
        }
        Commands::Screen {
            kind,
            sort,
            rank_top,
            days,
            period,
            min_rank_return,
            max_drawdown,
            min_sharpe,
            max_mgmt_fee,
            min_alpha,
            max_volatility,
            min_total_return,
            deep_limit,
            full_scan,
            sort_by,
            limit,
            output,
            format,
        } => {
            run_screen(
                ctx,
                ScreenRequest {
                    kind,
                    sort,
                    rank_top,
                    days,
                    period,
                    filters: ScreenFilters {
                        min_rank_return_pct: min_rank_return,
                        max_drawdown_pct: max_drawdown,
                        min_sharpe,
                        max_mgmt_fee_pct: max_mgmt_fee,
                        min_alpha_pct: min_alpha,
                        max_volatility_pct: max_volatility,
                        min_total_return_pct: min_total_return,
                    },
                    deep_limit,
                    full_scan,
                    sort_by,
                    limit,
                    output,
                    format,
                },
            )
            .await
        }
        _ => unreachable!("only Brief/Screen in dispatch_workflow"),
    }
}
