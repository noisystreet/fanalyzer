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
}

#[derive(Debug, Clone, Default)]
pub struct FundStockHoldingsReport {
    pub as_of: Option<String>,
    pub rows: Vec<FundStockHoldingRow>,
}

static AS_OF_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"截止至：<font[^>]*>([^<]+)</font>").expect("as_of regex"));

/// 股票投资明细表一行：`序号 | 代码 | 名称 | … | 占净值 | 持股数 | 市值`。
static JJCC_ROW_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        r"<tr><td>(\d+)</td><td><a[^>]*>([0-9]{6})</a></td>",
        r"<td class='tol'><a[^>]*>([^<]+)</a></td>",
        r"[\s\S]*?<td class='xglj'>[\s\S]*?</td>",
        r"<td class='tor'>([^<]+)</td><td class='tor'>([^<]+)</td><td class='tor'>([^<]+)</td></tr>"
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
        });
    }

    Ok(FundStockHoldingsReport { as_of, rows })
}

/// 拉取季报披露的股票投资明细（重仓）；`topline` 为接口请求条数上限（官网常用 10～50）。
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
        .get(url)
        .header("Referer", referer)
        .header("Accept", "*/*")
        .send()
        .await?
        .text()
        .await?;

    parse_jjcc_apidata(&text).map_err(EastMoneyError::ParseFailed)
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
