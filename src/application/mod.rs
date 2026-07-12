//! 应用层：用例编排（无 Clap 依赖）。

mod analyze;
mod brief;
mod compare;
mod concurrency;
pub mod context;
mod data_source;
mod export;
mod fund_service;
mod mappers;
mod output_profile;
mod portfolio;
mod portfolio_config;
mod queries;
mod research_fund;
mod screen;
mod watchlist_ops;

#[cfg(test)]
pub mod test_support;

pub use analyze::{AnalyzeRequest, run_analyze};
pub use brief::{BriefRequest, gather_brief, run_brief};
pub use compare::{CompareRequest, gather_compare_analyses, run_compare, sort_compare_analyses};
pub use context::{CommandContext, FundRepository, Session, StructuredOutput, require_online};
pub use data_source::FundDataSource;
pub use export::{ExportRequest, run_export};
pub use output_profile::OutputProfile;
pub use portfolio::{
    PortfolioGatherRequest, PortfolioRequest, gather_portfolio_report, run_portfolio,
};
pub use portfolio_config::{PortfolioConfigPayload, PortfolioHoldingItem, run_portfolio_config};
pub use queries::{
    FetchRequest, HoldingsRequest, InfoRequest, RankRequest, SectorsRequest, load_fund_holdings,
    load_fund_holdings_resolved, load_fund_overview, load_fund_overview_resolved,
    load_sectors_resolved, run_fetch, run_holdings, run_info, run_rank, run_sectors,
};
pub use research_fund::{
    FundResearchIo, ResearchFundResult, gather_fund_research_io, gather_research_fund,
};
pub use screen::{ScreenRequest, run_screen};
pub use watchlist_ops::{
    WatchlistItem, run_watchlist_add, run_watchlist_list, run_watchlist_remove,
};

pub use fund_service::{
    analyze_fund, analyze_fund_with_navs, fetch_nav_series, get_benchmark_data, get_fund_meta,
    resolve_fund_identifier,
};
