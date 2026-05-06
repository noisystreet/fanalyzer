pub use crate::api::eastmoney_error::EastMoneyError;
use crate::api::fund_ranking::FundRankingPage;
use crate::api::nav_merge::merge_navs_by_date;
use crate::models::FundNav;
use chrono::{Duration, FixedOffset};
use reqwest::Client;
use std::time::Duration as StdDuration;

/// 构建 `EastMoneyClient`（超时、UA、代理）；由 CLI 从 `AppConfig.api` 映射而来。
#[derive(Debug, Clone)]
pub struct EastMoneyClientOptions {
    pub timeout_secs: u64,
    pub user_agent: Option<String>,
    pub proxy: Option<String>,
}

impl Default for EastMoneyClientOptions {
    fn default() -> Self {
        Self {
            timeout_secs: 30,
            user_agent: None,
            proxy: None,
        }
    }
}

const DEFAULT_USER_AGENT: &str =
    "Mozilla/5.0 (X11; Linux x86_64; rv:109.0) Gecko/20100101 Firefox/115.0";

pub struct EastMoneyClient {
    client: reqwest::Client,
}

impl Default for EastMoneyClient {
    fn default() -> Self {
        Self::with_options(EastMoneyClientOptions::default())
            .expect("default EastMoneyClientOptions builds a valid HTTP client")
    }
}

impl EastMoneyClient {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_options(opts: EastMoneyClientOptions) -> Result<Self, EastMoneyError> {
        let timeout = StdDuration::from_secs(opts.timeout_secs.max(1));
        let ua = opts
            .user_agent
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or(DEFAULT_USER_AGENT);

        let mut builder = Client::builder()
            .user_agent(ua)
            .http1_only()
            .timeout(timeout);

        if let Some(ref p) = opts.proxy {
            if !p.is_empty() {
                let proxy = reqwest::Proxy::all(p).map_err(|e| {
                    EastMoneyError::ClientBuildFailed(format!("invalid proxy URL: {e}"))
                })?;
                builder = builder.proxy(proxy);
            }
        }

        let client = builder.build().map_err(EastMoneyError::HttpFailed)?;
        Ok(Self { client })
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
            .get(url)
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
        if days == 0 {
            return Ok(Vec::new());
        }

        let today = chrono::Local::now().date_naive();
        let cutoff = today
            .checked_sub_signed(Duration::days(days as i64))
            .unwrap_or(today);

        const PAGE_SIZE: u32 = 100;
        let mut page = 1u32;
        let mut collected: Vec<FundNav> = Vec::new();

        loop {
            let (batch, total) = self.fetch_nav_history(fund_code, page, PAGE_SIZE).await?;
            if batch.is_empty() {
                break;
            }

            let batch_min = batch.iter().map(|n| n.date).min().expect("batch non-empty");
            collected.extend(batch);

            let fetched_all = total == 0 || page.saturating_mul(PAGE_SIZE) >= total;
            if batch_min <= cutoff || fetched_all {
                break;
            }

            page += 1;
            tokio::time::sleep(StdDuration::from_millis(250)).await;
        }

        let mut merged = merge_navs_by_date(collected);
        merged.retain(|n| n.date >= cutoff);
        merged.sort_by_key(|n| n.date);
        Ok(merged)
    }

    /// 开放式基金排行单页（`ft`=`gp|hh|zq|zs|qdii|fof`，`sc` 如 `1n` 近一年、`zzf` 默认口径）。
    pub async fn fetch_fund_ranking_page(
        &self,
        fund_type: &str,
        sort_code: &str,
        page_index: u32,
        page_size: u32,
    ) -> Result<FundRankingPage, EastMoneyError> {
        crate::api::eastmoney_ranking::fetch_fund_ranking_page(
            &self.client,
            fund_type,
            sort_code,
            page_index,
            page_size,
        )
        .await
    }

