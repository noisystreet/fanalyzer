//! 分析服务：基准数据、元信息、净值分析。

mod analyzer;
mod benchmark;

pub use analyzer::FundAnalyzer;
pub use benchmark::{resolve_benchmark, IndexBenchmark, HS300};

#[derive(Debug, Clone)]
pub struct BenchmarkData {
    pub dates: Vec<chrono::NaiveDate>,
    pub returns: Vec<f64>,
    pub label: String,
}

pub struct FundMetaInfo {
    pub manager_name: String,
    pub manager_tenure_days: i32,
    pub manager_total_return: f64,
    pub management_fee: f64,
    pub custody_fee: f64,
}
