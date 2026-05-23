//! 多基金对比用例。

use super::context::CommandContext;
use super::fund_service;
use crate::domain::{parse_sort_key, resolve_analysis_days, sort_analyses, AnalysisSortKey};
use crate::presentation::render_comparison;
use std::path::PathBuf;

pub struct CompareRequest {
    pub codes: Vec<String>,
    pub days: u32,
    pub period: Option<String>,
    pub sort: Option<String>,
    pub output: Option<PathBuf>,
    pub format: String,
}

pub async fn run_compare(ctx: &CommandContext<'_>, req: CompareRequest) -> anyhow::Result<()> {
    if req.codes.len() < 2 {
        anyhow::bail!("对比至少需要 2 只基金（当前 {} 条）", req.codes.len());
    }
    let days = resolve_analysis_days(req.period.as_deref(), req.days)?;
    tracing::info!(codes = ?req.codes, days = days, "Comparing funds");

    let mut analyses = Vec::new();
    for identifier in &req.codes {
        match fund_service::analyze_fund(&ctx.session, identifier, days, ctx.offline).await {
            Ok(Some(a)) => analyses.push(a),
            Ok(None) => tracing::warn!(identifier = %identifier, "分析数据不足，跳过"),
            Err(e) => tracing::warn!(identifier = %identifier, error = %e, "跳过该标的"),
        }
    }

    if analyses.len() < 2 {
        tracing::warn!("有效样本不足（需要≥2）；请检查离线缓存或数据源");
        return Ok(());
    }

    if let Some(ref sort_raw) = req.sort {
        let key = parse_sort_key(sort_raw)?;
        sort_analyses(&mut analyses, key, key.default_desc());
    } else {
        sort_analyses(&mut analyses, AnalysisSortKey::Code, false);
    }

    render_comparison(&analyses, req.output.as_deref(), &req.format)
}
