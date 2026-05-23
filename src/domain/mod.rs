//! 领域层：纯计算与规则（无 IO）。

mod analyzer;
mod benchmark;
mod period;
mod rank_kind;
mod screen_filter;
mod sort;
mod types;

pub use analyzer::FundAnalyzer;
pub use benchmark::{resolve_benchmark, IndexBenchmark, HS300};
pub use period::{days_for_rank_sort, resolve_analysis_days};
pub use rank_kind::rank_ft_code;
pub use screen_filter::{passes_screen, ScreenFilters};
pub use sort::{parse_sort_key, sort_analyses, AnalysisSortKey};
pub use types::{BenchmarkData, FundMetaInfo};