    /// 连续翻页直到凑满 `top` 条（单页最多请求 100 条）。
    pub async fn fetch_fund_ranking_top(
        &self,
        fund_type: &str,
        sort_code: &str,
        top: u32,
    ) -> Result<FundRankingPage, EastMoneyError> {
        crate::api::eastmoney_ranking::fetch_fund_ranking_top(
            &self.client,
            fund_type,
            sort_code,
            top,
        )
        .await
    }

    pub async fn fetch_fund_name(&self, fund_code: &str) -> Result<String, EastMoneyError> {
        let url = format!(
            "https://fundgz.1234567.com.cn/js/{}.js?rt={}",
            fund_code,
            chrono::Local::now().timestamp_millis()
        );

        let resp = self
            .client
            .get(&url)
            .header("Referer", "https://fund.eastmoney.com/")
            .send()
            .await?
            .text()
            .await?;

        let json_str = resp
            .trim_start_matches("jsonpgz(")
            .trim_end_matches(");")
            .trim();

        let parsed: serde_json::Value = serde_json::from_str(json_str)
            .map_err(|e| EastMoneyError::ParseFailed(e.to_string()))?;

        let name = parsed
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(fund_code)
            .to_string();

        Ok(name)
    }

    pub async fn search_fund(
        &self,
        keyword: &str,
    ) -> Result<Vec<(String, String)>, EastMoneyError> {
        let url = "https://fund.eastmoney.com/js/fundcode_search.js";

        let resp = self
            .client
            .get(url)
            .header("Referer", "https://fund.eastmoney.com/")
            .send()
            .await?
            .text()
            .await?;

        let json_str = resp
            .trim_start_matches("var r = ")
            .trim_end_matches(";")
            .trim();

        let parsed: Vec<Vec<String>> = serde_json::from_str(json_str)
            .map_err(|e| EastMoneyError::ParseFailed(e.to_string()))?;

        let keyword_lower = keyword.to_lowercase();
        let results: Vec<(String, String)> = parsed
            .into_iter()
            .filter(|item| {
                if item.len() >= 2 {
                    let code = &item[0];
                    let name = &item[2];
                    code.to_lowercase().contains(&keyword_lower)
                        || name.to_lowercase().contains(&keyword_lower)
                } else {
                    false
                }
            })
            .map(|item| (item[0].clone(), item[2].clone()))
            .take(10)
            .collect();

        Ok(results)
    }

    pub async fn fetch_index_history(
        &self,
        index_code: &str,
        page_index: u32,
        page_size: u32,
    ) -> Result<(Vec<IndexData>, u32), EastMoneyError> {
        let url = format!(
            "https://push2his.eastmoney.com/api/qt/stock/kline/get?secid={}&fields1=f1,f2,f3,f4,f5,f6,f7,f8,f9,f10,f11,f12,f13&fields2=f51,f52,f53,f54,f55,f56,f57,f58,f59,f60,f61&klt=101&fqt=0&beg=0&end=20500101&smplmt=460&lmt=1000000&_=1704067200000",
            index_code
        );

        let resp = self
            .client
            .get(&url)
            .header("Referer", "https://quote.eastmoney.com/")
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        let data = resp
            .get("data")
            .ok_or_else(|| EastMoneyError::ParseFailed("Missing data field".to_string()))?;

        let klines = data
            .get("klines")
            .and_then(|v| v.as_array())
            .ok_or_else(|| EastMoneyError::ParseFailed("Missing klines field".to_string()))?;

        let total = data
            .get("total")
            .and_then(|v| v.as_u64())
            .unwrap_or(klines.len() as u64) as u32;

        let start_idx = ((page_index - 1) * page_size) as usize;
        let end_idx = (start_idx + page_size as usize).min(klines.len());

        let mut index_data = Vec::new();
        for i in start_idx..end_idx {
            if let Some(line) = klines.get(i).and_then(|v| v.as_str()) {
                let parts: Vec<&str> = line.split(',').collect();
                if parts.len() >= 5 {
                    if let (Ok(date_str), Ok(close)) =
                        (parts[0].parse::<String>(), parts[2].parse::<f64>())
                    {
                        if let Ok(date) = chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
                            index_data.push(IndexData {
                                date: chrono::DateTime::from_naive_utc_and_offset(
                                    date.and_hms_opt(0, 0, 0).unwrap(),
                                    FixedOffset::east_opt(0).unwrap(),
                                ),
                                close,
                            });
                        }
                    }
                }
            }
        }

