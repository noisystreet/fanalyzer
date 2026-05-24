//! Leptos 布局与页面组件（纯 SSR）。

use crate::models::FundAnalysis;
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
                    <a class="brand" href="/">"analysis_fund"</a>
                    <nav>
                        <a href="/">"首页"</a>
                        <a href="/analyze">"分析"</a>
                        <a href="/compare">"对比"</a>
                    </nav>
                </header>
                <main>{children()}</main>
                <footer>"数据仅供研究参考，不构成投资建议。"</footer>
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

fn pct(v: f64) -> String {
    format!("{:.2}%", v * 100.0)
}

fn num(v: f64) -> String {
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
        </section>
    }
}

#[component]
pub fn CompareTable(analyses: Vec<FundAnalysis>) -> impl IntoView {
    view! {
        <section class="card">
            <h2>"对比结果"</h2>
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
        </section>
    }
}

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <Layout title="analysis_fund".into()>
            <section class="card">
                <h1>"基金分析 Web"</h1>
                <p class="muted">"基于 Leptos SSR 的简易界面，复用 CLI 同一套分析引擎。"</p>
                <p>
                    <a class="btn" href="/analyze">"单基金分析"</a>
                    " "
                    <a class="btn" href="/compare">"多基金对比"</a>
                </p>
            </section>
            <section class="card">
                <h2>"使用说明"</h2>
                <ul>
                    <li>"分析页：输入 6 位代码或基金名称，选择分析窗口。"</li>
                    <li>"对比页：逗号分隔多只基金，至少 2 只。"</li>
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
                <h1>"单基金分析"</h1>
                <form method="get" action="/analyze">
                    <div class="row">
                        <label>"基金代码/名称"
                            <input name="code" type="text" placeholder="000001" value=code />
                        </label>
                        <label>"日历天"
                            <input name="days" type="number" min="7" value=days.to_string() />
                        </label>
                        <label>"period（可选）"
                            <input name="period" type="text" placeholder="1y / 3m / ytd" value=period />
                        </label>
                        <button type="submit">"分析"</button>
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
                <h1>"多基金对比"</h1>
                <form method="get" action="/compare">
                    <div class="row">
                        <label>"基金列表（逗号分隔）"
                            <input name="codes" type="text" placeholder="000001,110011" value=codes />
                        </label>
                        <label>"日历天"
                            <input name="days" type="number" min="7" value=days.to_string() />
                        </label>
                        <label>"period（可选）"
                            <input name="period" type="text" value=period />
                        </label>
                        <label>"排序"
                            <select name="sort">
                                <option value="" selected=sort.is_empty()>"代码"</option>
                                <option value="sharpe" selected=sort == "sharpe">"夏普"</option>
                                <option value="sortino" selected=sort == "sortino">"Sortino"</option>
                                <option value="calmar" selected=sort == "calmar">"Calmar"</option>
                                <option value="total-return" selected=sort == "total-return">"总收益"</option>
                            </select>
                        </label>
                        <button type="submit">"对比"</button>
                    </div>
                </form>
            </section>
            {error.map(|e| view! { <ErrorAlert message=e /> })}
            {(!analyses.is_empty()).then(|| view! { <CompareTable analyses=analyses /> })}
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
}
