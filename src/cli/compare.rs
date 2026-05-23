//! `compare` 子命令参数与执行。

use super::analysis_sort::{parse_sort_key, sort_analyses, AnalysisSortKey};
use super::compare_output::render_comparison;
use super::fund_session::analyze_fund;
use super::Cli;
use crate::analysis_period::resolve_analysis_days;
use crate::api::eastmoney::EastMoneyClient;
use crate::cache::FundCache;
use crate::nav_cache::NavCache;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct CompareOpts {
    pub codes: Vec<String>,
    pub days: u32,
    pub period: Option<String>,
    pub sort: Option<String>,
    pub output: Option<PathBuf>,
    pub format: String,
}

pub async fn run_compare(
    cli: &Cli,
    client: &EastMoneyClient,
    cache: &Arc<Mutex<FundCache>>,
    nav_store: &NavCache,
    opts: CompareOpts,
) -> anyhow::Result<()> {
    if opts.codes.len() < 2 {
        anyhow::bail!("对比至少需要 2 只基金（当前 {} 条）", opts.codes.len());
    }
    let days = resolve_analysis_days(opts.period.as_deref(), opts.days)?;
    tracing::info!(codes = ?opts.codes, days = days, "Comparing funds");

    let mut analyses = Vec::new();
    for identifier in &opts.codes {
        match analyze_fund(client, cache, nav_store, identifier, days, cli.offline).await {
            Ok(Some(a)) => analyses.push(a),
            Ok(None) => tracing::warn!(identifier = %identifier, "分析数据不足，跳过"),
            Err(e) => tracing::warn!(identifier = %identifier, error = %e, "跳过该标的"),
        }
    }

    if analyses.len() < 2 {
        tracing::warn!("有效样本不足（需要≥2）；请检查离线缓存或数据源");
        return Ok(());
    }

    if let Some(ref sort_raw) = opts.sort {
        let key = parse_sort_key(sort_raw)?;
        sort_analyses(&mut analyses, key, key.default_desc());
    } else {
        sort_analyses(&mut analyses, AnalysisSortKey::Code, false);
    }

    render_comparison(&analyses, opts.output.as_deref(), &opts.format)
}
