//! 兼容 re-export；分析逻辑已迁至 `domain`。

pub use crate::domain::{
    resolve_benchmark, AnalysisSortKey, BenchmarkData, FundAnalyzer, FundMetaInfo, IndexBenchmark,
    HS300,
};
