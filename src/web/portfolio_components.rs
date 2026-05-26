//! 组合分析 Web 组件。

use super::super::chart_components::PortfolioCharts;
use super::super::portfolio_draft::portfolio_draft_script;
use super::{num, pct, ErrorAlert, Layout};
use crate::models::{CorrelationMatrix, OverlapPair, PortfolioInterpretation, PortfolioReport};
use leptos::prelude::*;

#[component]
pub fn PortfolioInsightsPanel(interp: PortfolioInterpretation) -> impl IntoView {
    view! {
        <section class="card insights-card">
            <h2>"分析解读"</h2>
            <p class="insights-headline">{interp.headline.clone()}</p>
            <ul class="insights-list">
                {interp.insights.into_iter().map(|item| {
                    let class = match item.level {
                        crate::models::InsightLevel::Positive => "insight insight-positive",
                        crate::models::InsightLevel::Info => "insight insight-info",
                        crate::models::InsightLevel::Caution => "insight insight-caution",
                    };
                    view! {
                        <li class=class>
                            <span class="insight-tag">{insight_tag(item.level)}</span>
                            {item.message.clone()}
                        </li>
                    }
                }).collect_view()}
            </ul>
            <p class="muted form-hint">"以上为规则化参考解读，不构成投资建议；请结合 F10 与自身风险承受能力判断。"</p>
        </section>
    }
}

fn insight_tag(level: crate::models::InsightLevel) -> &'static str {
    match level {
        crate::models::InsightLevel::Positive => "正面",
        crate::models::InsightLevel::Info => "参考",
        crate::models::InsightLevel::Caution => "注意",
    }
}

#[component]
pub fn PortfolioMetricsCard(summary: crate::models::PortfolioSummary) -> impl IntoView {
    view! {
        <section class="card">
            <h2>{summary.name.clone()}</h2>
            <p class="muted">
                "分析窗口 " {summary.period_days} " 日历天 · 对齐交易日 " {summary.aligned_days} " 天"
            </p>
            <div class="table-scroll">
            <table class="metrics">
                <tbody>
                    <tr><th>"组合总收益率"</th><td>{pct(summary.total_return)}</td></tr>
                    <tr><th>"组合年化收益率"</th><td>{pct(summary.annualized_return)}</td></tr>
                    <tr><th>"组合波动率"</th><td>{pct(summary.volatility)}</td></tr>
                    <tr><th>"组合最大回撤"</th><td>{pct(summary.max_drawdown)}</td></tr>
                    <tr><th>"组合夏普比率"</th><td>{num(summary.sharpe_ratio)}</td></tr>
                </tbody>
            </table>
            </div>
        </section>
    }
}

#[component]
pub fn PortfolioMembersTable(members: Vec<crate::models::PortfolioMember>) -> impl IntoView {
    view! {
        <section class="card">
            <h2>"成分基金与静态贡献"</h2>
            <div class="table-scroll">
            <table class="compare">
                <thead>
                    <tr>
                        <th>"代码"</th>
                        <th>"名称"</th>
                        <th>"权重"</th>
                        <th>"总收益"</th>
                        <th>"波动率"</th>
                        <th>"回撤"</th>
                        <th>"夏普"</th>
                        <th>"贡献"</th>
                    </tr>
                </thead>
                <tbody>
                    {members.into_iter().map(|m| view! {
                        <tr>
                            <td>{m.code.clone()}</td>
                            <td>{m.name.clone()}</td>
                            <td>{pct(m.weight)}</td>
                            <td>{pct(m.total_return)}</td>
                            <td>{pct(m.volatility)}</td>
                            <td>{pct(m.max_drawdown)}</td>
                            <td>{num(m.sharpe_ratio)}</td>
                            <td>{pct(m.return_contribution)}</td>
                        </tr>
                    }).collect_view()}
                </tbody>
            </table>
            </div>
        </section>
    }
}

#[component]
pub fn CorrelationTable(matrix: CorrelationMatrix) -> impl IntoView {
    let labels = matrix.labels.clone();
    let values = matrix.values.clone();
    view! {
        <section class="card">
            <h2>"日收益相关矩阵"</h2>
            <div class="table-scroll">
            <table class="compare">
                <thead>
                    <tr>
                        <th></th>
                        {labels.iter().map(|l| view! { <th>{l.clone()}</th> }).collect_view()}
                    </tr>
                </thead>
                <tbody>
                    {values.into_iter().enumerate().map(|(i, row)| {
                        let label = labels[i].clone();
                        view! {
                            <tr>
                                <th>{label}</th>
                                {row.into_iter().map(|v| view! {
                                    <td>{format!("{:.3}", v)}</td>
                                }).collect_view()}
                            </tr>
                        }
                    }).collect_view()}
                </tbody>
            </table>
            </div>
        </section>
    }
}

