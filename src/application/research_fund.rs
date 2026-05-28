//! 单基金研究复合编排（MCP `research_fund`）。

use super::context::Session;
use super::fund_service::{analyze_fund_with_navs, fetch_nav_series, resolve_fund_identifier};
use super::queries::{
    load_fund_holdings_resolved, load_fund_overview_resolved, load_sectors_resolved,
};
use crate::models::FundAnalysisReport;
use crate::presentation::{
    error_from_anyhow, failure_envelope_json, item_error_insufficient, success_envelope_json,
    AnalysisMeta, BaseMeta, BatchMeta, BatchPayload, HoldingsItem, SectorItem,
};
use anyhow::Context;
use serde_json::{json, Map, Value};
use std::time::Instant;

pub struct ResearchFundResult {
    pub ok: bool,
    pub steps: Map<String, Value>,
    pub duration_ms: u64,
    pub offline: bool,
}

/// 单基金研究：一次 resolve + 一次净值，四步并行拉取/分析。
pub async fn gather_research_fund(
    session: &Session<'_>,
    identifier: &str,
    days: u32,
    offline: bool,
    rolling_window: u32,
    holdings_top: u32,
) -> anyhow::Result<ResearchFundResult> {
    let started = Instant::now();
    let (resolved_code, name) = resolve_fund_identifier(session, identifier, offline).await?;
    let navs = fetch_nav_series(session, &resolved_code, days, offline)
        .await
        .context("拉取净值失败")?;

    let rolling = rolling_window;
    let code = resolved_code.clone();
    let name_for = name.clone();
    let top = holdings_top.clamp(1, 50);

    let (info_r, analyze_r, sectors_r, holdings_r) = tokio::join!(
        async {
            if offline {
                Err(anyhow::anyhow!("`--offline` 无法获取 info（需联网）"))
            } else {
                load_fund_overview_resolved(session, &code).await
            }
        },
        async {
            analyze_fund_with_navs(session, &code, &name_for, &navs, days, offline, rolling).await
        },
        async {
            if offline {
                Err(anyhow::anyhow!("`--offline` 无法获取 sectors（需联网）"))
            } else {
                load_sectors_resolved(session, &code, &name_for).await
            }
        },
        async {
            if offline {
                Err(anyhow::anyhow!("`--offline` 无法获取 holdings（需联网）"))
            } else {
                load_fund_holdings_resolved(session, &code, &name_for, top).await
            }
        },
    );

    let mut steps = Map::new();
    let mut any_error = false;

    any_error |= push_info_step(&mut steps, info_r, offline);
    any_error |= push_analyze_step(
        &mut steps,
        analyze_r,
        &resolved_code,
        days,
        offline,
        rolling,
    );
    any_error |= push_sectors_step(&mut steps, sectors_r, offline);
    any_error |= push_holdings_step(&mut steps, holdings_r, offline);

    Ok(ResearchFundResult {
        ok: !any_error,
        steps,
        duration_ms: started.elapsed().as_millis() as u64,
        offline,
    })
}

impl ResearchFundResult {
    pub fn to_envelope_json(&self) -> anyhow::Result<String> {
        let envelope = json!({
            "v": 1,
            "command": "research_fund",
            "ok": self.ok,
            "warnings": [],
            "meta": {
                "offline": self.offline,
                "steps_completed": self.steps.len() as u32,
                "duration_ms": self.duration_ms,
            },
            "data": self.steps,
        });
        Ok(serde_json::to_string(&envelope)?)
    }
}

fn push_info_step(
    steps: &mut Map<String, Value>,
    result: anyhow::Result<crate::models::FundOverview>,
    offline: bool,
) -> bool {
    match result {
        Ok(item) => match step_success(
            steps,
            "info",
            BatchPayload {
                items: vec![item],
                errors: vec![],
            },
            batch_meta(offline, 1, 1),
        ) {
            Ok(()) => false,
            Err(e) => {
                insert_step_failure(steps, "info", e);
                true
            }
        },
        Err(e) => {
            insert_step_failure(steps, "info", e);
            true
        }
    }
}

