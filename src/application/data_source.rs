//! 基金远程数据源抽象（`Session` 门面；便于测试 mock）。

use crate::api::eastmoney::{
    EastMoneyClient, EastMoneyError, FundFeeInfo, FundManagerInfo, FundProfile, IndexData,
};
use crate::api::fund_holdings::FundStockHoldingsReport;
use crate::api::fund_industry::FundIndustryReport;
use crate::api::fund_ranking::FundRankingPage;
use crate::models::FundNav;
use async_trait::async_trait;

/// 应用层使用的基金数据访问接口（默认由 `EastMoneyClient` 实现）。
#[async_trait]
pub trait FundDataSource: Send + Sync {
    async fn fetch_nav_history(
        &self,
        fund_code: &str,
        page_index: u32,
        page_size: u32,
    ) -> Result<(Vec<FundNav>, u32), EastMoneyError>;

    async fn fetch_nav_history_by_days(
        &self,
        fund_code: &str,
        days: u32,
    ) -> Result<Vec<FundNav>, EastMoneyError>;

    async fn fetch_fund_ranking_top(
        &self,
        fund_type: &str,
        sort_code: &str,
        top: u32,
    ) -> Result<FundRankingPage, EastMoneyError>;

    async fn fetch_fund_industry_allocation(
        &self,
        fund_code: &str,
    ) -> Result<FundIndustryReport, EastMoneyError>;

    async fn fetch_fund_stock_holdings(
        &self,
        fund_code: &str,
        topline: u32,
    ) -> Result<FundStockHoldingsReport, EastMoneyError>;

    async fn fetch_fund_name(&self, fund_code: &str) -> Result<String, EastMoneyError>;

    async fn search_fund(&self, query: &str) -> Result<Vec<(String, String)>, EastMoneyError>;

    async fn fetch_index_history(
        &self,
        index_code: &str,
        page_index: u32,
        page_size: u32,
    ) -> Result<(Vec<IndexData>, u32), EastMoneyError>;

    async fn fetch_fund_manager(&self, fund_code: &str) -> Result<FundManagerInfo, EastMoneyError>;

    async fn fetch_fund_fee(&self, fund_code: &str) -> Result<FundFeeInfo, EastMoneyError>;

    async fn fetch_fund_profile(&self, fund_code: &str) -> Result<FundProfile, EastMoneyError>;
}

#[async_trait]
impl FundDataSource for EastMoneyClient {
    async fn fetch_nav_history(
        &self,
        fund_code: &str,
        page_index: u32,
        page_size: u32,
    ) -> Result<(Vec<FundNav>, u32), EastMoneyError> {
        self.fetch_nav_history(fund_code, page_index, page_size)
            .await
    }

    async fn fetch_nav_history_by_days(
        &self,
        fund_code: &str,
        days: u32,
    ) -> Result<Vec<FundNav>, EastMoneyError> {
        self.fetch_nav_history_by_days(fund_code, days).await
    }

    async fn fetch_fund_ranking_top(
        &self,
        fund_type: &str,
        sort_code: &str,
        top: u32,
    ) -> Result<FundRankingPage, EastMoneyError> {
        self.fetch_fund_ranking_top(fund_type, sort_code, top).await
    }

    async fn fetch_fund_industry_allocation(
        &self,
        fund_code: &str,
    ) -> Result<FundIndustryReport, EastMoneyError> {
        self.fetch_fund_industry_allocation(fund_code).await
    }

    async fn fetch_fund_stock_holdings(
        &self,
        fund_code: &str,
        topline: u32,
    ) -> Result<FundStockHoldingsReport, EastMoneyError> {
        self.fetch_fund_stock_holdings(fund_code, topline).await
    }

    async fn fetch_fund_name(&self, fund_code: &str) -> Result<String, EastMoneyError> {
        self.fetch_fund_name(fund_code).await
    }

    async fn search_fund(&self, query: &str) -> Result<Vec<(String, String)>, EastMoneyError> {
        self.search_fund(query).await
    }

    async fn fetch_index_history(
        &self,
        index_code: &str,
        page_index: u32,
        page_size: u32,
    ) -> Result<(Vec<IndexData>, u32), EastMoneyError> {
        self.fetch_index_history(index_code, page_index, page_size)
            .await
    }

    async fn fetch_fund_manager(&self, fund_code: &str) -> Result<FundManagerInfo, EastMoneyError> {
        self.fetch_fund_manager(fund_code).await
    }

    async fn fetch_fund_fee(&self, fund_code: &str) -> Result<FundFeeInfo, EastMoneyError> {
        self.fetch_fund_fee(fund_code).await
    }

    async fn fetch_fund_profile(&self, fund_code: &str) -> Result<FundProfile, EastMoneyError> {
        self.fetch_fund_profile(fund_code).await
    }
}

#[cfg(test)]
pub mod mock {
    use super::*;
    use std::collections::HashMap;

