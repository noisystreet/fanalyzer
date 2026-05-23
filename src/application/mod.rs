//! 应用层：用例编排（无 Clap 依赖）。

mod analyze;
mod brief;
mod compare;
pub mod context;
mod export;
mod fund_service;
mod mappers;
mod queries;
mod screen;

pub use analyze::{run_analyze, AnalyzeRequest};
pub use brief::{run_brief, BriefRequest};
pub use compare::{run_compare, CompareRequest};
pub use context::{require_online, CommandContext, FundRepository, Session};
pub use export::{run_export, ExportRequest};
pub use queries::{
    run_fetch, run_holdings, run_info, run_rank, run_sectors, FetchRequest, HoldingsRequest,
    InfoRequest, RankRequest, SectorsRequest,
};
pub use screen::{run_screen, ScreenRequest};

pub use fund_service::{
    analyze_fund, fetch_nav_series, get_benchmark_data, get_fund_meta, resolve_fund_identifier,
};
