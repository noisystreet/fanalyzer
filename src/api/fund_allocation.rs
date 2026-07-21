//! 基金 F10「资产配置」页（`zcpz_{code}.html`）解析。

use crate::api::eastmoney_error::EastMoneyError;
use regex::Regex;
use reqwest::Client;
use std::sync::LazyLock;

/// 单期资产配置。
#[derive(Debug, Clone)]
pub struct FundAllocationRow {
    pub as_of: String,
    /// 股票占净值（百分点）
    pub stock_pct: f64,
    /// 债券占净值（百分点）
    pub bond_pct: f64,
    /// 现金占净值（百分点）
    pub cash_pct: f64,
    /// 净资产（亿元）
    pub net_asset_yi: Option<f64>,
}

#[derive(Debug, Clone, Default)]
pub struct FundAllocationReport {
    /// 页面摘要（如规模环比、股票仓位变化说明），可能为空
    pub summary: Option<String>,
    /// 按报告期降序（最新在前）
    pub rows: Vec<FundAllocationRow>,
}

static ROW_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        r"<tr><td>(\d{4}-\d{2}-\d{2})</td>",
        r#"<td class="tor">([^<]+)</td>"#,
        r#"<td class="tor">([^<]+)</td>"#,
        r#"<td class="tor">([^<]+)</td>"#,
        r#"<td class="tor">([^<]+)</td></tr>"#
    ))
    .expect("zcpz row regex")
});

static SUMMARY_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"<label class="left">(截至[^<]+)</label>"#).expect("zcpz summary regex")
});

fn parse_pct(s: &str) -> Result<f64, String> {
    let t = s.trim().trim_end_matches('%').trim();
    t.parse::<f64>()
        .map_err(|_| format!("invalid percent: {s:?}"))
}

fn parse_yi(s: &str) -> Option<f64> {
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

/// 解析 `zcpz_*.html` 资产配置明细表。
pub fn parse_zcpz_html(html: &str) -> Result<FundAllocationReport, String> {
    let summary = SUMMARY_RE.captures(html).map(|c| clean_text(&c[1]));
    let mut rows = Vec::new();
    for cap in ROW_RE.captures_iter(html) {
        rows.push(FundAllocationRow {
            as_of: cap[1].to_string(),
            stock_pct: parse_pct(&cap[2])?,
            bond_pct: parse_pct(&cap[3])?,
            cash_pct: parse_pct(&cap[4])?,
            net_asset_yi: parse_yi(&cap[5]),
        });
    }
    if rows.is_empty() {
        return Err("no asset allocation rows found".into());
    }
    Ok(FundAllocationReport { summary, rows })
}

fn clean_text(s: &str) -> String {
    let mut out = s.to_string();
    while let Some(start) = out.find('<') {
        if let Some(end) = out[start..].find('>') {
            out.replace_range(start..start + end + 1, "");
        } else {
            break;
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// 拉取基金资产配置历史（股/债/现占净值与净资产）。
pub async fn fetch_fund_allocation_zcpz(
    http: &Client,
    fund_code: &str,
) -> Result<FundAllocationReport, EastMoneyError> {
    let url = format!("https://fundf10.eastmoney.com/zcpz_{fund_code}.html");
    let text = http
        .get(&url)
        .header("Referer", "https://fund.eastmoney.com/")
        .header("Accept", "text/html")
        .send()
        .await?
        .text()
        .await?;
    parse_zcpz_html(&text).map_err(EastMoneyError::ParseFailed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_sample_zcpz_rows() {
        let html = r#"
<label class="left">截至2026-06-30，华夏成长混合 净资产规模39.38亿元，比上一期增加了48.93%</label>
<table class="w782 comm tzxq"><thead></thead><tbody>
<tr><td>2026-06-30</td><td class="tor">79.98%</td><td class="tor">20.19%</td><td class="tor">0.79%</td><td class="tor">39.38</td></tr>
<tr><td>2026-03-31</td><td class="tor">72.51%</td><td class="tor">20.63%</td><td class="tor">2.39%</td><td class="tor">26.44</td></tr>
</tbody></table>
"#;
        let r = parse_zcpz_html(html).unwrap();
        assert!(r.summary.as_ref().unwrap().contains("39.38"));
        assert_eq!(r.rows.len(), 2);
        assert_eq!(r.rows[0].as_of, "2026-06-30");
        assert!((r.rows[0].stock_pct - 79.98).abs() < 1e-6);
        assert_eq!(r.rows[0].net_asset_yi, Some(39.38));
    }
}