fn push_analyze_step(
    steps: &mut Map<String, Value>,
    result: anyhow::Result<Option<FundAnalysisReport>>,
    code: &str,
    days: u32,
    offline: bool,
    rolling_window: u32,
) -> bool {
    match result {
        Ok(Some(report)) => {
            let meta = AnalysisMeta {
                base: base_meta(offline),
                days,
                period: None,
                rolling_window: Some(rolling_window),
                requested: 1,
                succeeded: 1,
            };
            match step_success(
                steps,
                "analyze",
                BatchPayload {
                    items: vec![report],
                    errors: vec![],
                },
                meta,
            ) {
                Ok(()) => false,
                Err(e) => {
                    insert_step_failure(steps, "analyze", e);
                    true
                }
            }
        }
        Ok(None) => {
            let err = item_error_insufficient(code);
            insert_step_failure(steps, "analyze", anyhow::anyhow!(err.message));
            true
        }
        Err(e) => {
            insert_step_failure(steps, "analyze", e);
            true
        }
    }
}

fn push_sectors_step(
    steps: &mut Map<String, Value>,
    result: anyhow::Result<SectorItem>,
    offline: bool,
) -> bool {
    match result {
        Ok(item) => match step_success(
            steps,
            "sectors",
            BatchPayload {
                items: vec![item],
                errors: vec![],
            },
            batch_meta(offline, 1, 1),
        ) {
            Ok(()) => false,
            Err(e) => {
                insert_step_failure(steps, "sectors", e);
                true
            }
        },
        Err(e) => {
            insert_step_failure(steps, "sectors", e);
            true
        }
    }
}

fn push_holdings_step(
    steps: &mut Map<String, Value>,
    result: anyhow::Result<HoldingsItem>,
    offline: bool,
) -> bool {
    match result {
        Ok(item) => match step_success(
            steps,
            "holdings",
            BatchPayload {
                items: vec![item],
                errors: vec![],
            },
            batch_meta(offline, 1, 1),
        ) {
            Ok(()) => false,
            Err(e) => {
                insert_step_failure(steps, "holdings", e);
                true
            }
        },
        Err(e) => {
            insert_step_failure(steps, "holdings", e);
            true
        }
    }
}

fn batch_meta(offline: bool, requested: usize, succeeded: usize) -> BatchMeta {
    BatchMeta {
        base: base_meta(offline),
        requested,
        succeeded,
    }
}

fn base_meta(offline: bool) -> BaseMeta {
    BaseMeta {
        offline,
        generated_at: chrono::Local::now().to_rfc3339(),
        duration_ms: None,
    }
}

fn step_success<T: serde::Serialize, M: serde::Serialize>(
    steps: &mut Map<String, Value>,
    command: &'static str,
    data: T,
    meta: M,
) -> anyhow::Result<()> {
    let json = success_envelope_json(command, &data, Some(&meta), &[])?;
    steps.insert(command.into(), parse_step_json(json));
    Ok(())
}

fn parse_step_json(raw: String) -> Value {
    serde_json::from_str(&raw).unwrap_or(Value::Null)
}

fn insert_step_failure(steps: &mut Map<String, Value>, command: &str, err: anyhow::Error) {
    let structured = error_from_anyhow(&err);
    let json = failure_envelope_json(command, &structured, &[]).unwrap_or_else(|_| {
        json!({
            "v": 1,
            "command": command,
            "ok": false,
            "warnings": [],
            "error": {"code": "COMMAND_FAILED", "message": err.to_string()}
        })
        .to_string()
    });
    steps.insert(command.into(), parse_step_json(json));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_support::seed_offline_two_funds;
    use crate::application::{CommandContext, FundDataSource, StructuredOutput};
    use std::path::Path;

    #[tokio::test]
    async fn gather_research_fund_offline_analyze_step_succeeds() {
        let dir = tempfile::tempdir().unwrap();
        let cache_root = dir.path().join("cache");
        let (nav_store, name_cache) =
            seed_offline_two_funds(&cache_root, &[("000001", "基金A")]).await;
        let client = crate::api::eastmoney::EastMoneyClient::default();
        let ctx = CommandContext::new(
            &client as &dyn FundDataSource,
            &name_cache,
            &nav_store,
            true,
            Path::new("config/watchlist.toml"),
            StructuredOutput::OFF,
        );
        let result = gather_research_fund(&ctx.session, "000001", 90, true, 60, 10)
            .await
            .unwrap();
        assert!(!result.ok);
        assert_eq!(result.steps["analyze"]["ok"], true);
        assert_eq!(result.steps["info"]["ok"], false);
    }
}
