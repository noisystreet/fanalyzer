//! 组合配置结构化查询。

use super::context::CommandContext;
use crate::portfolio::load_portfolio;
use crate::presentation::{base_meta, emit};
use schemars::JsonSchema;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Serialize, JsonSchema)]
pub struct PortfolioConfigPayload {
    pub name: String,
    pub holdings: Vec<PortfolioHoldingItem>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct PortfolioHoldingItem {
    pub code: String,
    pub weight: f64,
}

pub async fn run_portfolio_config(
    ctx: &CommandContext<'_>,
    portfolio_file: PathBuf,
) -> anyhow::Result<()> {
    let def = load_portfolio(&portfolio_file)?;
    let payload = PortfolioConfigPayload {
        name: def.name,
        holdings: def
            .holdings
            .into_iter()
            .map(|(code, weight)| PortfolioHoldingItem { code, weight })
            .collect(),
    };
    emit(
        ctx,
        "portfolio_config",
        &payload,
        Some(&base_meta(ctx)),
        None,
    )
}
