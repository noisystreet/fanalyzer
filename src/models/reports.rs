//! 呈现层使用的基金报告视图类型（与 HTTP DTO 解耦）。

use super::FundAnalysis;

/// 基金概况（F10 / info 命令）。
#[derive(Debug, Clone, Default)]
pub struct FundOverview {
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
    pub benchmark: String,
}

/// 行业配置一行。
#[derive(Debug, Clone)]
pub struct IndustryRow {
    pub rank: u32,
    pub industry: String,
    pub pct_nav: f64,
    pub market_value_wan: Option<f64>,
}

/// 行业配置报告。
#[derive(Debug, Clone, Default)]
pub struct IndustryAllocation {
    pub as_of: Option<String>,
    pub rows: Vec<IndustryRow>,
}

/// 重仓股一行。
#[derive(Debug, Clone)]
pub struct StockHoldingRow {
    pub rank: u32,
    pub stock_code: String,
    pub stock_name: String,
    pub pct_nav: f64,
    pub shares_wan: Option<f64>,
    pub market_value_wan: Option<f64>,
}

/// 重仓股报告。
#[derive(Debug, Clone, Default)]
pub struct StockHoldings {
    pub as_of: Option<String>,
    pub rows: Vec<StockHoldingRow>,
}

/// 排行表一行（展示用）。
#[derive(Debug, Clone)]
pub struct FundRankRow {
    pub code: String,
    pub name: String,
    pub pct_week: Option<f64>,
    pub pct_month: Option<f64>,
    pub pct_3m: Option<f64>,
    pub pct_6m: Option<f64>,
    pub pct_1y: Option<f64>,
    pub pct_this_year: Option<f64>,
}

/// 选基综合简报（终端与 Markdown 共用）。
#[derive(Debug, Clone)]
pub struct FundBrief {
    pub code: String,
    pub name: String,
    pub fund_type: String,
    pub company: String,
    pub asset_size: String,
    pub days: u32,
    pub analysis: Option<FundAnalysis>,
    pub industry: IndustryAllocation,
    pub holdings: StockHoldings,
    pub industry_top: usize,
    pub holdings_top: usize,
}
