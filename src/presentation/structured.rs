//! CLI 结构化 JSON 输出（Agent / 自动化调用）。

use crate::application::CommandContext;
use crate::models::{FundAnalysisReport, PortfolioReport};
use chrono::Local;
use schemars::JsonSchema;
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
    pub retryable: Option<bool>,
    pub hint: Option<String>,
}

impl CodedError {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            retryable: None,
            hint: None,
        }
    }

    pub fn with_hint(
        code: &'static str,
        message: impl Into<String>,
        hint: impl Into<String>,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            retryable: Some(true),
            hint: Some(hint.into()),
        }
    }
}

/// 统一错误体。
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct StructuredError {
    pub code: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retryable: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

/// 批量条目失败信息。
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ItemError {
    pub code: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
}

/// 多基金/多条目结果（含可选 partial errors）。
#[derive(Debug, Serialize, JsonSchema)]
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
    pub warnings: Vec<String>,
    pub data: T,
}

/// 失败响应信封（无 data）。
#[derive(Debug, Serialize, JsonSchema)]
pub struct StructuredFailureEnvelope {
    pub v: u32,
    pub command: String,
    pub ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
    pub warnings: Vec<String>,
    pub error: StructuredError,
}

/// 信封 meta 公共字段。
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct BaseMeta {
    pub offline: bool,
    pub generated_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
}

/// 分析类命令 meta。
#[derive(Debug, Clone, Serialize, JsonSchema)]
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
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct BatchMeta {
    #[serde(flatten)]
    pub base: BaseMeta,
    pub requested: usize,
    pub succeeded: usize,
}

/// 排行 meta。
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct RankMeta {
    #[serde(flatten)]
    pub base: BaseMeta,
    pub kind: String,
    pub sort: String,
    pub top: u32,
}

/// 筛选 meta。
#[derive(Debug, Clone, Serialize, JsonSchema)]
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
#[derive(Debug, Clone, Serialize, JsonSchema)]
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
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ExportMeta {
    #[serde(flatten)]
    pub base: BaseMeta,
    pub days: u32,
    pub requested: usize,
    pub succeeded: usize,
}

/// 排行结果。
#[derive(Debug, Serialize, JsonSchema)]
pub struct RankPayload {
    pub kind: String,
    pub sort: String,
    pub total_records: u32,
    pub rows: Vec<crate::models::FundRankRow>,
}

/// 筛选结果。
#[derive(Debug, Serialize, JsonSchema)]
pub struct ScreenPayload {
    pub kind: String,
    pub sort: String,
    pub days: u32,
    pub pool_size: usize,
    pub analyzed: usize,
    pub passed: Vec<crate::models::FundAnalysis>,
}

/// 净值拉取结果。
#[derive(Debug, Serialize, JsonSchema)]
pub struct FetchPayload {
    pub code: String,
    pub name: String,
    pub total: u32,
    pub fetched: usize,
    pub nav: Vec<crate::models::FundNav>,
}

/// 行业配置条目。
#[derive(Debug, Serialize, JsonSchema)]
pub struct SectorItem {
    pub code: String,
    pub name: String,
    pub industry: crate::models::IndustryAllocation,
}

/// 重仓股条目。
#[derive(Debug, Serialize, JsonSchema)]
pub struct HoldingsItem {
    pub code: String,
    pub name: String,
    pub holdings: crate::models::StockHoldings,
}

/// 净值导出条目。
#[derive(Debug, Serialize, JsonSchema)]
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
        duration_ms: Some(ctx.elapsed_ms()),
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
            retryable: coded.retryable,
            hint: coded.hint.clone(),
        };
    }
    let message = err.to_string();
    let (code, retryable, hint) = classify_error(&message);
    StructuredError {
        code: code.to_string(),
        message,
        retryable: Some(retryable),
        hint,
    }
}

