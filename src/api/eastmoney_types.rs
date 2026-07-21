use chrono::FixedOffset;

/// 指数行情数据点
#[derive(Debug, Clone)]
pub struct IndexData {
    pub date: chrono::DateTime<FixedOffset>,
    pub close: f64,
}

/// 基金经理信息
#[derive(Debug, Clone, Default)]
pub struct FundManagerInfo {
    pub name: String,
    /// 本基金上任日期（YYYY-MM-DD），未知时为空
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

/// 同类排名快照（来自 pingzhongdata 近 3 月口径）。
#[derive(Debug, Clone, Default)]
pub struct PeerRankSnapshot {
    /// 排名对应日期（YYYY-MM-DD）
    pub as_of: Option<String>,
    /// 名次（1 最好）
    pub rank: Option<u32>,
    /// 同类基金数
    pub peer_count: Option<u32>,
    /// 百分位 0–100（越高越好，约等于「前 X%」）
    pub percentile: Option<f64>,
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
    /// 现任经理列表（可多人）
    pub managers: Vec<FundManagerInfo>,
    /// 兼容字段：经理姓名拼接（顿号分隔）
    pub manager_name: String,
    pub manager_tenure_days: i32,
    pub manager_total_return: f64,
    pub management_fee: f64,
    pub custody_fee: f64,
    /// 申购费率（百分点；优先天天基金优惠档）
    pub purchase_fee: f64,
    /// 最高赎回费率（百分点；阶梯费率取最高档）
    pub redemption_fee: f64,
    /// 申购/交易状态（如「限大额」）
    pub subscribe_status: String,
    /// 赎回状态（如「开放赎回」）
    pub redeem_status: String,
    pub investment_target: String,
    pub investment_scope: String,
    pub investment_strategy: String,
    pub benchmark: String,
    pub peer_rank: PeerRankSnapshot,
}
