//! 分析页与组合页图表组件。

use super::charts::{svg_line_chart, ChartValueKind};
use crate::models::{FundAnalysisSeries, PortfolioTimeSeries};
use leptos::prelude::*;

#[component]
pub fn AnalysisCharts(
    series: FundAnalysisSeries,
    benchmark_label: Option<String>,
) -> impl IntoView {
    let window = series.rolling_window;
    let beta_title = benchmark_label
        .map(|l| format!("滚动 Beta（{l}，{window} 日）"))
        .unwrap_or_else(|| format!("滚动 Beta（{window} 日）"));

    view! {
        <section class="card charts-card">
            <h2>"净值与滚动指标"</h2>
            <p class="muted">"滚动窗口 " {window} " 个交易日；曲线基于对齐后的日收益计算。"</p>
            <div class="chart-grid">
                <div class="chart-panel" inner_html=svg_line_chart(
                    &series.nav_normalized,
                    420,
                    180,
                    "归一化净值",
                    ChartValueKind::Number,
                ) />
                <div class="chart-panel" inner_html=svg_line_chart(
                    &series.drawdown,
                    420,
                    180,
                    "回撤曲线",
                    ChartValueKind::Percent,
                ) />
                <div class="chart-panel" inner_html=svg_line_chart(
                    &series.rolling_sharpe,
                    420,
                    180,
                    &format!("滚动夏普（{window} 日）"),
                    ChartValueKind::Number,
                ) />
                <div class="chart-panel" inner_html=svg_line_chart(
                    &series.rolling_volatility,
                    420,
                    180,
                    &format!("滚动波动率（{window} 日）"),
                    ChartValueKind::Percent,
                ) />
                {(!series.rolling_beta.is_empty()).then(|| view! {
                    <div class="chart-panel" inner_html=svg_line_chart(
                        &series.rolling_beta,
                        420,
                        180,
                        &beta_title,
                        ChartValueKind::Number,
                    ) />
                })}
            </div>
        </section>
    }
}

#[component]
pub fn PortfolioCharts(series: PortfolioTimeSeries) -> impl IntoView {
    let window = series.rolling_window;
    view! {
        <section class="card charts-card">
            <h2>"组合净值与滚动指标"</h2>
            <p class="muted">"加权组合曲线 · 滚动窗口 " {window} " 个交易日"</p>
            <div class="chart-grid">
                <div class="chart-panel" inner_html=svg_line_chart(
                    &series.nav_normalized,
                    420,
                    180,
                    "组合归一化净值",
                    ChartValueKind::Number,
                ) />
                <div class="chart-panel" inner_html=svg_line_chart(
                    &series.drawdown,
                    420,
                    180,
                    "组合回撤",
                    ChartValueKind::Percent,
                ) />
                <div class="chart-panel" inner_html=svg_line_chart(
                    &series.rolling_sharpe,
                    420,
                    180,
                    &format!("滚动夏普（{window} 日）"),
                    ChartValueKind::Number,
                ) />
                <div class="chart-panel" inner_html=svg_line_chart(
                    &series.rolling_volatility,
                    420,
                    180,
                    &format!("滚动波动率（{window} 日）"),
                    ChartValueKind::Percent,
                ) />
            </div>
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::SeriesPoint;
    use chrono::NaiveDate;

    #[test]
    fn analysis_charts_renders_svg() {
        let series = FundAnalysisSeries {
            rolling_window: 60,
            nav_normalized: (0..5)
                .map(|i| SeriesPoint {
                    date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap() + chrono::Duration::days(i),
                    value: 1.0 + i as f64 * 0.01,
                })
                .collect(),
            drawdown: (0..5)
                .map(|i| SeriesPoint {
                    date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap() + chrono::Duration::days(i),
                    value: -0.01 * i as f64,
                })
                .collect(),
            rolling_sharpe: (0..5)
                .map(|i| SeriesPoint {
                    date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap() + chrono::Duration::days(i),
                    value: 1.0 + i as f64 * 0.1,
                })
                .collect(),
            rolling_beta: vec![],
            rolling_volatility: (0..5)
                .map(|i| SeriesPoint {
                    date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap() + chrono::Duration::days(i),
                    value: 0.1 + i as f64 * 0.01,
                })
                .collect(),
        };
        let html = view! {
            <AnalysisCharts series=series benchmark_label=None />
        }
        .to_html();
        assert!(html.contains("chart-svg"));
        assert!(html.contains("归一化净值"));
    }
}