fn classify_error(message: &str) -> (&'static str, bool, Option<String>) {
    if message.contains("至少需要 2 只") || message.contains("有效样本不足") {
        return (
            "INSUFFICIENT_SAMPLES",
            false,
            Some("请增加 --codes 或使用 --watchlist；离线模式下需先有缓存".into()),
        );
    }
    if message.contains("无有效") || message.contains("分析数据不足") {
        return (
            "INSUFFICIENT_DATA",
            true,
            Some("先运行 fetch 或 analyze 写入缓存后再试".into()),
        );
    }
    if message.contains("--offline") || message.contains("需要访问网络") {
        return (
            "OFFLINE_UNSUPPORTED",
            true,
            Some("去掉 --offline 或先在线 fetch 所需基金".into()),
        );
    }
    if message.contains("自选列表为空")
        || message.contains("请指定")
        || message.contains("缺少 code")
    {
        return (
            "INVALID_ARGS",
            false,
            Some("检查参数是否符合工具 inputSchema / CLI 帮助".into()),
        );
    }
    if message.contains("未知工具") || message.contains("未知子命令") {
        return (
            "UNKNOWN_TOOL",
            false,
            Some("使用 tools/list 或 `fanalyzer json --help` 查看可用命令".into()),
        );
    }
    ("COMMAND_FAILED", false, None)
}

fn meta_to_value<M: Serialize>(meta: Option<&M>) -> anyhow::Result<Option<serde_json::Value>> {
    match meta {
        Some(m) => Ok(Some(serde_json::to_value(m)?)),
        None => Ok(None),
    }
}

fn write_json_output(ctx: Option<&CommandContext<'_>>, json: &str) -> anyhow::Result<()> {
    if let Some(ctx) = ctx
        && ctx.capture_enabled()
    {
        ctx.store_captured(json.to_string());
        return Ok(());
    }
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

/// 构建成功响应 JSON 字符串（供复合编排复用）。
pub fn success_envelope_json<T: Serialize, M: Serialize>(
    command: &'static str,
    data: &T,
    meta: Option<&M>,
    warnings: &[String],
) -> anyhow::Result<String> {
    serialize_success(command, data, meta, warnings, false)
}

/// 构建失败响应 JSON 字符串（供复合编排复用）。
pub fn failure_envelope_json(
    command: &str,
    error: &StructuredError,
    warnings: &[String],
) -> anyhow::Result<String> {
    serialize_failure(command, error, None::<&BaseMeta>, warnings, false)
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
    write_json_output(Some(ctx), &json)
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
    write_json_output(Some(ctx), &json)
}

/// 捕获模式：返回失败 JSON 字符串（不写 stdout）。
pub fn print_failure_capture(
    ctx: &CommandContext<'_>,
    command: &str,
    error: &StructuredError,
) -> anyhow::Result<String> {
    serialize_failure(
        command,
        error,
        Some(&base_meta(ctx)),
        &ctx.take_warnings(),
        ctx.json_compact(),
    )
}

/// 顶层错误处理：`json` 子命令模式下将 anyhow 错误转为 stdout 失败信封。
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

/// `json` 子命令模式下输出；可选同时写 `--output` 文件。
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

/// summary profile：精简简报中的行业/重仓明细。
pub fn compact_brief_summary(brief: &mut crate::models::FundBrief) {
    brief.industry.rows.truncate(3);
    brief.holdings.rows.truncate(3);
    brief.industry_top = brief.industry.rows.len();
    brief.holdings_top = brief.holdings.rows.len();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_meta_includes_duration_ms() {
        use crate::api::eastmoney::EastMoneyClient;
        use crate::application::{CommandContext, FundDataSource, StructuredOutput};
        use crate::cache::FundCache;
        use crate::nav_cache::NavCache;
        use std::path::Path;
        use std::sync::Arc;
        use std::thread;
        use std::time::Duration;
        use tokio::sync::Mutex;

        let client = EastMoneyClient::default();
        let name_cache = Arc::new(Mutex::new(FundCache::new()));
        let nav_store = NavCache::new();
        let ctx = CommandContext::new(
            &client as &dyn FundDataSource,
            &name_cache,
            &nav_store,
            false,
            Path::new("config/watchlist.toml"),
            StructuredOutput::OFF,
        );
        thread::sleep(Duration::from_millis(2));
        let meta = base_meta(&ctx);
        assert!(meta.duration_ms.unwrap_or(0) >= 1);
    }

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
                retryable: Some(false),
                hint: None,
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
    fn error_from_anyhow_includes_hint_for_insufficient_data() {
        let err = anyhow::anyhow!("无有效净值数据");
        let structured = error_from_anyhow(&err);
        assert_eq!(structured.code, "INSUFFICIENT_DATA");
        assert_eq!(structured.retryable, Some(true));
        assert!(structured.hint.is_some());
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
