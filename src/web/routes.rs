//! Axum 路由与 SSR 渲染。

use super::components::{
    AnalyzePage, BriefPage, ComparePage, DisclaimerPage, HomePage, InfoPage, PortfolioPage,
};
use super::services;
use super::state::AppState;
use axum::{
    extract::{Query, State},
    response::Html,
    routing::get,
    Router,
};
use leptos::prelude::*;
use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
struct AnalyzeParams {
    code: Option<String>,
    days: Option<u32>,
    period: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct CompareParams {
    codes: Option<String>,
    days: Option<u32>,
    period: Option<String>,
    sort: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct InfoParams {
    code: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct BriefParams {
    code: Option<String>,
    days: Option<u32>,
    period: Option<String>,
    industry_top: Option<u32>,
    holdings_top: Option<u32>,
}

#[derive(Debug, Deserialize, Default)]
struct PortfolioParams {
    name: Option<String>,
    holdings: Option<String>,
    days: Option<u32>,
    period: Option<String>,
    holdings_top: Option<u32>,
    /// 表单提交时为 "1"
    run: Option<String>,
}

fn render<V>(view: V) -> Html<String>
where
    V: RenderHtml,
{
    Html(view.to_html())
}

async fn home() -> Html<String> {
    render(view! { <HomePage /> })
}

async fn analyze(State(state): State<AppState>, Query(q): Query<AnalyzeParams>) -> Html<String> {
    let code = q.code.unwrap_or_default();
    let days = q.days.unwrap_or(90);
    let period = q.period.unwrap_or_default();
    let period_opt = (!period.is_empty()).then_some(period.as_str());

    let (report, error) = if code.trim().is_empty() {
        (None, None)
    } else {
        match services::analyze_one(&state, &code, days, period_opt).await {
            Ok(Some(r)) => (Some(r), None),
            Ok(None) => (None, Some("净值数据不足，无法完成分析".into())),
            Err(e) => (None, Some(e.to_string())),
        }
    };

    render(view! {
        <AnalyzePage
            code=code
            days=days
            period=period
            report=report
            error=error
        />
    })
}

async fn compare(State(state): State<AppState>, Query(q): Query<CompareParams>) -> Html<String> {
    let codes_raw = q.codes.unwrap_or_default();
    let days = q.days.unwrap_or(90);
    let period = q.period.unwrap_or_default();
    let sort = q.sort.unwrap_or_default();
    let period_opt = (!period.is_empty()).then_some(period.as_str());

    let (analyses, error) = if codes_raw.trim().is_empty() {
        (Vec::new(), None)
    } else {
        let list = services::parse_code_list(&codes_raw);
        if list.len() < 2 {
            (Vec::new(), Some("对比至少需要 2 只基金".into()))
        } else {
            match services::compare_funds(
                &state,
                &list,
                days,
                period_opt,
                Some(sort.as_str()).filter(|s| !s.is_empty()),
            )
            .await
            {
                Ok(rows) if rows.len() >= 2 => (rows, None),
                Ok(_) => (
                    Vec::new(),
                    Some("有效样本不足（需 ≥2）；请检查代码或网络".into()),
                ),
                Err(e) => (Vec::new(), Some(e.to_string())),
            }
        }
    };

    render(view! {
        <ComparePage
            codes=codes_raw
            days=days
            period=period
            sort=sort
            analyses=analyses
            error=error
        />
    })
}

async fn info(State(state): State<AppState>, Query(q): Query<InfoParams>) -> Html<String> {
    let code = q.code.unwrap_or_default();
    let (profile, error) = if code.trim().is_empty() {
        (None, None)
    } else {
        match services::fetch_overview(&state, &code).await {
            Ok(p) => (Some(p), None),
            Err(e) => (None, Some(e.to_string())),
        }
    };

    render(view! { <InfoPage code=code profile=profile error=error /> })
}

async fn brief(State(state): State<AppState>, Query(q): Query<BriefParams>) -> Html<String> {
    let code = q.code.unwrap_or_default();
    let days = q.days.unwrap_or(90);
    let period = q.period.unwrap_or_default();
    let industry_top = q.industry_top.unwrap_or(5).clamp(1, 20);
    let holdings_top = q.holdings_top.unwrap_or(10).clamp(1, 50);
    let period_opt = (!period.is_empty()).then_some(period.as_str());

    let (brief, error) = if code.trim().is_empty() {
        (None, None)
    } else {
        match services::build_brief(&state, &code, days, period_opt, industry_top, holdings_top)
            .await
        {
            Ok(b) => (Some(b), None),
            Err(e) => (None, Some(e.to_string())),
        }
    };

    render(view! {
        <BriefPage
            code=code
            days=days
            period=period
            industry_top=industry_top
            holdings_top=holdings_top
            brief=brief
            error=error
        />
    })
}

async fn disclaimer() -> Html<String> {
    render(view! { <DisclaimerPage /> })
}

async fn portfolio(
    State(state): State<AppState>,
    Query(q): Query<PortfolioParams>,
) -> Html<String> {
    let days = q.days.unwrap_or(90);
    let period = q.period.unwrap_or_default();
    let holdings_top = q.holdings_top.unwrap_or(10).clamp(1, 50);
    let period_opt = (!period.is_empty()).then_some(period.as_str());
    let should_run = q.run.as_deref() == Some("1");

    let (default_name, default_holdings) = crate::portfolio::default_editor_content(
        &state.inner.portfolio_path,
        &state.inner.watchlist_path,
    );
    let portfolio_name = q
        .name
        .filter(|s| !s.trim().is_empty())
        .unwrap_or(default_name);
    let holdings_text = q
        .holdings
        .filter(|s| !s.trim().is_empty())
        .unwrap_or(default_holdings);

    let (report, error) = if should_run {
        match crate::portfolio::portfolio_from_text(Some(&portfolio_name), &holdings_text) {
            Ok(def) => {
                match services::analyze_portfolio(&state, &def, days, period_opt, holdings_top)
                    .await
                {
                    Ok(r) => (Some(r), None),
                    Err(e) => (None, Some(e.to_string())),
                }
            }
            Err(e) => (None, Some(e.to_string())),
        }
    } else {
        (None, None)
    };

    render(view! {
        <PortfolioPage
            portfolio_name=portfolio_name
            holdings_text=holdings_text
            days=days
            period=period
            holdings_top=holdings_top
            report=report
            error=error
        />
    })
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(home))
        .route("/analyze", get(analyze))
        .route("/compare", get(compare))
        .route("/portfolio", get(portfolio))
        .route("/info", get(info))
        .route("/brief", get(brief))
        .route("/disclaimer", get(disclaimer))
        .with_state(state)
}
