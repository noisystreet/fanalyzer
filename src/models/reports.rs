//! 呈现层使用的基金报告视图类型（与 HTTP DTO 解耦）。

use super::FundAnalysis;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// 单个现任基金经理（呈现层）。
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct FundManagerView {
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub start_date: String,
    pub tenure_days: i32,
    pub total_return: f64,
}

/// 同类排名（近 3 月，来自天天基金 pingzhongdata）。
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct PeerRankInfo {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub as_of: Option<String>,
    /// 名次（1 最好）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rank: Option<u32>,
    /// 同类基金数
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub peer_count: Option<u32>,
    /// 百分位 0–100（越高越好）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub percentile: Option<f64>,
}

impl PeerRankInfo {
    pub fn is_empty(&self) -> bool {
        self.rank.is_none() && self.peer_count.is_none() && self.percentile.is_none()
    }
}

/// 基金概况（F10 / info 命令）。
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct FundOverview {
    pub code: String,
    pub name: String,
    pub full_name: String,
    pub fund_type: String,
    pub establishment_date: String,
    pub asset_size: String,
    pub company: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub managers: Vec<FundManagerView>,
    pub manager_name: String,
    pub manager_tenure_days: i32,
    pub manager_total_return: f64,
    pub management_fee: f64,
    pub custody_fee: f64,
    pub investment_target: String,
    pub investment_scope: String,
    pub benchmark: String,
    #[serde(default, skip_serializing_if = "PeerRankInfo::is_empty")]
    pub peer_rank: PeerRankInfo,
}

/// 行业配置一行。
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct IndustryRow {
    pub rank: u32,
    pub industry: String,
    pub pct_nav: f64,
    pub market_value_wan: Option<f64>,
}

/// 行业配置报告。
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct IndustryAllocation {
    pub as_of: Option<String>,
    pub rows: Vec<IndustryRow>,
}

/// 重仓股一行。
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct StockHoldingRow {
    pub rank: u32,
    pub stock_code: String,
    pub stock_name: String,
    pub pct_nav: f64,
    pub shares_wan: Option<f64>,
    pub market_value_wan: Option<f64>,
}

/// 重仓股报告。
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct StockHoldings {
    pub as_of: Option<String>,
    pub rows: Vec<StockHoldingRow>,
}

/// 排行表一行（展示用）。
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
    #[serde(default, skip_serializing_if = "PeerRankInfo::is_empty")]
    pub peer_rank: PeerRankInfo,
}
