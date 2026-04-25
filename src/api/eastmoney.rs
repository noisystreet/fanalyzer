use crate::models::FundNav;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EastMoneyError {
    #[error("HTTP request failed: {0}")]
    HttpFailed(#[from] reqwest::Error),
    #[error("API returned error code: {0}")]
    ApiError(i32),
    #[error("Failed to parse value: {0}")]
    ParseFailed(String),
}

pub struct EastMoneyClient {
    client: reqwest::Client,
}

impl Default for EastMoneyClient {
    fn default() -> Self {
        Self::new()
    }
}

impl EastMoneyClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent(
                    "Mozilla/5.0 (X11; Linux x86_64; rv:109.0) Gecko/20100101 Firefox/115.0",
                )
                .http1_only()
                .build()
                .unwrap_or_default(),
        }
    }

    pub async fn fetch_nav_history(
        &self,
        fund_code: &str,
        page_index: u32,
        page_size: u32,
    ) -> Result<(Vec<FundNav>, u32), EastMoneyError> {
        let referer = format!("https://fundf10.eastmoney.com/jjjz_{}.html", fund_code);
        let url = format!(
            "https://api.fund.eastmoney.com/f10/lsjz?fundCode={}&pageIndex={}&pageSize={}",
            fund_code, page_index, page_size
        );

        let resp = self
            .client
            .get(&url)
            .header("Referer", &referer)
            .header("Accept", "application/json, text/javascript, */*; q=0.01")
            .header("Accept-Language", "zh-CN,zh;q=0.9,en;q=0.8")
            .header("X-Requested-With", "XMLHttpRequest")
            .send()
            .await?
            .text()
            .await?;

        let parsed: serde_json::Value =
            serde_json::from_str(&resp).map_err(|e| EastMoneyError::ParseFailed(e.to_string()))?;

        let err_code = parsed.get("ErrCode").and_then(|v| v.as_i64()).unwrap_or(0);
        if err_code != 0 {
            return Err(EastMoneyError::ApiError(err_code as i32));
        }

        let total_count = parsed
            .get("TotalCount")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as u32;

        let list = parsed
            .get("Data")
            .and_then(|d| d.get("LSJZList"))
            .and_then(|l| l.as_array())
            .cloned()
            .unwrap_or_default();

        let mut navs = Vec::with_capacity(list.len());
        for item in &list {
            let date_str = item.get("FSRQ").and_then(|v| v.as_str()).unwrap_or("");
            let nav_str = item.get("DWJZ").and_then(|v| v.as_str()).unwrap_or("");
            let acc_nav_str = item.get("LJJZ").and_then(|v| v.as_str()).unwrap_or("");
            let daily_return_str = item.get("JZZZL").and_then(|v| v.as_str()).unwrap_or("");

            let date = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                .map_err(|_| EastMoneyError::ParseFailed(date_str.to_string()))?;
            let nav: f64 = nav_str
                .parse()
                .map_err(|_| EastMoneyError::ParseFailed(nav_str.to_string()))?;
            let acc_nav: f64 = acc_nav_str
                .parse()
                .map_err(|_| EastMoneyError::ParseFailed(acc_nav_str.to_string()))?;

            let daily_return = if daily_return_str.is_empty() || daily_return_str == "--" {
                None
            } else {
                daily_return_str.parse::<f64>().ok().map(|v| v / 100.0)
            };

            navs.push(FundNav {
                code: fund_code.to_string(),
                date,
                nav,
                acc_nav,
                daily_return,
            });
        }

        Ok((navs, total_count))
    }

    pub async fn fetch_all_nav_history(
        &self,
        fund_code: &str,
    ) -> Result<Vec<FundNav>, EastMoneyError> {
        let page_size = 40u32;
        let mut all_navs = Vec::new();
        let mut page_index = 1u32;

        loop {
            let (navs, total_count) = self
                .fetch_nav_history(fund_code, page_index, page_size)
                .await?;

            all_navs.extend(navs);

            let fetched = page_index * page_size;
            if fetched >= total_count {
                break;
            }
            page_index += 1;

            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        }

        Ok(all_navs)
    }

    pub async fn fetch_nav_history_by_days(
        &self,
        fund_code: &str,
        days: u32,
    ) -> Result<Vec<FundNav>, EastMoneyError> {
        let need_records = ((days as f64 * 5.0 / 7.0).ceil() as u32).max(1);
        let page_size = need_records.min(100);
        let (mut navs, _) = self.fetch_nav_history(fund_code, 1, page_size).await?;
        navs.truncate(need_records as usize);
        Ok(navs)
    }
}
