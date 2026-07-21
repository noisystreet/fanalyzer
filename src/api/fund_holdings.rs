//! 基金重仓股票：`FundArchivesDatas.aspx?type=jjcc` 内嵌 HTML 解析。

use crate::api::eastmoney_error::EastMoneyError;
use crate::api::f10_apidata::extract_apidata_content;
use regex::Regex;
use reqwest::Client;
use std::sync::LazyLock;

#[derive(Debug, Clone)]
pub struct FundStockHoldingRow {
    pub rank: u32,
    pub stock_code: String,
    pub stock_name: String,
    /// 占净值比例（百分点）
    pub pct_nav: f64,
    /// 持股数（万股）
    pub shares_wan: Option<f64>,
    /// 持仓市值（万元）
    pub market_value_wan: Option<f64>,
    /// 较上期占净值变化（百分点）
    pub pct_nav_chg: Option<f64>,
}

#[derive(Debug, Clone, Default)]
pub struct FundStockHoldingsReport {
    pub as_of: Option<String>,
    pub rows: Vec<FundStockHoldingRow>,
}

static AS_OF_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"截止至：<font[^>]*>([^<]+)</font>").expect("as_of regex"));

/// 股票投资明细表一行：兼容旧版简表与现行 `tzxq` 表；代码支持 4～8 位。
static JJCC_ROW_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        r"<tr><td>(\d+)</td>",
        r"<td(?: class='toc')?><a[^>]*>([0-9A-Za-z]{4,8})</a></td>",
        r"<td class='t(?:ol|oc)'[^>]*><a[^>]*>([^<]+)</a></td>",
        r"[\s\S]*?<td class='xglj'>[\s\S]*?</td>",
        r"<td class='t(?:or|oc)'>([^<]+)</td>",
        r"<td class='t(?:or|oc)'>([^<]+)</td>",
        r"<td class='t(?:or|oc)'>([^<]+)</td></tr>"
    ))
    .expect("jjcc row regex")
});

fn parse_pct_cell(s: &str) -> Result<f64, String> {
    let t = s.trim().trim_end_matches('%').trim();
    t.parse::<f64>()
        .map_err(|_| format!("invalid percent: {s:?}"))
}

fn parse_float_cell(s: &str) -> Option<f64> {
    let t = s.trim();
    if t.is_empty() || t == "--" || t == "-" {
        return None;
    }
    let flat: String = t
        .chars()
        .filter(|c| *c == '.' || c.is_ascii_digit())
        .collect();
    flat.parse().ok()
}

/// 解析 `FundArchivesDatas.aspx` 返回的 jjcc 正文。
pub fn parse_jjcc_apidata(body: &str) -> Result<FundStockHoldingsReport, String> {
    let html = extract_apidata_content(body)?;
    let as_of = AS_OF_RE.captures(html).map(|c| c[1].trim().to_string());

    let mut rows = Vec::new();
    for cap in JJCC_ROW_RE.captures_iter(html) {
        let rank: u32 = cap[1].parse().map_err(|e| format!("rank parse: {e}"))?;
        let stock_code = cap[2].to_string();
        let stock_name = cap[3].trim().to_string();
        let pct_nav = parse_pct_cell(&cap[4])?;
        let shares_wan = parse_float_cell(&cap[5]);
        let market_value_wan = parse_float_cell(&cap[6]);
        rows.push(FundStockHoldingRow {
            rank,
            stock_code,
            stock_name,
            pct_nav,
            shares_wan,
            market_value_wan,
            pct_nav_chg: None,
        });
    }

    Ok(FundStockHoldingsReport { as_of, rows })
}

fn prior_report_year_month(as_of: &str) -> Option<(u32, u32)> {
    use chrono::{Datelike, Months, NaiveDate};
    let d = NaiveDate::parse_from_str(as_of.trim(), "%Y-%m-%d").ok()?;
    let prev = d.checked_sub_months(Months::new(3))?;
    Some((prev.year() as u32, prev.month()))
}

/// 拉取季报披露的股票投资明细（重仓）；`topline` 为接口请求条数上限（官网常用 10～50）。
/// 若能解析截止日，会再拉上一季报告并填充 `pct_nav_chg`。
pub async fn fetch_fund_stock_holdings_jjcc(
    http: &Client,
    fund_code: &str,
    topline: u32,
) -> Result<FundStockHoldingsReport, EastMoneyError> {
    let referer = format!("https://fundf10.eastmoney.com/ccmx_{fund_code}.html");
    let top = topline.clamp(1, 50);
    let url = format!(
        "https://fundf10.eastmoney.com/FundArchivesDatas.aspx?type=jjcc&code={}&topline={}&year=&month=",
        fund_code, top
    );

    let text = http
        .get(&url)
        .header("Referer", &referer)
        .header("Accept", "*/*")
        .send()
        .await?
        .text()
        .await?;

    let mut report = parse_jjcc_apidata(&text).map_err(EastMoneyError::ParseFailed)?;
    fill_holdings_pct_chg(http, fund_code, top, &referer, &mut report).await;
    Ok(report)
}

async fn fill_holdings_pct_chg(
    http: &Client,
    fund_code: &str,
    top: u32,
    referer: &str,
    report: &mut FundStockHoldingsReport,
) {
    let Some(as_of) = report.as_of.as_deref() else {
        return;
    };
    let Some((year, month)) = prior_report_year_month(as_of) else {
        return;
    };
    let url = format!(
        "https://fundf10.eastmoney.com/FundArchivesDatas.aspx?type=jjcc&code={fund_code}&topline={top}&year={year}&month={month}"
    );
    let Ok(text) = http
        .get(url)
        .header("Referer", referer)
        .header("Accept", "*/*")
        .send()
        .await
    else {
        return;
    };
    let Ok(text) = text.text().await else {
        return;
    };
    let Ok(prev) = parse_jjcc_apidata(&text) else {
        return;
    };
    let prev_map: std::collections::HashMap<&str, f64> = prev
        .rows
        .iter()
        .map(|r| (r.stock_code.as_str(), r.pct_nav))
        .collect();
    for row in &mut report.rows {
        if let Some(prev_pct) = prev_map.get(row.stock_code.as_str()) {
            row.pct_nav_chg = Some(row.pct_nav - prev_pct);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn include_fixture(name: &str) -> String {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/api")
            .join(name);
        std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read fixture {}: {e}", path.display()))
    }

    #[test]
    fn parse_sample_jjcc_row() {
        let js = include_fixture("jjcc_sample.js");
        let r = parse_jjcc_apidata(&js).unwrap();
        assert_eq!(r.as_of.as_deref(), Some("2026-03-31"));
        assert_eq!(r.rows.len(), 1);
        assert_eq!(r.rows[0].stock_code, "300308");
        assert_eq!(r.rows[0].stock_name, "中际旭创");
        assert!((r.rows[0].pct_nav - 4.31).abs() < 1e-6);
        assert_eq!(r.rows[0].shares_wan, Some(20.0));
        assert_eq!(r.rows[0].market_value_wan, Some(11388.20));
    }
}
