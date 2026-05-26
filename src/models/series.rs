//! 时间序列与带图表的分析报告模型。

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use super::FundAnalysis;

/// 单个时间序列数据点。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriesPoint {
    pub date: NaiveDate,
    pub value: f64,
}

/// 单基金滚动指标与净值/回撤曲线。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundAnalysisSeries {
    /// 滚动窗口（交易日）
    pub rolling_window: u32,
    pub nav_normalized: Vec<SeriesPoint>,
    pub drawdown: Vec<SeriesPoint>,
    pub rolling_sharpe: Vec<SeriesPoint>,
    pub rolling_beta: Vec<SeriesPoint>,
    pub rolling_volatility: Vec<SeriesPoint>,
}

/// 单基金完整分析报告（标量快照 + 可选时间序列）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundAnalysisReport {
    pub snapshot: FundAnalysis,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub series: Option<FundAnalysisSeries>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub benchmark_label: Option<String>,
}

/// 组合层时间序列（加权净值与滚动指标）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioTimeSeries {
    pub rolling_window: u32,
    pub nav_normalized: Vec<SeriesPoint>,
    pub drawdown: Vec<SeriesPoint>,
    pub rolling_sharpe: Vec<SeriesPoint>,
    pub rolling_volatility: Vec<SeriesPoint>,
}