    fn not_implemented() -> EastMoneyError {
        EastMoneyError::ParseFailed("mock: method not stubbed".into())
    }

    /// 测试用内存数据源：按基金代码返回预设净值/名称及可选简报字段。
    pub struct MockFundDataSource {
        pub navs_by_code: HashMap<String, Vec<FundNav>>,
        pub names_by_code: HashMap<String, String>,
        pub profiles_by_code: HashMap<String, FundProfile>,
        pub industry_by_code: HashMap<String, FundIndustryReport>,
        pub holdings_by_code: HashMap<String, FundStockHoldingsReport>,
    }

    impl MockFundDataSource {
        pub fn with_navs(code: &str, name: &str, navs: Vec<FundNav>) -> Self {
            let mut navs_by_code = HashMap::new();
            navs_by_code.insert(code.to_string(), navs);
            let mut names_by_code = HashMap::new();
            names_by_code.insert(code.to_string(), name.to_string());
            Self {
                navs_by_code,
                names_by_code,
                profiles_by_code: HashMap::new(),
                industry_by_code: HashMap::new(),
                holdings_by_code: HashMap::new(),
            }
        }

        fn minimal_profile(code: &str, name: &str) -> FundProfile {
            FundProfile {
                code: code.to_string(),
                name: name.to_string(),
                full_name: name.to_string(),
                fund_type: "混合型".to_string(),
                establishment_date: String::new(),
                asset_size: "10.00亿".to_string(),
                company: "测试基金公司".to_string(),
                manager_name: String::new(),
                manager_tenure_days: 0,
                manager_total_return: 0.0,
                management_fee: 0.0,
                custody_fee: 0.0,
                investment_target: String::new(),
                investment_scope: String::new(),
                investment_strategy: String::new(),
                benchmark: String::new(),
                peer_rank: Default::default(),
            }
        }
    }

    #[async_trait]
    impl FundDataSource for MockFundDataSource {
        async fn fetch_nav_history(
            &self,
            fund_code: &str,
            _page_index: u32,
            page_size: u32,
        ) -> Result<(Vec<FundNav>, u32), EastMoneyError> {
            let navs = self
                .navs_by_code
                .get(fund_code)
                .cloned()
                .ok_or_else(not_implemented)?;
            let total = navs.len() as u32;
            let end = page_size.min(total) as usize;
            Ok((navs[..end].to_vec(), total))
        }

        async fn fetch_nav_history_by_days(
            &self,
            fund_code: &str,
            days: u32,
        ) -> Result<Vec<FundNav>, EastMoneyError> {
            let navs = self
                .navs_by_code
                .get(fund_code)
                .cloned()
                .ok_or_else(not_implemented)?;
            Ok(crate::nav_cache::filter_covering_calendar_days(navs, days))
        }

        async fn fetch_fund_name(&self, fund_code: &str) -> Result<String, EastMoneyError> {
            Ok(self
                .names_by_code
                .get(fund_code)
                .cloned()
                .unwrap_or_else(|| fund_code.to_string()))
        }

        async fn fetch_fund_ranking_top(
            &self,
            _: &str,
            _: &str,
            _: u32,
        ) -> Result<FundRankingPage, EastMoneyError> {
            Err(not_implemented())
        }

        async fn fetch_fund_industry_allocation(
            &self,
            fund_code: &str,
        ) -> Result<FundIndustryReport, EastMoneyError> {
            Ok(self
                .industry_by_code
                .get(fund_code)
                .cloned()
                .unwrap_or_default())
        }

        async fn fetch_fund_stock_holdings(
            &self,
            fund_code: &str,
            _: u32,
        ) -> Result<FundStockHoldingsReport, EastMoneyError> {
            Ok(self
                .holdings_by_code
                .get(fund_code)
                .cloned()
                .unwrap_or_default())
        }

        async fn search_fund(&self, _: &str) -> Result<Vec<(String, String)>, EastMoneyError> {
            Err(not_implemented())
        }

        async fn fetch_index_history(
            &self,
            _: &str,
            _: u32,
            _: u32,
        ) -> Result<(Vec<IndexData>, u32), EastMoneyError> {
            Err(not_implemented())
        }

        async fn fetch_fund_manager(&self, _: &str) -> Result<FundManagerInfo, EastMoneyError> {
            Err(not_implemented())
        }

        async fn fetch_fund_fee(&self, _: &str) -> Result<FundFeeInfo, EastMoneyError> {
            Err(not_implemented())
        }

        async fn fetch_fund_profile(&self, fund_code: &str) -> Result<FundProfile, EastMoneyError> {
            if let Some(p) = self.profiles_by_code.get(fund_code) {
                return Ok(p.clone());
            }
            let name = self
                .names_by_code
                .get(fund_code)
                .cloned()
                .unwrap_or_else(|| fund_code.to_string());
            Ok(MockFundDataSource::minimal_profile(fund_code, &name))
        }
    }
}
