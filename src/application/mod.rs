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

pub use analyze::{run_analyze, AnalyzeRequest};
pub use brief::{gather_brief, run_brief, BriefRequest};
pub use compare::{gather_compare_analyses, run_compare, sort_compare_analyses, CompareRequest};
pub use context::{require_online, CommandContext, FundRepository, Session, StructuredOutput};
pub use data_source::FundDataSource;
pub use export::{run_export, ExportRequest};
pub use output_profile::OutputProfile;
pub use portfolio::{
    gather_portfolio_report, run_portfolio, PortfolioGatherRequest, PortfolioRequest,
};
pub use portfolio_config::{run_portfolio_config, PortfolioConfigPayload, PortfolioHoldingItem};
pub use queries::{
    load_fund_holdings, load_fund_holdings_resolved, load_fund_overview,
    load_fund_overview_resolved, load_sectors_resolved, run_fetch, run_holdings, run_info,
    run_rank, run_sectors, FetchRequest, HoldingsRequest, InfoRequest, RankRequest, SectorsRequest,
};
pub use research_fund::{
    gather_fund_research_io, gather_research_fund, FundResearchIo, ResearchFundResult,
};
pub use screen::{run_screen, ScreenRequest};
pub use watchlist_ops::{
    run_watchlist_add, run_watchlist_list, run_watchlist_remove, WatchlistItem,
};

pub use fund_service::{
    analyze_fund, analyze_fund_with_navs, fetch_nav_series, get_benchmark_data, get_fund_meta,
    resolve_fund_identifier,
};
