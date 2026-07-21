//! API DTO → models 视图映射（应用层边界）。

use crate::api::eastmoney::{FundProfile, PeerRankSnapshot};
use crate::api::fund_holdings::{FundStockHoldingRow, FundStockHoldingsReport};
use crate::api::fund_industry::{FundIndustryReport, FundIndustryRow};
use crate::api::fund_ranking::FundRankEntry;
use crate::models::reports::{
    FundOverview, FundRankRow, IndustryAllocation, IndustryRow, PeerRankInfo, StockHoldingRow,
    StockHoldings,
};

pub fn map_peer_rank(p: &PeerRankSnapshot) -> PeerRankInfo {
    PeerRankInfo {
        as_of: p.as_of.clone(),
        rank: p.rank,
        peer_count: p.peer_count,
        percentile: p.percentile,
    }
}

pub fn map_profile(p: &FundProfile) -> FundOverview {
    FundOverview {
        code: p.code.clone(),
        name: p.name.clone(),
        full_name: p.full_name.clone(),
        fund_type: p.fund_type.clone(),
        establishment_date: p.establishment_date.clone(),
        asset_size: p.asset_size.clone(),
        company: p.company.clone(),
        manager_name: p.manager_name.clone(),
        manager_tenure_days: p.manager_tenure_days,
        manager_total_return: p.manager_total_return,
        management_fee: p.management_fee,
        custody_fee: p.custody_fee,
        investment_target: p.investment_target.clone(),
        investment_scope: p.investment_scope.clone(),
        benchmark: p.benchmark.clone(),
        peer_rank: map_peer_rank(&p.peer_rank),
    }
}

fn map_industry_row(r: &FundIndustryRow) -> IndustryRow {
    IndustryRow {
        rank: r.rank,
        industry: r.industry.clone(),
        pct_nav: r.pct_nav,
        market_value_wan: r.market_value_wan,
    }
}

pub fn map_industry(report: &FundIndustryReport) -> IndustryAllocation {
    IndustryAllocation {
        as_of: report.as_of.clone(),
        rows: report.rows.iter().map(map_industry_row).collect(),
    }
}

fn map_holding_row(r: &FundStockHoldingRow) -> StockHoldingRow {
    StockHoldingRow {
        rank: r.rank,
        stock_code: r.stock_code.clone(),
        stock_name: r.stock_name.clone(),
        pct_nav: r.pct_nav,
        shares_wan: r.shares_wan,
        market_value_wan: r.market_value_wan,
    }
}

pub fn map_holdings(report: &FundStockHoldingsReport) -> StockHoldings {
    StockHoldings {
        as_of: report.as_of.clone(),
        rows: report.rows.iter().map(map_holding_row).collect(),
    }
}

pub fn map_rank_row(r: &FundRankEntry) -> FundRankRow {
    FundRankRow {
        code: r.code.clone(),
        name: r.name.clone(),
        pct_week: r.pct_week,
        pct_month: r.pct_month,
        pct_3m: r.pct_3m,
        pct_6m: r.pct_6m,
        pct_1y: r.pct_1y,
        pct_this_year: r.pct_this_year,
    }
}

pub fn map_rank_rows(rows: &[FundRankEntry]) -> Vec<FundRankRow> {
    rows.iter().map(map_rank_row).collect()
}
