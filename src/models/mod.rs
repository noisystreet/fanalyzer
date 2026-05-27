pub mod portfolio;
pub mod reports;
pub mod series;

pub use portfolio::{
    CorrelationMatrix, InsightLevel, OverlapPair, PortfolioInsight, PortfolioInterpretation,
    PortfolioMember, PortfolioReport, PortfolioSummary,
};
pub use reports::{
    FundBrief, FundOverview, FundRankRow, IndustryAllocation, IndustryRow, StockHoldingRow,
    StockHoldings,
};
pub use series::{FundAnalysisReport, FundAnalysisSeries, PortfolioTimeSeries, SeriesPoint};

use chrono::NaiveDate;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum ModelError {
    #[error("Invalid fund code: {0}")]
    InvalidCode(String),
    #[error("Data point missing for date: {0}")]
    MissingDataPoint(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fund {
    pub code: String,
    pub name: String,
    pub fund_type: FundType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[allow(clippy::upper_case_acronyms)]
pub enum FundType {
    Stock,
    Bond,
    Hybrid,
    Index,
    Monetary,
    QDII,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FundNav {
    pub code: String,
    pub date: NaiveDate,
    pub nav: f64,
    pub acc_nav: f64,
    pub daily_return: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FundAnalysis {
    pub code: String,
    pub name: String,
    pub period_days: u32,
    pub avg_nav: f64,
    pub max_nav: f64,
    pub min_nav: f64,
    pub total_return: f64,
    pub annualized_return: f64,
    pub volatility: f64,
    pub max_drawdown: f64,
    pub sharpe_ratio: f64,
    pub sortino_ratio: f64,
    pub calmar_ratio: f64,
    pub alpha: f64,
    pub beta: f64,
    pub manager_name: String,
    pub manager_tenure_days: i32,
    pub manager_total_return: f64,
    pub management_fee: f64,
    pub custody_fee: f64,
}
