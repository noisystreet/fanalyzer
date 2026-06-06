//! 单基金选基综合简报：分析 + 行业 + 重仓。

use super::concurrency::{map_concurrent, FUND_CONCURRENCY};
use super::context::{require_online, resolve_fund_ids, CommandContext};
use super::fund_service::{analyze_fund, resolve_fund_identifier};
use super::mappers::{map_holdings, map_industry};
use crate::domain::resolve_analysis_days;
use crate::models::FundBrief;
use crate::presentation::{
    base_meta, compact_brief_summary, emit, item_error_failed, print_brief_separator,
    render_brief_terminal, write_brief_markdown, AnalysisMeta, BatchPayload, ItemError,
};
use chrono::Local;

/// `brief` 请求参数。
pub struct BriefRequest {
    pub code: Option<String>,
    pub pick_watchlist: bool,
    pub days: u32,
    pub period: Option<String>,
    pub industry_top: u32,
    pub holdings_top: u32,
    pub output: Option<std::path::PathBuf>,
}

enum BriefBatchOutcome {
    Ok(Box<FundBrief>),
    Err(ItemError),
}

async fn brief_one_for_batch(
    session: &super::context::Session<'_>,
    id: String,
    days: u32,
    holdings_top: u32,
    industry_top: u32,
) -> BriefBatchOutcome {
    match gather_brief(session, &id, days, holdings_top, industry_top).await {
        Ok(brief) => BriefBatchOutcome::Ok(Box::new(brief)),
        Err(e) => BriefBatchOutcome::Err(item_error_failed(&id, e)),
    }
}

pub async fn run_brief(ctx: &CommandContext<'_>, req: BriefRequest) -> anyhow::Result<()> {
    require_online(ctx.offline, "brief")?;
    let today = Local::now().date_naive();
    let days = resolve_analysis_days(req.period.as_deref(), req.days, today)?;
    let ids = resolve_fund_ids(
        req.code,
        req.pick_watchlist,
        ctx.watchlist_path,
        "--code/--watchlist",
    )?;
    let requested = ids.len();
    let multi = requested > 1;

    if ctx.structured() {
        let outcomes = map_concurrent(&ids, FUND_CONCURRENCY, |id| {
            brief_one_for_batch(&ctx.session, id, days, req.holdings_top, req.industry_top)
        })
        .await;
        let mut errors = Vec::new();
        let mut items: Vec<FundBrief> = outcomes
            .into_iter()
            .filter_map(|outcome| match outcome {
                BriefBatchOutcome::Ok(brief) => Some(*brief),
                BriefBatchOutcome::Err(err) => {
                    errors.push(err);
                    None
                }
            })
            .collect();
        if items.is_empty() {
            anyhow::bail!("无有效简报结果");
        }
        if !errors.is_empty() {
            ctx.warn(format!("{} 只标的简报生成失败", errors.len()));
        }
        if ctx.summary_mode() {
            for item in &mut items {
                compact_brief_summary(item);
            }
        }
        let meta = AnalysisMeta {
            base: base_meta(ctx),
            days,
            period: req.period.clone(),
            rolling_window: None,
            requested,
            succeeded: items.len(),
        };
        emit(
            ctx,
            "brief",
            &BatchPayload { items, errors },
            Some(&meta),
            None,
        )?;
        return Ok(());
    }

    for id in ids {
        match gather_brief(&ctx.session, &id, days, req.holdings_top, req.industry_top).await {
            Ok(brief) => {
                render_brief_terminal(&brief);
                if let Some(ref path) = req.output {
                    write_brief_markdown(&brief, path)?;
                    tracing::info!(path = %path.display(), "Wrote brief markdown");
                }
                if multi {
                    print_brief_separator();
                }
            }
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

pub async fn gather_brief(
    session: &super::context::Session<'_>,
    identifier: &str,
    days: u32,
    holdings_top: u32,
    industry_top: u32,
) -> anyhow::Result<FundBrief> {
    let (code, name) = resolve_fund_identifier(session, identifier, false).await?;
    tracing::info!(code = %code, days = days, "Building fund brief");

    let analysis = analyze_fund(
        session,
        &code,
        days,
        false,
        crate::domain::DEFAULT_ROLLING_WINDOW,
    )
    .await?
    .map(|r| r.snapshot);

    let profile = session.source.fetch_fund_profile(&code).await.ok();
    let fund_type = profile
        .as_ref()
        .map(|p| p.fund_type.clone())
        .unwrap_or_default();
    let company = profile
        .as_ref()
        .map(|p| p.company.clone())
        .unwrap_or_default();
    let asset_size = profile
        .as_ref()
        .map(|p| p.asset_size.clone())
        .unwrap_or_default();
    let display_name = profile
        .as_ref()
        .map(|p| p.name.clone())
        .filter(|n| !n.is_empty())
        .unwrap_or(name);

    let industry_api = session
        .source
        .fetch_fund_industry_allocation(&code)
        .await
        .unwrap_or_default();
    let holdings_api = session
        .source
        .fetch_fund_stock_holdings(&code, holdings_top.clamp(1, 50))
        .await
        .unwrap_or_default();

    Ok(FundBrief {
        code,
        name: display_name,
        fund_type,
        company,
        asset_size,
        days,
        analysis,
        industry: map_industry(&industry_api),
        holdings: map_holdings(&holdings_api),
        industry_top: industry_top as usize,
        holdings_top: holdings_top as usize,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::data_source::mock::MockFundDataSource;
    use crate::application::output_profile::OutputProfile;
    use crate::application::test_support::{linear_nav_series, strip_volatile_envelope_fields};
    use crate::application::{CommandContext, FundDataSource, StructuredOutput};
    use crate::cache::FundCache;
    use crate::nav_cache::NavCache;
    use std::path::Path;
    use std::sync::Arc;
    use tempfile::tempdir;
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn brief_golden_envelope_with_mock_session() {
        let code = "000001";
        let navs = linear_nav_series(code, 91);
        let mock = MockFundDataSource::with_navs(code, "简报测试基金", navs);
        let dir = tempdir().unwrap();
        let cache_root = dir.path().join("cache");
        let name_cache = Arc::new(Mutex::new(FundCache::with_root(cache_root.clone())));
        let nav_store = NavCache::with_root(cache_root);
        let ctx = CommandContext::new(
            &mock as &dyn FundDataSource,
            &name_cache,
            &nav_store,
            false,
            Path::new("config/watchlist.toml"),
            StructuredOutput::for_capture(OutputProfile::Standard),
        );
        run_brief(
            &ctx,
            BriefRequest {
                code: Some(code.into()),
                pick_watchlist: false,
                days: 90,
                period: None,
                industry_top: 5,
                holdings_top: 10,
                output: None,
            },
        )
        .await
        .unwrap();

        let raw = ctx.take_captured().expect("captured json");
        let stable = strip_volatile_envelope_fields(serde_json::from_str(&raw).unwrap());
        assert_eq!(stable["ok"], true);
        assert_eq!(stable["command"], "brief");
        assert_eq!(stable["data"]["items"][0]["code"], "000001");
        assert_eq!(stable["data"]["items"][0]["name"], "简报测试基金");
        assert!(stable["data"]["items"][0]["analysis"].is_object());
    }
}
