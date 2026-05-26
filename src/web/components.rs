//! Leptos 布局与页面组件（纯 SSR）。

#[path = "portfolio_components.rs"]
mod portfolio_components;

pub use portfolio_components::PortfolioPage;

use crate::models::{FundAnalysis, FundBrief, FundOverview, IndustryAllocation, StockHoldings};
use leptos::prelude::*;

#[component]
pub fn Layout(title: String, children: Children) -> impl IntoView {
    view! {
        <html lang="zh-CN">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <title>{title.clone()}</title>
                <style>{include_str!("style.css")}</style>
            </head>
            <body>
                <header>
                    <a class="brand" href="/">"Fanalyzer"</a>
                    <nav>
                        <a href="/">"首页"</a>
                        <a href="/analyze">"分析"</a>
                        <a href="/compare">"对比"</a>
                        <a href="/portfolio">"组合"</a>
                        <a href="/info">"概况"</a>
                        <a href="/brief">"简报"</a>
                    </nav>
                </header>
                <main>{children()}</main>
                <footer class="site-footer">
                    <p>"数据来自第三方公开渠道，仅供个人研究参考，不构成投资建议。"</p>
                    <p><a href="/disclaimer">"免责声明与数据使用说明"</a></p>
                </footer>
            </body>
        </html>
    }
}

#[component]
pub fn ErrorAlert(message: String) -> impl IntoView {
    view! {
        <div class="alert alert-error">{message}</div>
    }
}

pub(crate) fn pct(v: f64) -> String {
    format!("{:.2}%", v * 100.0)
}

pub(crate) fn num(v: f64) -> String {
    format!("{:.2}", v)
}

#[component]
pub fn AnalysisMetrics(analysis: FundAnalysis) -> impl IntoView {
    let mgr = (!analysis.manager_name.is_empty()).then(|| analysis.manager_name.clone());
    let fee = (analysis.management_fee > 0.0).then(|| {
        format!(
            "{:.2}% / {:.2}%",
            analysis.management_fee, analysis.custody_fee
        )
    });
    view! {
        <section class="card">
            <h2>{analysis.name.clone()} " (" {analysis.code.clone()} ")"</h2>
            <p class="muted">"分析窗口 " {analysis.period_days} " 日历天"</p>
            <div class="table-scroll">
            <table class="metrics">
                <tbody>
                    <tr><th>"总收益率"</th><td>{pct(analysis.total_return)}</td></tr>
                    <tr><th>"年化收益率"</th><td>{pct(analysis.annualized_return)}</td></tr>
                    <tr><th>"波动率"</th><td>{pct(analysis.volatility)}</td></tr>
                    <tr><th>"最大回撤"</th><td>{pct(analysis.max_drawdown)}</td></tr>
                    <tr><th>"夏普比率"</th><td>{num(analysis.sharpe_ratio)}</td></tr>
                    <tr><th>"索提诺比率"</th><td>{num(analysis.sortino_ratio)}</td></tr>
                    <tr><th>"卡玛比率"</th><td>{num(analysis.calmar_ratio)}</td></tr>
                    <tr><th>"Alpha"</th><td>{pct(analysis.alpha)}</td></tr>
                    <tr><th>"Beta"</th><td>{num(analysis.beta)}</td></tr>
                    {mgr.map(|n| view! { <tr><th>"基金经理"</th><td>{n}</td></tr> })}
                    {fee.map(|f| view! { <tr><th>"管理/托管费率"</th><td>{f}</td></tr> })}
                </tbody>
            </table>
            </div>
        </section>
    }
}

