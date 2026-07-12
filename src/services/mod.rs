//! 兼容 re-export；分析逻辑已迁至 `domain`。

pub use crate::domain::{
    AnalysisSortKey, BenchmarkData, FundAnalyzer, FundMetaInfo, HS300, IndexBenchmark,
    resolve_benchmark,
};
