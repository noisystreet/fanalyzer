//! 开放式基金排行 HTTP（需与浏览器一致的 Referer）。

use crate::api::eastmoney_error::EastMoneyError;
use crate::api::fund_ranking::{parse_rankhandler_body, FundRankingPage};
use reqwest::Client;
use std::time::Duration as StdDuration;

const FUND_RANKING_REFERER: &str = "https://fund.eastmoney.com/data/fundranking.html";

/// 开放式基金排行单页（`ft`=`gp|hh|zq|zs|qdii|fof`；`sc` 为排序列如 `1n`/`1nzf`、`zzf`，与官网表头一致）。
pub async fn fetch_fund_ranking_page(
    http: &Client,
    fund_type: &str,
    sort_code: &str,
    page_index: u32,
    page_size: u32,
) -> Result<FundRankingPage, EastMoneyError> {
    let url = format!(
        "https://fund.eastmoney.com/data/rankhandler.aspx?\
         op=ph&dt=kf&ft={}&rs=&gs=0&sc={}&st=desc&pi={}&pn={}",
        fund_type, sort_code, page_index, page_size
    );

    let text = http
        .get(url)
        .header("Referer", FUND_RANKING_REFERER)
        .header("Accept", "*/*")
        .send()
        .await?
        .text()
        .await?;

    parse_rankhandler_body(&text).map_err(EastMoneyError::ParseFailed)
}

/// 连续翻页直到凑满 `top` 条（单页最多请求 100 条）。
pub async fn fetch_fund_ranking_top(
    http: &Client,
    fund_type: &str,
    sort_code: &str,
    top: u32,
) -> Result<FundRankingPage, EastMoneyError> {
    let page_size = 100u32.min(top.max(1));
    let mut first = fetch_fund_ranking_page(http, fund_type, sort_code, 1, page_size).await?;
    let total_market = first.total_records;
    let mut rows = std::mem::take(&mut first.rows);
    let mut pi = 2u32;

    while (rows.len() as u32) < top {
        let mut page = fetch_fund_ranking_page(http, fund_type, sort_code, pi, page_size).await?;
        let got = page.rows.len();
        if got == 0 {
            break;
        }
        rows.append(&mut page.rows);
        if got < page_size as usize {
            break;
        }
        pi += 1;
        tokio::time::sleep(StdDuration::from_millis(250)).await;
    }

    rows.truncate(top as usize);
    Ok(FundRankingPage {
        rows,
        total_records: total_market,
    })
}
