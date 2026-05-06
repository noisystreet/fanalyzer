//! 基金 F10「行业配置」页面（`type=hypz`）解析：数据源为内嵌 HTML。

use crate::api::eastmoney_error::EastMoneyError;
use regex::Regex;
use reqwest::Client;
use std::sync::LazyLock;

/// 一行行业配置（证监会行业分类口径，与官网表一致）。
#[derive(Debug, Clone)]
pub struct FundIndustryRow {
    pub rank: u32,
    pub industry: String,
    /// 占净值比例（百分点，如 61.14 表示 61.14%）
    pub pct_nav: f64,
    /// 市值（万元），接口可能为 `-` 或空
    pub market_value_wan: Option<f64>,
}

#[derive(Debug, Clone, Default)]
pub struct FundIndustryReport {
    /// 报告截止日（页面「截止至」）
    pub as_of: Option<String>,
    pub rows: Vec<FundIndustryRow>,
}

static ROW_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        r"<tr><td>(\d+)</td><td class='tol'>([^<]+)</td>",
        r"<td><a[^>]*>[^<]*</a></td>",
        r"<td class='tor'>([^<]+)</td><td class='tor'>([^<]+)</td>"
    ))
    .expect("fund industry row regex")
});

static AS_OF_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"截止至：<font[^>]*>([^<]+)</font>").expect("as_of regex"));

fn extract_apidata_html(js: &str) -> Result<&str, String> {
    let key = "content:\"";
    let start = js
        .find(key)
        .ok_or_else(|| "F10 hypz: missing content:\"".to_string())?
        + key.len();
    let tail = &js[start..];
    let end = tail
        .find("\",arryear:")
        .ok_or_else(|| "F10 hypz: missing \",arryear:\"".to_string())?;
    Ok(&tail[..end])
}

fn parse_pct_cell(s: &str) -> Result<f64, String> {
    let t = s.trim().trim_end_matches('%').trim();
    t.parse::<f64>()
        .map_err(|_| format!("invalid percent: {s:?}"))
}

fn parse_wan_cell(s: &str) -> Option<f64> {
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

/// 从 F10 返回的 `var apidata={ content:"..."};` 中解析行业表。
pub fn parse_hypz_apidata(body: &str) -> Result<FundIndustryReport, String> {
    let html = extract_apidata_html(body.trim())?;
    let as_of = AS_OF_RE.captures(html).map(|c| c[1].trim().to_string());

    let mut rows = Vec::new();
    for cap in ROW_RE.captures_iter(html) {
        let rank: u32 = cap[1].parse().map_err(|e| format!("rank parse: {e}"))?;
        let industry = cap[2].trim().to_string();
        let pct_nav = parse_pct_cell(&cap[3])?;
        let market_value_wan = parse_wan_cell(&cap[4]);
        rows.push(FundIndustryRow {
            rank,
            industry,
            pct_nav,
            market_value_wan,
        });
    }

    Ok(FundIndustryReport { as_of, rows })
}

/// 拉取基金最新披露的行业配置（股票仓位相关行业；债券型基金可能为空表）。
pub async fn fetch_fund_industry_hypz(
    http: &Client,
    fund_code: &str,
) -> Result<FundIndustryReport, EastMoneyError> {
    let referer = format!("https://fundf10.eastmoney.com/{fund_code}.html");
    let url = format!(
        "https://fundf10.eastmoney.com/F10DataApi.aspx?type=hypz&code={}&topline=500&company=&sector=&reportName=",
        fund_code
    );

    let text = http
        .get(url)
        .header("Referer", referer)
        .header("Accept", "*/*")
        .send()
        .await?
        .text()
        .await?;

    parse_hypz_apidata(&text).map_err(EastMoneyError::ParseFailed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_sample_hypz() {
        let js = r#"var apidata={ content:"<div class='box'><h4><label class='right'>截止至：<font class='px12'>2026-03-31</font></label></h4><table class='hypz'><tbody><tr><td>1</td><td class='tol'>制造业</td><td><a href='#'>变动详情</a></td><td class='tor'>61.14%</td><td class='tor'>161,661.79</td><td></td></tr><tr><td>2</td><td class='tol'>信息传输</td><td><a href='#'>变动详情</a></td><td class='tor'>7.18%</td><td class='tor'>18,987.58</td><td></td></tr></tbody></table></div>",arryear:[2026],curyear:2026};"#;
        let r = parse_hypz_apidata(js).unwrap();
        assert_eq!(r.as_of.as_deref(), Some("2026-03-31"));
        assert_eq!(r.rows.len(), 2);
        assert_eq!(r.rows[0].industry, "制造业");
        assert!((r.rows[0].pct_nav - 61.14).abs() < 1e-6);
        assert_eq!(r.rows[0].market_value_wan, Some(161661.79));
    }

    #[test]
    fn missing_content_err() {
        assert!(parse_hypz_apidata("var x=1;").is_err());
    }
}