#[component]
pub fn OverlapTable(pairs: Vec<OverlapPair>) -> impl IntoView {
    view! {
        <section class="card">
            <h2>"重仓股加权重叠"</h2>
            <div class="table-scroll">
            <table class="compare">
                <thead>
                    <tr>
                        <th>"基金 A"</th>
                        <th>"基金 B"</th>
                        <th>"重叠%"</th>
                        <th>"共同持仓"</th>
                    </tr>
                </thead>
                <tbody>
                    {pairs.into_iter().map(|p| view! {
                        <tr>
                            <td>{p.fund_a_code.clone()} " " {p.fund_a_name.clone()}</td>
                            <td>{p.fund_b_code.clone()} " " {p.fund_b_name.clone()}</td>
                            <td>{format!("{:.2}", p.overlap_pct)}</td>
                            <td>{p.shared_count}</td>
                        </tr>
                    }).collect_view()}
                </tbody>
            </table>
            </div>
        </section>
    }
}

#[component]
pub fn PortfolioPage(
    portfolio_name: String,
    holdings_text: String,
    days: u32,
    period: String,
    holdings_top: u32,
    rolling_window: u32,
    report: Option<PortfolioReport>,
    error: Option<String>,
) -> impl IntoView {
    let draft_script = portfolio_draft_script();
    view! {
        <Layout title="组合分析".into()>
            <section class="card">
                <div class="page-header">
                    <h1>"组合分析"</h1>
                    <p class="muted">"在下方编辑自选组合（基金代码或名称 + 权重），提交后按此配置分析。"</p>
                </div>
                <form class="query-form portfolio-form" method="get" action="/portfolio">
                    <input type="hidden" name="run" value="1"/>
                    <div class="form-grid">
                        <label class="field field-wide">"组合名称"
                            <input name="name" type="text" placeholder="my-portfolio" value=portfolio_name />
                        </label>
                        <label class="field field-wide">"自选组合（每行：代码 权重）"
                            <textarea
                                name="holdings"
                                rows="6"
                                placeholder="000001 0.5\n110011 0.5"
                            >{holdings_text.clone()}</textarea>
                        </label>
                        <label class="field">"日历天"
                            <input name="days" type="number" min="7" value=days.to_string() />
                        </label>
                        <label class="field">"period（可选）"
                            <input name="period" type="text" placeholder="1y / 3m / ytd" value=period />
                        </label>
                        <label class="field">"重仓重叠 Top N"
                            <input name="holdings_top" type="number" min="1" max="50" value=holdings_top.to_string() />
                        </label>
                        <label class="field">"滚动窗口（交易日）"
                            <input name="rolling_window" type="number" min="10" max="252" value=rolling_window.to_string() />
                        </label>
                    </div>
                    <p class="muted form-hint">"支持空格/逗号分隔；# 开头为注释。权重合计不为 1 时将自动归一化。编辑内容会自动暂存到浏览器；首次打开会预填 portfolio.toml 或自选等权。"</p>
                    <div class="form-actions">
                        <a class="btn btn-secondary" href="/portfolio?import=watchlist">"从自选导入等权"</a>
                        <button type="submit" class="btn btn-primary">"分析组合"</button>
                    </div>
                </form>
            </section>
            {error.map(|e| view! { <ErrorAlert message=e /> })}
            {report.as_ref().and_then(|r| r.interpretation.clone()).map(|i| view! {
                <PortfolioInsightsPanel interp=i />
            })}
            {report.as_ref().map(|r| view! {
                <PortfolioMetricsCard summary=r.summary.clone() />
            })}
            {report.as_ref().and_then(|r| r.series.clone()).map(|series| view! {
                <PortfolioCharts series=series />
            })}
            {report.as_ref().map(|r| view! {
                <PortfolioMembersTable members=r.summary.members.clone() />
            })}
            {report.as_ref().map(|r| view! {
                <CorrelationTable matrix=r.correlation.clone() />
            })}
            {report.filter(|r| !r.overlaps.is_empty()).map(|r| view! {
                <OverlapTable pairs=r.overlaps.clone() />
            })}
            <script inner_html=draft_script></script>
        </Layout>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portfolio_page_renders_submit_button() {
        let html = view! {
            <PortfolioPage
                portfolio_name="demo".into()
                holdings_text="000001 0.5\n110011 0.5".into()
                days=90
                period=String::new()
                holdings_top=10
                rolling_window=60
                report=None
                error=None
            />
        }
        .to_html();
        assert!(html.contains("分析组合"));
        assert!(html.contains("/portfolio"));
        assert!(html.contains("textarea"));
        assert!(html.contains("name=\"holdings\""));
        assert!(html.contains("从自选导入等权"));
        assert!(html.contains("rolling_window"));
        assert!(html.contains("fanalyzer.portfolio.draft"));
    }
}
