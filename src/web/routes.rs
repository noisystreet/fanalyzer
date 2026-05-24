//! Axum 路由与 SSR 渲染。

use super::components::{AnalyzePage, ComparePage, HomePage};
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

    let (analysis, error) = if code.trim().is_empty() {
        (None, None)
    } else {
        match services::analyze_one(&state, &code, days, period_opt).await {
            Ok(Some(a)) => (Some(a), None),
            Ok(None) => (None, Some("净值数据不足，无法完成分析".into())),
            Err(e) => (None, Some(e.to_string())),
        }
    };

    render(view! {
        <AnalyzePage
            code=code
            days=days
            period=period
            analysis=analysis
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

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(home))
        .route("/analyze", get(analyze))
        .route("/compare", get(compare))
        .with_state(state)
}