#[component]
pub fn CompareTable(analyses: Vec<FundAnalysis>) -> impl IntoView {
    view! {
        <section class="card">
            <h2>"对比结果"</h2>
            <div class="table-scroll">
            <table class="compare">
                <thead>
                    <tr>
                        <th>"代码"</th>
                        <th>"简称"</th>
                        <th>"总收益"</th>
                        <th>"年化"</th>
                        <th>"回撤"</th>
                        <th>"夏普"</th>
                        <th>"Sortino"</th>
                        <th>"Alpha"</th>
                    </tr>
                </thead>
                <tbody>
                    {analyses.into_iter().map(|a| {
                        view! {
                            <tr>
                                <td>{a.code.clone()}</td>
                                <td>{a.name.clone()}</td>
                                <td>{pct(a.total_return)}</td>
                                <td>{pct(a.annualized_return)}</td>
                                <td>{pct(a.max_drawdown)}</td>
                                <td>{num(a.sharpe_ratio)}</td>
                                <td>{num(a.sortino_ratio)}</td>
                                <td>{pct(a.alpha)}</td>
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
pub fn HomePage() -> impl IntoView {
    view! {
        <Layout title="Fanalyzer".into()>
            <section class="card">
                <div class="page-header">
                    <h1>"Fanalyzer"</h1>
                    <p class="muted">"Fund analysis Web — 基于 Leptos SSR，与 CLI 共用同一套分析引擎。"</p>
                </div>
                <div class="feature-grid">
                    <a class="feature-card" href="/analyze">
                        <h3>"单基金分析"</h3>
                        <p class="muted">"输入代码或名称，查看风险收益与 Alpha/Beta。"</p>
                    </a>
                    <a class="feature-card" href="/compare">
                        <h3>"多基金对比"</h3>
                        <p class="muted">"并排比较夏普、回撤、年化等核心指标。"</p>
                    </a>
                    <a class="feature-card" href="/portfolio">
                        <h3>"组合分析"</h3>
                        <p class="muted">"页面编辑自选组合与权重，查看收益、相关性与重仓重叠。"</p>
                    </a>
                    <a class="feature-card" href="/info">
                        <h3>"基金概况"</h3>
                        <p class="muted">"F10 基本信息、经理任期与费率。"</p>
                    </a>
                    <a class="feature-card" href="/brief">
                        <h3>"选基简报"</h3>
                        <p class="muted">"分析 + 行业配置 + 重仓股一页汇总。"</p>
                    </a>
                </div>
            </section>
            <section class="card">
                <h2>"使用说明"</h2>
                <ul>
                    <li>"分析页：输入 6 位代码或基金名称，选择分析窗口。"</li>
                    <li>"对比页：逗号分隔多只基金，至少 2 只。"</li>
                    <li>"组合页：编辑自选组合（每行 code weight），分析加权收益与相关矩阵。"</li>
                    <li>"概况页：F10 基金基本信息、经理与费率。"</li>
                    <li>"简报页：分析 + 行业配置 + 重仓股（季报口径）。"</li>
                    <li>"需联网访问东方财富接口；与 CLI 共用本地缓存。"</li>
                </ul>
            </section>
        </Layout>
    }
}

#[component]
pub fn AnalyzePage(
    code: String,
    days: u32,
    period: String,
    analysis: Option<FundAnalysis>,
    error: Option<String>,
) -> impl IntoView {
    view! {
        <Layout title="基金分析".into()>
            <section class="card">
                <div class="page-header">
                    <h1>"单基金分析"</h1>
                    <p class="muted">"输入基金代码或名称，选择分析窗口后提交。"</p>
                </div>
                <form class="query-form" method="get" action="/analyze">
                    <div class="form-grid">
                        <label class="field field-wide">"基金代码/名称"
                            <input name="code" type="text" placeholder="000001" value=code />
                        </label>
                        <label class="field">"日历天"
                            <input name="days" type="number" min="7" value=days.to_string() />
                        </label>
                        <label class="field">"period（可选）"
                            <input name="period" type="text" placeholder="1y / 3m / ytd" value=period />
                        </label>
                    </div>
                    <div class="form-actions">
                        <button type="submit" class="btn btn-primary">"开始分析"</button>
                    </div>
                </form>
            </section>
            {error.map(|e| view! { <ErrorAlert message=e /> })}
            {analysis.map(|a| view! { <AnalysisMetrics analysis=a /> })}
        </Layout>
    }
}

#[component]
pub fn ComparePage(
    codes: String,
    days: u32,
    period: String,
    sort: String,
    analyses: Vec<FundAnalysis>,
    error: Option<String>,
) -> impl IntoView {
    view! {
        <Layout title="基金对比".into()>
            <section class="card">
                <div class="page-header">
                    <h1>"多基金对比"</h1>
                    <p class="muted">"逗号分隔至少 2 只基金，可按夏普等指标排序。"</p>
                </div>
                <form class="query-form" method="get" action="/compare">
                    <div class="form-grid">
                        <label class="field field-wide">"基金列表（逗号分隔）"
                            <input name="codes" type="text" placeholder="000001,110011" value=codes />
                        </label>
                        <label class="field">"日历天"
                            <input name="days" type="number" min="7" value=days.to_string() />
                        </label>
                        <label class="field">"period（可选）"
                            <input name="period" type="text" value=period />
                        </label>
                        <label class="field">"排序"
                            <select name="sort">
                                <option value="" selected=sort.is_empty()>"代码"</option>
                                <option value="sharpe" selected=sort == "sharpe">"夏普"</option>
                                <option value="sortino" selected=sort == "sortino">"Sortino"</option>
                                <option value="calmar" selected=sort == "calmar">"Calmar"</option>
                                <option value="total-return" selected=sort == "total-return">"总收益"</option>
                            </select>
                        </label>
                    </div>
                    <div class="form-actions">
                        <button type="submit" class="btn btn-primary">"开始对比"</button>
                    </div>
                </form>
            </section>
            {error.map(|e| view! { <ErrorAlert message=e /> })}
            {(!analyses.is_empty()).then(|| view! { <CompareTable analyses=analyses /> })}
        </Layout>
    }
}

#[component]
pub fn OverviewDetail(profile: FundOverview) -> impl IntoView {
    let tenure = profile.manager_tenure_days as f64 / 365.0;
    view! {
        <section class="card">
            <h2>"基金概况"</h2>
            <div class="table-scroll">
            <table class="metrics">
                <tbody>
                    {(!profile.full_name.is_empty()).then(|| view! {
                        <tr><th>"基金全称"</th><td>{profile.full_name.clone()}</td></tr>
                    })}
                    <tr><th>"基金简称"</th><td>{profile.name.clone()}</td></tr>
                    <tr><th>"基金代码"</th><td>{profile.code.clone()}</td></tr>
                    {(!profile.fund_type.is_empty()).then(|| view! {
                        <tr><th>"基金类型"</th><td>{profile.fund_type.clone()}</td></tr>
                    })}
                    {(!profile.establishment_date.is_empty()).then(|| view! {
                        <tr><th>"成立日期"</th><td>{profile.establishment_date.clone()}</td></tr>
                    })}
                    {(!profile.asset_size.is_empty()).then(|| view! {
                        <tr><th>"资产规模"</th><td>{profile.asset_size.clone()}</td></tr>
                    })}
                    {(!profile.company.is_empty()).then(|| view! {
                        <tr><th>"管理公司"</th><td>{profile.company.clone()}</td></tr>
                    })}
                    {(!profile.benchmark.is_empty()).then(|| view! {
                        <tr><th>"业绩比较基准"</th><td>{profile.benchmark.clone()}</td></tr>
                    })}
                    {(!profile.manager_name.is_empty()).then(|| view! {
                        <tr><th>"基金经理"</th><td>{profile.manager_name.clone()}</td></tr>
                    })}
                    {(profile.manager_tenure_days > 0).then(|| view! {
                        <tr><th>"经理任期"</th><td>{format!("{tenure:.1} 年")}</td></tr>
                    })}
                    {(profile.manager_total_return != 0.0).then(|| view! {
                        <tr><th>"任职回报"</th><td>{format!("{:.2}%", profile.manager_total_return * 100.0)}</td></tr>
                    })}
                    {(profile.management_fee > 0.0).then(|| view! {
                        <tr><th>"管理/托管费率"</th><td>{format!("{:.2}% / {:.2}%", profile.management_fee, profile.custody_fee)}</td></tr>
                    })}
                </tbody>
            </table>
            </div>
            {(!profile.investment_target.is_empty()).then(|| view! {
                <section>
                    <h3>"投资目标"</h3>
                    <p>{profile.investment_target.clone()}</p>
                </section>
            })}
            {(!profile.investment_scope.is_empty()).then(|| view! {
                <section>
                    <h3>"投资范围"</h3>
                    <p>{profile.investment_scope.clone()}</p>
                </section>
            })}
        </section>
    }
}

#[component]
pub fn IndustryTable(report: IndustryAllocation) -> impl IntoView {
    view! {
        <section class="card">
            <h3>"行业配置"</h3>
            {report.as_of.clone().map(|d| view! { <p class="muted">"报告截止: " {d}</p> })}
            {report.rows.is_empty().then(|| view! { <p class="muted">"暂无行业配置数据。"</p> })}
            {(!report.rows.is_empty()).then(|| view! {
                <div class="table-scroll">
                <table class="compare">
                    <thead><tr><th>"序号"</th><th>"行业"</th><th>"占净值"</th></tr></thead>
                    <tbody>
                        {report.rows.iter().map(|r| view! {
                            <tr>
                                <td>{r.rank}</td>
                                <td>{r.industry.clone()}</td>
                                <td>{format!("{:.2}%", r.pct_nav)}</td>
                            </tr>
                        }).collect_view()}
                    </tbody>
                </table>
                </div>
            })}
        </section>
    }
}

#[component]
pub fn HoldingsTable(report: StockHoldings) -> impl IntoView {
    view! {
        <section class="card">
            <h3>"重仓股"</h3>
            {report.as_of.clone().map(|d| view! { <p class="muted">"报告截止: " {d}</p> })}
            {report.rows.is_empty().then(|| view! { <p class="muted">"暂无重仓股数据。"</p> })}
            {(!report.rows.is_empty()).then(|| view! {
                <div class="table-scroll">
                <table class="compare">
                    <thead><tr><th>"序号"</th><th>"代码"</th><th>"名称"</th><th>"占净值"</th></tr></thead>
                    <tbody>
                        {report.rows.iter().map(|r| view! {
                            <tr>
                                <td>{r.rank}</td>
                                <td>{r.stock_code.clone()}</td>
                                <td>{r.stock_name.clone()}</td>
                                <td>{format!("{:.2}%", r.pct_nav)}</td>
                            </tr>
                        }).collect_view()}
                    </tbody>
                </table>
                </div>
            })}
        </section>
    }
}

#[component]
pub fn InfoPage(
    code: String,
    profile: Option<FundOverview>,
    error: Option<String>,
) -> impl IntoView {
    view! {
        <Layout title="基金概况".into()>
            <section class="card">
                <div class="page-header">
                    <h1>"基金概况"</h1>
                    <p class="muted">"查询 F10 基本信息、基金经理与费率。"</p>
                </div>
                <form class="query-form" method="get" action="/info">
                    <div class="form-grid">
                        <label class="field field-wide">"基金代码/名称"
                            <input name="code" type="text" placeholder="000001" value=code />
                        </label>
                    </div>
                    <div class="form-actions">
                        <button type="submit" class="btn btn-primary">"查询概况"</button>
                    </div>
                </form>
            </section>
            {error.map(|e| view! { <ErrorAlert message=e /> })}
            {profile.map(|p| view! { <OverviewDetail profile=p /> })}
        </Layout>
    }
}

#[component]
pub fn BriefPage(
    code: String,
    days: u32,
    period: String,
    industry_top: u32,
    holdings_top: u32,
    brief: Option<FundBrief>,
    error: Option<String>,
) -> impl IntoView {
    view! {
        <Layout title="选基简报".into()>
            <section class="card">
                <div class="page-header">
                    <h1>"选基综合简报"</h1>
                    <p class="muted">"一页汇总分析指标、行业配置与重仓股。"</p>
                </div>
                <form class="query-form" method="get" action="/brief">
                    <div class="form-grid">
                        <label class="field field-wide">"基金代码/名称"
                            <input name="code" type="text" placeholder="000001" value=code />
                        </label>
                        <label class="field">"日历天"
                            <input name="days" type="number" min="7" value=days.to_string() />
                        </label>
                        <label class="field">"period（可选）"
                            <input name="period" type="text" placeholder="1y / 3m" value=period />
                        </label>
                        <label class="field">"行业前 N"
                            <input name="industry_top" type="number" min="1" value=industry_top.to_string() />
                        </label>
                        <label class="field">"重仓前 N"
                            <input name="holdings_top" type="number" min="1" max="50" value=holdings_top.to_string() />
                        </label>
                    </div>
                    <div class="form-actions">
                        <button type="submit" class="btn btn-primary">"生成简报"</button>
                    </div>
                </form>
            </section>
            {error.map(|e| view! { <ErrorAlert message=e /> })}
            {brief.map(|b| {
                let mut ind = b.industry.clone();
                ind.rows.truncate(b.industry_top.max(1));
                let mut hold = b.holdings.clone();
                hold.rows.truncate(b.holdings_top.max(1));
                view! {
                    <section class="card">
                        <h2>{b.name.clone()} " (" {b.code.clone()} ")"</h2>
                        <p class="muted">
                            {if !b.fund_type.is_empty() { format!("类型: {}  ", b.fund_type) } else { String::new() }}
                            {if !b.company.is_empty() { format!("管理人: {}  ", b.company) } else { String::new() }}
                            {if !b.asset_size.is_empty() { format!("规模: {}", b.asset_size) } else { String::new() }}
                        </p>
                    </section>
                    {b.analysis.clone().map(|a| view! { <AnalysisMetrics analysis=a /> })}
                    {b.analysis.is_none().then(|| view! {
                        <section class="card"><p class="muted">"（净值分析数据不足，跳过风险收益段）"</p></section>
                    })}
                    <IndustryTable report=ind />
                    <HoldingsTable report=hold />
                }
            })}
        </Layout>
    }
}

#[component]
pub fn DisclaimerPage() -> impl IntoView {
    let body = include_str!("../../docs/DISCLAIMER.md");
    view! {
        <Layout title="免责声明".into()>
            <section class="card">
                <div class="page-header">
                    <h1>"免责声明与数据使用说明"</h1>
                    <p class="muted">"使用本软件前请仔细阅读以下条款。"</p>
                </div>
                <article class="legal-body">{body}</article>
            </section>
        </Layout>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::FundAnalysis;

    fn sample() -> FundAnalysis {
        FundAnalysis {
            code: "000001".into(),
            name: "测试".into(),
            period_days: 90,
            avg_nav: 1.0,
            max_nav: 1.1,
            min_nav: 0.9,
            total_return: 0.05,
            annualized_return: 0.08,
            volatility: 0.12,
            max_drawdown: -0.06,
            sharpe_ratio: 1.2,
            sortino_ratio: 1.3,
            calmar_ratio: 1.1,
            alpha: 0.01,
            beta: 0.95,
            manager_name: String::new(),
            manager_tenure_days: 0,
            manager_total_return: 0.0,
            management_fee: 0.0,
            custody_fee: 0.0,
        }
    }

    #[test]
    fn analysis_metrics_renders_code() {
        let html = view! { <AnalysisMetrics analysis=sample() /> }.to_html();
        assert!(html.contains("000001"));
        assert!(html.contains("总收益率"));
    }

    #[test]
    fn analyze_page_renders_submit_button() {
        let html = view! {
            <AnalyzePage
                code=String::new()
                days=90
                period=String::new()
                analysis=None
                error=None
            />
        }
        .to_html();
        assert!(html.contains("<button"), "missing button: {html}");
        assert!(html.contains("type=\"submit\""), "missing submit: {html}");
        assert!(html.contains("开始分析"), "missing button label: {html}");
        assert!(html.contains("form-grid"), "missing form-grid: {html}");
    }

    #[test]
    fn disclaimer_page_renders_notice() {
        let html = view! { <DisclaimerPage /> }.to_html();
        assert!(html.contains("非投资建议"));
        assert!(html.contains("东方财富"));
        assert!(html.contains("免责声明与数据使用说明"));
    }

    #[test]
    fn overview_detail_renders() {
        let profile = FundOverview {
            code: "000001".into(),
            name: "测试基金".into(),
            ..Default::default()
        };
        let html = view! { <OverviewDetail profile=profile /> }.to_html();
        assert!(html.contains("000001"));
        assert!(html.contains("基金概况"));
    }
}
