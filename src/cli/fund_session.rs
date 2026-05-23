//! 基金解析、净值序列、分析等 CLI 共用异步逻辑。

use crate::api::eastmoney::EastMoneyClient;
use crate::cache::FundCache;
use crate::models::FundAnalysis;
use crate::nav_cache::{filter_covering_calendar_days, NavCache};
use crate::services::{BenchmarkData, FundAnalyzer, FundMetaInfo};
use anyhow::Context;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn fetch_nav_series(
    client: &EastMoneyClient,
    nav_store: &NavCache,
    resolved_code: &str,
    days: u32,
    offline: bool,
) -> anyhow::Result<Vec<crate::models::FundNav>> {
    if offline {
        let loaded = nav_store.load(resolved_code).with_context(|| {
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
        let navs = client
            .fetch_nav_history_by_days(resolved_code, days)
            .await?;
        if !navs.is_empty() && nav_store.save_merged(resolved_code, &navs).is_err() {
            tracing::warn!("写入净值缓存失败（已忽略）：{}", resolved_code);
        }
        Ok(navs)
    }
}

pub async fn resolve_fund_identifier(
    client: &EastMoneyClient,
    cache: &Arc<Mutex<FundCache>>,
    identifier: &str,
    offline: bool,
) -> anyhow::Result<(String, String)> {
    let is_likely_code = identifier.chars().all(|c| c.is_ascii_digit()) && identifier.len() == 6;

    if is_likely_code {
        let name = if offline {
            let g = cache.lock().await;
            g.get_name(identifier)
                .unwrap_or_else(|| identifier.to_string())
        } else {
            get_fund_name(client, cache, identifier).await
        };
        return Ok((identifier.to_string(), name));
    }

    if offline {
        let code = cache.lock().await.get_code(identifier).ok_or_else(|| {
            anyhow::anyhow!(
                "`--offline` 无法解析名称 `{id}`，请先在线跑一次或直接使用 6 位代码",
                id = identifier
            )
        })?;
        return Ok((code, identifier.to_string()));
    }

    {
        let cache_guard = cache.lock().await;
        if let Some(code) = cache_guard.get_code(identifier) {
            return Ok((code, identifier.to_string()));
        }
    }

    match client.search_fund(identifier).await {
        Ok(results) => {
            if let Some((code, name)) = results.first() {
                let mut cache_guard = cache.lock().await;
                cache_guard.set_mapping(code, name);
                Ok((code.clone(), name.clone()))
            } else {
                anyhow::bail!("未找到与 `{identifier}` 匹配的基金")
            }
        }
        Err(e) => anyhow::bail!("基金搜索失败：{e}"),
    }
}

pub async fn get_benchmark_data(client: &EastMoneyClient, days: u32) -> Option<BenchmarkData> {
    const HS300_CODE: &str = "1.000300";

    match client.fetch_index_history(HS300_CODE, 1, days * 2).await {
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

            Some(BenchmarkData { dates, returns })
        }
        Err(e) => {
            tracing::warn!(error = %e, "Failed to fetch benchmark data");
            None
        }
    }
}

pub async fn get_fund_meta(client: &EastMoneyClient, code: &str) -> Option<FundMetaInfo> {
    let manager = match client.fetch_fund_manager(code).await {
        Ok(m) => m,
        Err(e) => {
            tracing::warn!(code = %code, error = %e, "Failed to fetch fund manager");
            return None;
        }
    };

    let fee = match client.fetch_fund_fee(code).await {
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

async fn get_fund_name(
    client: &EastMoneyClient,
    cache: &Arc<Mutex<FundCache>>,
    code: &str,
) -> String {
    {
        let cache_guard = cache.lock().await;
        if let Some(name) = cache_guard.get_name(code) {
            return name;
        }
    }

    match client.fetch_fund_name(code).await {
        Ok(name) => {
            let mut cache_guard = cache.lock().await;
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
    client: &EastMoneyClient,
    cache: &Arc<Mutex<FundCache>>,
    nav_store: &NavCache,
    identifier: &str,
    days: u32,
    offline: bool,
) -> anyhow::Result<Option<FundAnalysis>> {
    let (resolved_code, name) = resolve_fund_identifier(client, cache, identifier, offline).await?;
    let benchmark = if offline {
        None
    } else {
        get_benchmark_data(client, days).await
    };
    let meta = if offline {
        None
    } else {
        get_fund_meta(client, &resolved_code).await
    };
    let navs = fetch_nav_series(client, nav_store, &resolved_code, days, offline).await?;
    if navs.is_empty() {
        return Ok(None);
    }
    Ok(FundAnalyzer::analyze(
        &navs,
        days,
        &name,
        benchmark.as_ref(),
        meta.as_ref(),
    ))
}
