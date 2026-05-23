//! 分析用领域类型（无 IO）。

#[derive(Debug, Clone)]
pub struct BenchmarkData {
    pub dates: Vec<chrono::NaiveDate>,
    pub returns: Vec<f64>,
    pub label: String,
}

#[derive(Debug, Clone)]
pub struct FundMetaInfo {
    pub manager_name: String,
    pub manager_tenure_days: i32,
    pub manager_total_return: f64,
    pub management_fee: f64,
    pub custody_fee: f64,
}
