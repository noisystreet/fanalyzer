use chrono::FixedOffset;

/// 指数行情数据点
#[derive(Debug, Clone)]
pub struct IndexData {
    pub date: chrono::DateTime<FixedOffset>,
    pub close: f64,
}

/// 基金经理信息
#[derive(Debug, Clone)]
pub struct FundManagerInfo {
    pub name: String,
    pub start_date: String,
    pub tenure_days: i32,
    pub total_return: f64,
}

/// 基金费率信息
#[derive(Debug, Clone)]
pub struct FundFeeInfo {
    pub management_fee: f64,
    pub custody_fee: f64,
    pub purchase_fee: f64,
    pub redemption_fee: f64,
}

/// 基金概况信息
#[derive(Debug, Clone)]
pub struct FundProfile {
    pub code: String,
    pub name: String,
    pub full_name: String,
    pub fund_type: String,
    pub establishment_date: String,
    pub asset_size: String,
    pub company: String,
    pub manager_name: String,
    pub manager_tenure_days: i32,
    pub manager_total_return: f64,
    pub management_fee: f64,
    pub custody_fee: f64,
    pub investment_target: String,
    pub investment_scope: String,
    pub investment_strategy: String,
    pub benchmark: String,
}
