//! 基金数据访问与分析编排（应用层）。

use super::context::Session;
use crate::domain::{
    build_fund_analysis_series, normalize_rolling_window, resolve_benchmark, BenchmarkData,
    FundAnalyzer, FundMetaInfo, IndexBenchmark, HS300,
};
use crate::models::FundAnalysisReport;
use crate::nav_cache::filter_covering_calendar_days;
use anyhow::Context;

pub async fn fetch_nav_series(
    session: &Session<'_>,
    resolved_code: &str,
    days: u32,
    offline: bool,
) -> anyhow::Result<Vec<crate::models::FundNav>> {
    if offline {
        let loaded = session.nav_store.load(resolved_code).with_context(|| {
            format!(
                "`--offline` 且无缓存 `{}`，请先在线跑一次 analyze/export",
                resolved_code
            )
        })?;
        let trimmed = filter_covering_calendar_days(loaded, days);
        if trimmed.is_empty() {
            anyhow::bail!(
                "`{}` 缓存中不包含最近 {} 天数据（或缓存过期），请先在线刷新",
                resolved_code,
                days
            );
        }
        Ok(trimmed)
    } else {
        let navs = session
            .source
            .fetch_nav_history_by_days(resolved_code, days)
            .await?;
        if !navs.is_empty() && session.nav_store.save_merged(resolved_code, &navs).is_err() {
            tracing::warn!("写入净值缓存失败（已忽略）：{}", resolved_code);
        }
        Ok(navs)
    }
}

pub async fn resolve_fund_identifier(
    session: &Session<'_>,
    identifier: &str,
    offline: bool,
) -> anyhow::Result<(String, String)> {
    let is_likely_code = identifier.chars().all(|c| c.is_ascii_digit()) && identifier.len() == 6;

    if is_likely_code {
        let name = if offline {
            let g = session.name_cache.lock().await;
            g.get_name(identifier)
                .unwrap_or_else(|| identifier.to_string())
        } else {
            get_fund_name(session, identifier).await
        };
        return Ok((identifier.to_string(), name));
    }

    if offline {
        let code = session
            .name_cache
            .lock()
            .await
            .get_code(identifier)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "`--offline` 无法解析名称 `{id}`，请先在线跑一次或直接使用 6 位代码",
                    id = identifier
                )
            })?;
        return Ok((code, identifier.to_string()));
    }

    {
        let cache_guard = session.name_cache.lock().await;
        if let Some(code) = cache_guard.get_code(identifier) {
            return Ok((code, identifier.to_string()));
        }
    }

    match session.source.search_fund(identifier).await {
        Ok(results) => {
            if let Some((code, name)) = results.first() {
                let mut cache_guard = session.name_cache.lock().await;
                cache_guard.set_mapping(code, name);
                Ok((code.clone(), name.clone()))
            } else {
                anyhow::bail!("未找到与 `{identifier}` 匹配的基金")
            }
        }
        Err(e) => anyhow::bail!("基金搜索失败：{e}"),
    }
}

pub async fn get_benchmark_data(
    session: &Session<'_>,
    days: u32,
    index: &IndexBenchmark,
) -> Option<BenchmarkData> {
    match session
        .source
        .fetch_index_history(index.secid, 1, days * 2)
        .await
    {
        Ok((data, _)) => {
            let mut dates = Vec::new();
            let mut returns = Vec::new();
            for i in 1..data.len() {
                let prev = &data[i - 1];
                let curr = &data[i];
                let daily_return = if prev.close != 0.0 {
                    (curr.close - prev.close) / prev.close
                } else {
                    0.0
                };
                dates.push(curr.date.date_naive());
                returns.push(daily_return);
            }
            Some(BenchmarkData {
                dates,
                returns,
                label: index.label.to_string(),
            })
        }
        Err(e) => {
            tracing::warn!(error = %e, index = index.label, "Failed to fetch benchmark data");
            None
        }
    }
}

async fn benchmark_for_fund(session: &Session<'_>, code: &str, days: u32) -> Option<BenchmarkData> {
    let index = match session.source.fetch_fund_profile(code).await {
        Ok(profile) => resolve_benchmark(&profile.benchmark, &profile.fund_type),
        Err(e) => {
            tracing::warn!(code = %code, error = %e, "Failed to fetch profile for benchmark; using HS300");
            HS300
        }
    };
    get_benchmark_data(session, days, &index).await
}

pub async fn get_fund_meta(session: &Session<'_>, code: &str) -> Option<FundMetaInfo> {
    let manager = match session.source.fetch_fund_manager(code).await {
        Ok(m) => m,
        Err(e) => {
            tracing::warn!(code = %code, error = %e, "Failed to fetch fund manager");
            return None;
        }
    };
    let fee = match session.source.fetch_fund_fee(code).await {
        Ok(f) => f,
        Err(e) => {
            tracing::warn!(code = %code, error = %e, "Failed to fetch fund fee");
            return None;
        }
    };
    Some(FundMetaInfo {
        manager_name: manager.name,
        manager_tenure_days: manager.tenure_days,
        manager_total_return: manager.total_return,
        management_fee: fee.management_fee,
        custody_fee: fee.custody_fee,
    })
}

async fn get_fund_name(session: &Session<'_>, code: &str) -> String {
    {
        let cache_guard = session.name_cache.lock().await;
        if let Some(name) = cache_guard.get_name(code) {
            return name;
        }
    }
    match session.source.fetch_fund_name(code).await {
        Ok(name) => {
            let mut cache_guard = session.name_cache.lock().await;
            cache_guard.set_mapping(code, &name);
            name
        }
        Err(e) => {
            tracing::warn!(code = %code, error = %e, "Failed to fetch fund name");
            code.to_string()
        }
    }
}

/// 拉净值并计算分析结果（不打印）。
pub async fn analyze_fund(
    session: &Session<'_>,
    identifier: &str,
    days: u32,
    offline: bool,
    rolling_window: u32,
) -> anyhow::Result<Option<FundAnalysisReport>> {
    let (resolved_code, name) = resolve_fund_identifier(session, identifier, offline).await?;
    let benchmark = if offline {
        None
    } else {
        benchmark_for_fund(session, &resolved_code, days).await
    };
    let meta = if offline {
        None
    } else {
        get_fund_meta(session, &resolved_code).await
    };
    let navs = fetch_nav_series(session, &resolved_code, days, offline).await?;
    if navs.is_empty() {
        return Ok(None);
    }
    let snapshot =
        match FundAnalyzer::analyze(&navs, days, &name, benchmark.as_ref(), meta.as_ref()) {
            Some(a) => a,
            None => return Ok(None),
        };
    let rolling = normalize_rolling_window(rolling_window);
    let series = build_fund_analysis_series(&navs, benchmark.as_ref(), rolling);
    let benchmark_label = benchmark.map(|b| b.label);
    Ok(Some(FundAnalysisReport {
        snapshot,
        series,
        benchmark_label,
    }))
}
