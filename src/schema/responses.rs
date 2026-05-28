//! 各 `json` 子命令成功响应信封（供 schemars 导出；与 runtime 信封字段一致）。

#![allow(dead_code)]

use crate::application::{PortfolioConfigPayload, WatchlistItem};
use crate::models::{FundAnalysis, FundAnalysisReport, FundBrief, FundOverview, PortfolioReport};
use crate::presentation::{
    AnalysisMeta, BatchMeta, BatchPayload, ExportMeta, ExportPayload, FetchPayload, HoldingsItem,
    PortfolioMeta, RankMeta, RankPayload, ScreenMeta, ScreenPayload, SectorItem, StructuredError,
};
use schemars::JsonSchema;

#[derive(JsonSchema)]
#[schemars(
    title = "AnalyzeSuccessEnvelope",
    description = "json analyze 成功响应"
)]
pub struct AnalyzeSuccessEnvelope {
    pub v: u32,
    pub command: String,
    pub ok: bool,
    pub meta: Option<AnalysisMeta>,
    pub warnings: Vec<String>,
    pub data: BatchPayload<FundAnalysisReport>,
}

#[derive(JsonSchema)]
#[schemars(title = "CompareSuccessEnvelope")]
pub struct CompareSuccessEnvelope {
    pub v: u32,
    pub command: String,
    pub ok: bool,
    pub meta: Option<AnalysisMeta>,
    pub warnings: Vec<String>,
    pub data: BatchPayload<FundAnalysis>,
}

#[derive(JsonSchema)]
#[schemars(title = "PortfolioSuccessEnvelope")]
pub struct PortfolioSuccessEnvelope {
    pub v: u32,
    pub command: String,
    pub ok: bool,
    pub meta: Option<PortfolioMeta>,
    pub warnings: Vec<String>,
    pub data: PortfolioReport,
}

#[derive(JsonSchema)]
#[schemars(title = "FetchSuccessEnvelope")]
pub struct FetchSuccessEnvelope {
    pub v: u32,
    pub command: String,
    pub ok: bool,
    pub meta: Option<BatchMeta>,
    pub warnings: Vec<String>,
    pub data: BatchPayload<FetchPayload>,
}

#[derive(JsonSchema)]
#[schemars(title = "ExportSuccessEnvelope")]
pub struct ExportSuccessEnvelope {
    pub v: u32,
    pub command: String,
    pub ok: bool,
    pub meta: Option<ExportMeta>,
    pub warnings: Vec<String>,
    pub data: BatchPayload<ExportPayload>,
}

#[derive(JsonSchema)]
#[schemars(title = "InfoSuccessEnvelope")]
pub struct InfoSuccessEnvelope {
    pub v: u32,
    pub command: String,
    pub ok: bool,
    pub meta: Option<BatchMeta>,
    pub warnings: Vec<String>,
    pub data: BatchPayload<FundOverview>,
}

#[derive(JsonSchema)]
#[schemars(title = "SectorsSuccessEnvelope")]
pub struct SectorsSuccessEnvelope {
    pub v: u32,
    pub command: String,
    pub ok: bool,
    pub meta: Option<BatchMeta>,
    pub warnings: Vec<String>,
    pub data: BatchPayload<SectorItem>,
}

#[derive(JsonSchema)]
#[schemars(title = "HoldingsSuccessEnvelope")]
pub struct HoldingsSuccessEnvelope {
    pub v: u32,
    pub command: String,
    pub ok: bool,
    pub meta: Option<BatchMeta>,
    pub warnings: Vec<String>,
    pub data: BatchPayload<HoldingsItem>,
}

#[derive(JsonSchema)]
#[schemars(title = "RankSuccessEnvelope")]
pub struct RankSuccessEnvelope {
    pub v: u32,
    pub command: String,
    pub ok: bool,
    pub meta: Option<RankMeta>,
    pub warnings: Vec<String>,
    pub data: RankPayload,
}

