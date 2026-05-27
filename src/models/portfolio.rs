//! 组合分析结果模型。

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PortfolioMember {
    pub code: String,
    pub name: String,
    pub weight: f64,
    pub total_return: f64,
    pub volatility: f64,
    pub max_drawdown: f64,
    pub sharpe_ratio: f64,
    /// 静态贡献近似：`weight × total_return`
    pub return_contribution: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PortfolioSummary {
    pub name: String,
    pub period_days: u32,
    pub aligned_days: u32,
    pub total_return: f64,
    pub annualized_return: f64,
    pub volatility: f64,
    pub max_drawdown: f64,
    pub sharpe_ratio: f64,
    pub members: Vec<PortfolioMember>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CorrelationMatrix {
    pub labels: Vec<String>,
    pub values: Vec<Vec<f64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OverlapPair {
    pub fund_a_code: String,
    pub fund_a_name: String,
    pub fund_b_code: String,
    pub fund_b_name: String,
    /// 加权重叠（占净值百分比，0～100）
    pub overlap_pct: f64,
    pub shared_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PortfolioReport {
    pub summary: PortfolioSummary,
    pub correlation: CorrelationMatrix,
    pub overlaps: Vec<OverlapPair>,
    /// 规则引擎生成的解读（可选，由 application 填充）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interpretation: Option<PortfolioInterpretation>,
    /// 组合净值与滚动指标时间序列
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub series: Option<crate::models::PortfolioTimeSeries>,
}

/// 解读条目严重程度。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum InsightLevel {
    Positive,
    Info,
    Caution,
}

/// 单条解读。
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PortfolioInsight {
    pub level: InsightLevel,
    pub category: String,
    pub message: String,
}

/// 组合解读汇总。
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PortfolioInterpretation {
    pub headline: String,
    pub insights: Vec<PortfolioInsight>,
}
