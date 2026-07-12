//! 领域层：纯计算与规则（无 IO）。

mod analyzer;
mod benchmark;
mod overlap;
mod period;
mod portfolio_insights;
mod rank_kind;
mod returns;
mod rolling;
mod screen_filter;
mod sort;
mod types;

pub use analyzer::FundAnalyzer;
pub use benchmark::{HS300, IndexBenchmark, resolve_benchmark};
pub use overlap::weighted_holdings_overlap;
pub use period::{days_for_rank_sort, resolve_analysis_days};
pub use portfolio_insights::{EqualWeightComparison, interpret_portfolio};
pub use rank_kind::rank_ft_code;
pub use returns::{
    PortfolioMetrics, align_daily_returns, correlation_matrix, daily_returns,
    metrics_from_daily_returns, weighted_portfolio_returns,
};
pub use rolling::{
    DEFAULT_ROLLING_WINDOW, build_fund_analysis_series, build_portfolio_series,
    normalize_rolling_window,
};
pub use screen_filter::{ScreenFilters, passes_screen};
pub use sort::{AnalysisSortKey, parse_sort_key, sort_analyses};
pub use types::{BenchmarkData, FundMetaInfo};