#[derive(JsonSchema)]
#[schemars(title = "BriefSuccessEnvelope")]
pub struct BriefSuccessEnvelope {
    pub v: u32,
    pub command: String,
    pub ok: bool,
    pub meta: Option<AnalysisMeta>,
    pub warnings: Vec<String>,
    pub data: BatchPayload<FundBrief>,
}

#[derive(JsonSchema)]
#[schemars(title = "ScreenSuccessEnvelope")]
pub struct ScreenSuccessEnvelope {
    pub v: u32,
    pub command: String,
    pub ok: bool,
    pub meta: Option<ScreenMeta>,
    pub warnings: Vec<String>,
    pub data: ScreenPayload,
}

#[derive(JsonSchema)]
#[schemars(title = "WatchlistSuccessEnvelope")]
pub struct WatchlistSuccessEnvelope {
    pub v: u32,
    pub command: String,
    pub ok: bool,
    pub meta: Option<BatchMeta>,
    pub warnings: Vec<String>,
    pub data: BatchPayload<WatchlistItem>,
}

#[derive(JsonSchema)]
#[schemars(title = "PortfolioConfigSuccessEnvelope")]
pub struct PortfolioConfigSuccessEnvelope {
    pub v: u32,
    pub command: String,
    pub ok: bool,
    pub meta: Option<crate::presentation::BaseMeta>,
    pub warnings: Vec<String>,
    pub data: PortfolioConfigPayload,
}

/// 复合工具 `research_fund` 单步结果（成功或失败子信封）。
#[derive(JsonSchema)]
#[schemars(title = "ResearchFundStepEnvelope")]
pub struct ResearchFundStepEnvelope {
    pub v: u32,
    pub command: String,
    pub ok: bool,
    pub warnings: Vec<String>,
    pub meta: Option<serde_json::Value>,
    pub data: Option<serde_json::Value>,
    pub error: Option<StructuredError>,
}

/// 复合工具 `research_fund` 的 data 字段。
#[derive(JsonSchema)]
#[schemars(title = "ResearchFundData")]
pub struct ResearchFundData {
    pub info: ResearchFundStepEnvelope,
    pub analyze: ResearchFundStepEnvelope,
    pub sectors: ResearchFundStepEnvelope,
    pub holdings: ResearchFundStepEnvelope,
}

/// 复合工具 `research_fund` meta。
#[derive(JsonSchema)]
#[schemars(title = "ResearchFundMeta")]
pub struct ResearchFundMeta {
    pub offline: bool,
    pub steps_completed: u32,
    pub duration_ms: Option<u64>,
}

#[derive(JsonSchema)]
#[schemars(
    title = "ResearchFundSuccessEnvelope",
    description = "MCP research_fund 复合工具成功响应"
)]
pub struct ResearchFundSuccessEnvelope {
    pub v: u32,
    pub command: String,
    pub ok: bool,
    pub meta: Option<ResearchFundMeta>,
    pub warnings: Vec<String>,
    pub data: ResearchFundData,
}

/// 供 index 注册的成功信封清单。
pub const SUCCESS_ENVELOPES: &[(&str, &str)] = &[
    ("analyze", "responses/analyze.success.json"),
    ("compare", "responses/compare.success.json"),
    ("portfolio", "responses/portfolio.success.json"),
    ("fetch", "responses/fetch.success.json"),
    ("export", "responses/export.success.json"),
    ("info", "responses/info.success.json"),
    ("sectors", "responses/sectors.success.json"),
    ("holdings", "responses/holdings.success.json"),
    ("rank", "responses/rank.success.json"),
    ("brief", "responses/brief.success.json"),
    ("screen", "responses/screen.success.json"),
    ("watchlist", "responses/watchlist.success.json"),
    (
        "portfolio_config",
        "responses/portfolio_config.success.json",
    ),
    ("research_fund", "responses/research_fund.success.json"),
];
