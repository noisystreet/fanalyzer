//! CLI 结构化 JSON 输出（Agent / 自动化调用）。

use crate::application::CommandContext;
use crate::models::{FundAnalysisReport, PortfolioReport};
use chrono::Local;
use serde::Serialize;
use std::io::{self, Write};
use std::path::Path;

/// 响应信封版本。
pub const ENVELOPE_VERSION: u32 = 1;

/// 结构化 CLI 错误（可 downcast 以映射 error code）。
#[derive(Debug, Clone, thiserror::Error)]
#[error("{message}")]
pub struct CodedError {
    pub code: &'static str,
    pub message: String,
}

impl CodedError {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

/// 统一错误体。
#[derive(Debug, Clone, Serialize)]
pub struct StructuredError {
    pub code: String,
    pub message: String,
}

/// 批量条目失败信息。
#[derive(Debug, Clone, Serialize)]
pub struct ItemError {
    pub code: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
}

/// 多基金/多条目结果（含可选 partial errors）。
#[derive(Debug, Serialize)]
pub struct BatchPayload<T> {
    pub items: Vec<T>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<ItemError>,
}

/// 兼容别名。
pub type ItemsPayload<T> = BatchPayload<T>;

/// 统一 JSON 响应信封（成功）。
#[derive(Debug, Serialize)]
pub struct StructuredEnvelope<T> {
    pub v: u32,
    pub command: &'static str,
    pub ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
    pub data: T,
}

/// 失败响应信封（无 data）。
#[derive(Debug, Serialize)]
pub struct StructuredFailureEnvelope {
    pub v: u32,
    pub command: String,
    pub ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
    pub error: StructuredError,
}

/// 信封 meta 公共字段。
#[derive(Debug, Clone, Serialize)]
pub struct BaseMeta {
    pub offline: bool,
    pub generated_at: String,
}

/// 分析类命令 meta。
#[derive(Debug, Clone, Serialize)]
pub struct AnalysisMeta {
    #[serde(flatten)]
    pub base: BaseMeta,
    pub days: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub period: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rolling_window: Option<u32>,
    pub requested: usize,
    pub succeeded: usize,
}

/// 批量查询 meta。
#[derive(Debug, Clone, Serialize)]
pub struct BatchMeta {
    #[serde(flatten)]
    pub base: BaseMeta,
    pub requested: usize,
    pub succeeded: usize,
}

/// 排行 meta。
#[derive(Debug, Clone, Serialize)]
pub struct RankMeta {
    #[serde(flatten)]
    pub base: BaseMeta,
    pub kind: String,
    pub sort: String,
    pub top: u32,
}

/// 筛选 meta。
#[derive(Debug, Clone, Serialize)]
pub struct ScreenMeta {
    #[serde(flatten)]
    pub base: BaseMeta,
    pub kind: String,
    pub sort: String,
    pub days: u32,
    pub pool_size: usize,
    pub analyzed: usize,
}

/// 组合 meta。
#[derive(Debug, Clone, Serialize)]
pub struct PortfolioMeta {
    #[serde(flatten)]
    pub base: BaseMeta,
    pub days: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub period: Option<String>,
    pub rolling_window: u32,
    pub holdings: usize,
}

/// 导出 meta。
#[derive(Debug, Clone, Serialize)]
pub struct ExportMeta {
    #[serde(flatten)]
    pub base: BaseMeta,
    pub days: u32,
    pub requested: usize,
    pub succeeded: usize,
}

/// 排行结果。
#[derive(Debug, Serialize)]
pub struct RankPayload {
    pub kind: String,
    pub sort: String,
    pub total_records: u32,
    pub rows: Vec<crate::models::FundRankRow>,
}

/// 筛选结果。
#[derive(Debug, Serialize)]
pub struct ScreenPayload {
    pub kind: String,
    pub sort: String,
    pub days: u32,
    pub pool_size: usize,
    pub analyzed: usize,
    pub passed: Vec<crate::models::FundAnalysis>,
}

/// 净值拉取结果。
#[derive(Debug, Serialize)]
pub struct FetchPayload {
    pub code: String,
    pub name: String,
    pub total: u32,
    pub fetched: usize,
    pub nav: Vec<crate::models::FundNav>,
}

/// 行业配置条目。
#[derive(Debug, Serialize)]
pub struct SectorItem {
    pub code: String,
    pub name: String,
    pub industry: crate::models::IndustryAllocation,
}

/// 重仓股条目。
#[derive(Debug, Serialize)]
pub struct HoldingsItem {
    pub code: String,
    pub name: String,
    pub holdings: crate::models::StockHoldings,
}

/// 净值导出条目。
#[derive(Debug, Serialize)]
pub struct ExportPayload {
    pub code: String,
    pub name: String,
    pub days: u32,
    pub nav: Vec<crate::models::FundNav>,
}

pub fn base_meta(ctx: &CommandContext<'_>) -> BaseMeta {
    BaseMeta {
        offline: ctx.offline,
        generated_at: Local::now().to_rfc3339(),
    }
}

pub fn item_error(code: &str, error_code: &'static str, message: impl Into<String>) -> ItemError {
    ItemError {
        code: code.to_string(),
        message: message.into(),
        error_code: Some(error_code.to_string()),
    }
}

pub fn item_error_insufficient(code: &str) -> ItemError {
    item_error(code, "INSUFFICIENT_DATA", "分析数据不足")
}

pub fn item_error_failed(code: &str, message: impl std::fmt::Display) -> ItemError {
    item_error(code, "ANALYSIS_FAILED", message.to_string())
}

/// 从 anyhow 错误映射结构化 error code。
pub fn error_from_anyhow(err: &anyhow::Error) -> StructuredError {
    if let Some(coded) = err.downcast_ref::<CodedError>() {
        return StructuredError {
            code: coded.code.to_string(),
            message: coded.message.clone(),
        };
    }
    let message = err.to_string();
    let code = if message.contains("至少需要 2 只") || message.contains("有效样本不足") {
        "INSUFFICIENT_SAMPLES"
    } else if message.contains("无有效") {
        "INSUFFICIENT_DATA"
    } else if message.contains("--offline") {
        "OFFLINE_UNSUPPORTED"
    } else {
        "COMMAND_FAILED"
    };
    StructuredError {
        code: code.to_string(),
        message,
    }
}

fn meta_to_value<M: Serialize>(meta: Option<&M>) -> anyhow::Result<Option<serde_json::Value>> {
    match meta {
        Some(m) => Ok(Some(serde_json::to_value(m)?)),
        None => Ok(None),
    }
}

fn write_json_stdout(json: &str) -> anyhow::Result<()> {
    let mut out = io::stdout().lock();
    out.write_all(json.as_bytes())?;
    out.write_all(b"\n")?;
    Ok(())
}

fn serialize_success<T: Serialize, M: Serialize>(
    command: &'static str,
    data: &T,
    meta: Option<&M>,
    warnings: &[String],
    compact: bool,
) -> anyhow::Result<String> {
    let envelope = StructuredEnvelope {
        v: ENVELOPE_VERSION,
        command,
        ok: true,
        meta: meta_to_value(meta)?,
        warnings: warnings.to_vec(),
        data,
    };
    if compact {
        Ok(serde_json::to_string(&envelope)?)
    } else {
        Ok(serde_json::to_string_pretty(&envelope)?)
    }
}

fn serialize_failure<M: Serialize>(
    command: &str,
    error: &StructuredError,
    meta: Option<&M>,
    warnings: &[String],
    compact: bool,
) -> anyhow::Result<String> {
    let envelope = StructuredFailureEnvelope {
        v: ENVELOPE_VERSION,
        command: command.to_string(),
        ok: false,
        meta: meta_to_value(meta)?,
        warnings: warnings.to_vec(),
        error: error.clone(),
    };
    if compact {
        Ok(serde_json::to_string(&envelope)?)
    } else {
        Ok(serde_json::to_string_pretty(&envelope)?)
    }
}

/// 向 stdout 打印成功 JSON 信封。
pub fn print_success_stdout<T: Serialize, M: Serialize>(
    ctx: &CommandContext<'_>,
    command: &'static str,
    data: &T,
    meta: Option<&M>,
) -> anyhow::Result<()> {
    let json = serialize_success(
        command,
        data,
        meta,
        &ctx.take_warnings(),
        ctx.json_compact(),
    )?;
    write_json_stdout(&json)
}

/// 向 stdout 打印失败 JSON 信封。
pub fn print_failure_stdout(
    ctx: &CommandContext<'_>,
    command: &str,
    error: &StructuredError,
    meta: Option<&BaseMeta>,
) -> anyhow::Result<()> {
    let json = serialize_failure(
        command,
        error,
        meta,
        &ctx.take_warnings(),
        ctx.json_compact(),
    )?;
    write_json_stdout(&json)
}

/// 顶层错误处理：`--json` 模式下将 anyhow 错误转为 stdout 失败信封。
pub fn print_failure_from_anyhow(
    ctx: &CommandContext<'_>,
    command: &str,
    err: &anyhow::Error,
) -> anyhow::Result<()> {
    let structured = error_from_anyhow(err);
    print_failure_stdout(ctx, command, &structured, Some(&base_meta(ctx)))
}

/// 将 JSON 信封写入文件（始终 pretty）。
pub fn write_file<T: Serialize, M: Serialize>(
    path: &Path,
    command: &'static str,
    data: &T,
    meta: Option<&M>,
    warnings: &[String],
) -> anyhow::Result<()> {
    let json = serialize_success(command, data, meta, warnings, false)?;
    std::fs::write(path, json)?;
    Ok(())
}

/// `--json` 模式下输出；可选同时写 `--output` 文件。
pub fn emit<T: Serialize, M: Serialize>(
    ctx: &CommandContext<'_>,
    command: &'static str,
    data: &T,
    meta: Option<&M>,
    file: Option<&Path>,
) -> anyhow::Result<()> {
    if !ctx.structured() {
        return Ok(());
    }
    let warnings = ctx.take_warnings();
    print_success_stdout(ctx, command, data, meta)?;
    if let Some(path) = file {
        write_file(path, command, data, meta, &warnings)?;
        tracing::info!(path = %path.display(), "Wrote structured JSON export");
    }
    Ok(())
}

/// 省略分析报告中的时间序列（省 token）。
pub fn compact_analysis_reports(reports: &mut [FundAnalysisReport]) {
    for report in reports {
        report.series = None;
    }
}

/// 省略组合报告中的时间序列。
pub fn compact_portfolio_report(report: &mut PortfolioReport) {
    report.series = None;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn envelope_serializes_command_field() {
        let json = serde_json::to_string(&StructuredEnvelope {
            v: ENVELOPE_VERSION,
            command: "analyze",
            ok: true,
            meta: None,
            warnings: vec![],
            data: BatchPayload {
                items: Vec::<i32>::new(),
                errors: vec![],
            },
        })
        .unwrap();
        assert!(json.contains("\"command\":\"analyze\""));
        assert!(json.contains("\"v\":1"));
        assert!(json.contains("\"ok\":true"));
    }

    #[test]
    fn failure_envelope_has_error_code() {
        let json = serde_json::to_string(&StructuredFailureEnvelope {
            v: ENVELOPE_VERSION,
            command: "compare".into(),
            ok: false,
            meta: None,
            warnings: vec![],
            error: StructuredError {
                code: "INSUFFICIENT_SAMPLES".into(),
                message: "有效样本不足".into(),
            },
        })
        .unwrap();
        assert!(json.contains("INSUFFICIENT_SAMPLES"));
        assert!(json.contains("\"ok\":false"));
    }

    #[test]
    fn batch_payload_includes_errors() {
        let json = serde_json::to_string(&BatchPayload {
            items: vec![1],
            errors: vec![ItemError {
                code: "000001".into(),
                message: "数据不足".into(),
                error_code: Some("INSUFFICIENT_DATA".into()),
            }],
        })
        .unwrap();
        assert!(json.contains("000001"));
        assert!(json.contains("INSUFFICIENT_DATA"));
    }

    #[test]
    fn error_from_anyhow_maps_insufficient_samples() {
        let err = anyhow::anyhow!("有效样本不足（需要≥2）");
        let structured = error_from_anyhow(&err);
        assert_eq!(structured.code, "INSUFFICIENT_SAMPLES");
    }

    #[test]
    fn error_from_anyhow_downcasts_coded_error() {
        let err: anyhow::Error = CodedError::new("CUSTOM", "custom failure").into();
        let structured = error_from_anyhow(&err);
        assert_eq!(structured.code, "CUSTOM");
    }

    #[test]
    fn compact_serialization_is_single_line_object() {
        let json = serialize_success(
            "analyze",
            &BatchPayload::<i32> {
                items: vec![1],
                errors: vec![],
            },
            None::<&BaseMeta>,
            &[],
            true,
        )
        .unwrap();
        assert!(!json.contains('\n'));
    }
}