        index_data.reverse();
        Ok((index_data, total))
    }

    pub async fn fetch_fund_manager(
        &self,
        fund_code: &str,
    ) -> Result<FundManagerInfo, EastMoneyError> {
        let url = format!("https://fund.eastmoney.com/pingzhongdata/{}.js", fund_code);

        let resp = self
            .client
            .get(&url)
            .header("Referer", "https://fund.eastmoney.com/")
            .send()
            .await?
            .text()
            .await?;

        // 从 JS 中提取 Data_currentFundManager 数组
        let manager_json = Self::extract_js_variable(&resp, "Data_currentFundManager")
            .ok_or_else(|| EastMoneyError::ParseFailed("Manager data not found".to_string()))?;

        let managers: Vec<serde_json::Value> =
            serde_json::from_str(&manager_json).map_err(|e| {
                EastMoneyError::ParseFailed(format!("Failed to parse manager data: {}", e))
            })?;

        let manager = managers
            .first()
            .ok_or_else(|| EastMoneyError::ParseFailed("No manager data".to_string()))?;

        let name = manager
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("未知")
            .to_string();

        // 解析 workTime 字段，格式如 "14年又138天"
        let work_time = manager
            .get("workTime")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let tenure_days = Self::parse_work_time(work_time);

        // 从 profit 中提取任期收益
        let total_return = manager
            .get("profit")
            .and_then(|p| p.get("series"))
            .and_then(|s| s.as_array())
            .and_then(|arr| arr.first())
            .and_then(|item| item.get("data"))
            .and_then(|d| d.as_array())
            .and_then(|arr| arr.first())
            .and_then(|item| item.get("y"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0)
            / 100.0; // 转换为小数

        Ok(FundManagerInfo {
            name,
            start_date: String::new(),
            tenure_days,
            total_return,
        })
    }

    fn extract_js_variable(js_content: &str, var_name: &str) -> Option<String> {
        let pattern = format!("var {} =", var_name);
        let start = js_content.find(&pattern)?;
        let start = start + pattern.len();
        let remaining = &js_content[start..];

        // 找到变量值的结束位置（分号或换行）
        let end = remaining.find(";").unwrap_or(remaining.len());
        let value = &remaining[..end].trim();

        Some(value.to_string())
    }

    fn parse_work_time(work_time: &str) -> i32 {
        // 解析 "14年又138天" 格式的字符串
        let mut days = 0i32;

        // 提取年数
        if let Some(year_idx) = work_time.find("年") {
            if let Ok(years) = work_time[..year_idx].trim().parse::<i32>() {
                days += years * 365;
            }
        }

        // 提取天数
        if let Some(day_start) = work_time.find("又") {
            if let Some(day_end) = work_time.find("天") {
                let day_str = &work_time[day_start + 3..day_end]; // "又" 是3字节UTF-8
                if let Ok(d) = day_str.trim().parse::<i32>() {
                    days += d;
                }
            }
        }

        days
    }

    pub async fn fetch_fund_fee(&self, fund_code: &str) -> Result<FundFeeInfo, EastMoneyError> {
        // 复用同一个 JS 数据源
        let url = format!("https://fund.eastmoney.com/pingzhongdata/{}.js", fund_code);

        let resp = self
            .client
            .get(&url)
            .header("Referer", "https://fund.eastmoney.com/")
            .send()
            .await?
            .text()
            .await?;

        // 从 JS 变量中提取费率信息
        // fund_sourceRate 是原费率，fund_Rate 是现费率
        let source_rate = Self::extract_js_string_value(&resp, "fund_sourceRate")
            .unwrap_or_else(|| "0".to_string());
        let current_rate =
            Self::extract_js_string_value(&resp, "fund_Rate").unwrap_or_else(|| "0".to_string());

        let management_fee = source_rate.parse::<f64>().unwrap_or(0.0);
        let purchase_fee = current_rate.parse::<f64>().unwrap_or(0.0);

        // 托管费率通常在 0.1%-0.25% 之间，JS 中没有直接提供，使用默认值
        // 可以通过其他 API 获取，这里先设为 0
        let custody_fee = 0.0;

        Ok(FundFeeInfo {
            management_fee,
            custody_fee,
            purchase_fee,
            redemption_fee: 0.0, // 赎回费通常是阶梯式的，这里简化处理
        })
    }

    pub async fn fetch_fund_profile(&self, fund_code: &str) -> Result<FundProfile, EastMoneyError> {
        // 从 pingzhongdata JS 数据源获取基本信息
        let js_url = format!("https://fund.eastmoney.com/pingzhongdata/{}.js", fund_code);

        let js_resp = self
            .client
            .get(&js_url)
            .header("Referer", "https://fund.eastmoney.com/")
            .send()
            .await?
            .text()
            .await?;

        // 提取基金名称和代码
        let name = Self::extract_js_string_value(&js_resp, "fS_name")
            .unwrap_or_else(|| "未知".to_string());

        // 提取基金经理信息
        let manager_json = Self::extract_js_variable(&js_resp, "Data_currentFundManager")
            .ok_or_else(|| EastMoneyError::ParseFailed("Manager data not found".to_string()))?;

        let managers: Vec<serde_json::Value> =
            serde_json::from_str(&manager_json).map_err(|e| {
                EastMoneyError::ParseFailed(format!("Failed to parse manager data: {}", e))
            })?;

        let manager = managers
            .first()
            .ok_or_else(|| EastMoneyError::ParseFailed("No manager data".to_string()))?;

        let manager_name = manager
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("未知")
            .to_string();

        let work_time = manager
            .get("workTime")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let manager_tenure_days = Self::parse_work_time(work_time);

        let manager_total_return = manager
            .get("profit")
            .and_then(|p| p.get("series"))
            .and_then(|s| s.as_array())
            .and_then(|arr| arr.first())
            .and_then(|item| item.get("data"))
            .and_then(|d| d.as_array())
            .and_then(|arr| arr.first())
            .and_then(|item| item.get("y"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0)
            / 100.0;

        // 提取费率信息
        let source_rate = Self::extract_js_string_value(&js_resp, "fund_sourceRate")
            .unwrap_or_else(|| "0".to_string());
        let management_fee = source_rate.parse::<f64>().unwrap_or(0.0);

        // 从 fundf10.eastmoney.com 获取详细基金概况
        let detail_url = format!("https://fundf10.eastmoney.com/jbgk_{}.html", fund_code);

        let detail_resp = self
            .client
            .get(&detail_url)
            .header("Referer", "https://fund.eastmoney.com/")
            .send()
            .await?
            .text()
            .await?;

        // 解析详细基金信息
        let detail_info = Self::parse_fund_detail(&detail_resp);

        Ok(FundProfile {
            code: fund_code.to_string(),
            name: name.clone(),
            full_name: detail_info.full_name.unwrap_or(name),
            fund_type: detail_info.fund_type,
            establishment_date: detail_info.establishment_date,
            asset_size: detail_info.asset_size,
            company: detail_info.company,
            manager_name,
            manager_tenure_days,
            manager_total_return,
            management_fee,
            custody_fee: 0.0,
            investment_target: detail_info.investment_target,
            investment_scope: detail_info.investment_scope,
            investment_strategy: detail_info.investment_strategy,
            benchmark: detail_info.benchmark,
        })
    }

    /// 关键字后紧随的首个 `<td>...</td>` 纯文本。
    fn extract_td_after_keyword(html: &str, anchor: &str) -> Option<String> {
        let idx = html.find(anchor)?;
        let tail = &html[idx..];
        let open = tail.find("<td>")?;
        let inner = &tail[open + 4..];
        let close = inner.find("</td>")?;
        Some(Self::clean_html(&inner[..close]))
    }

    /// 关键字后紧随的首个 `<td class="tditem">...</td>` 纯文本。
    fn extract_tditem_after_keyword(html: &str, anchor: &str) -> Option<String> {
        const TDITEM: &str = "<td class=\"tditem\">";
        let idx = html.find(anchor)?;
        let tail = &html[idx..];
        let open = tail.find(TDITEM)?;
        let inner = &tail[open + TDITEM.len()..];
        let close = inner.find("</td>")?;
        Some(Self::clean_html(&inner[..close]))
    }

    fn parse_fund_detail(html: &str) -> FundDetailInfo {
        FundDetailInfo {
            full_name: Self::extract_td_after_keyword(html, "基金全称"),
            fund_type: Self::extract_td_after_keyword(html, "基金类型").unwrap_or_default(),
            establishment_date: Self::extract_td_after_keyword(html, "成立日期")
                .unwrap_or_default(),
            asset_size: Self::extract_td_after_keyword(html, "资产规模").unwrap_or_default(),
            company: Self::extract_td_after_keyword(html, "基金管理人").unwrap_or_default(),
            investment_target: Self::extract_tditem_after_keyword(html, "投资目标")
                .unwrap_or_default(),
            investment_scope: Self::extract_tditem_after_keyword(html, "投资范围")
                .unwrap_or_default(),
            investment_strategy: Self::extract_tditem_after_keyword(html, "投资策略")
                .unwrap_or_default(),
            benchmark: Self::extract_td_after_keyword(html, "业绩比较基准").unwrap_or_default(),
        }
    }

    fn clean_html(html: &str) -> String {
        let mut result = html.to_string();
        // 移除HTML标签
        while let Some(start) = result.find('<') {
            if let Some(end) = result[start..].find('>') {
                result.replace_range(start..start + end + 1, "");
            } else {
                break;
            }
        }
        // 解码HTML实体
        result = result
            .replace("&nbsp;", " ")
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"");
        // 移除多余空白
        result.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    fn extract_js_string_value(js_content: &str, var_name: &str) -> Option<String> {
        // 去除可能的 UTF-8 BOM
        let content = js_content.strip_prefix('\u{feff}').unwrap_or(js_content);

        let pattern = format!("var {}=\"", var_name);
        let start = content.find(&pattern)?;
        let start = start + pattern.len();
        let remaining = &content[start..];

        // 找到引号结束位置
        let end = remaining.find("\"")?;
        Some(remaining[..end].to_string())
    }
}

#[derive(Debug, Clone)]
pub struct IndexData {
    pub date: chrono::DateTime<FixedOffset>,
    pub close: f64,
}

#[derive(Debug, Clone)]
pub struct FundManagerInfo {
    pub name: String,
    pub start_date: String,
    pub tenure_days: i32,
    pub total_return: f64,
}

#[derive(Debug, Clone)]
pub struct FundFeeInfo {
    pub management_fee: f64,
    pub custody_fee: f64,
    pub purchase_fee: f64,
    pub redemption_fee: f64,
}

#[derive(Debug, Clone, Default)]
struct FundDetailInfo {
    pub full_name: Option<String>,
    pub fund_type: String,
    pub establishment_date: String,
    pub asset_size: String,
    pub company: String,
    pub investment_target: String,
    pub investment_scope: String,
    pub investment_strategy: String,
    pub benchmark: String,
}

#[derive(Debug, Clone)]
pub struct FundProfile {
    pub code: String,
    pub name: String,
    pub full_name: String,
    pub fund_type: String,
    pub establishment_date: String,
    pub asset_size: String,
    pub company: String,
    pub manager_name: String,
    pub manager_tenure_days: i32,
    pub manager_total_return: f64,
    pub management_fee: f64,
    pub custody_fee: f64,
    pub investment_target: String,
    pub investment_scope: String,
    pub investment_strategy: String,
    pub benchmark: String,
}
