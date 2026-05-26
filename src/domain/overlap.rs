//! 重仓股重叠度（纯计算）。

use crate::models::StockHoldingRow;
use std::collections::HashMap;

/// 加权重叠度：对共同持仓取 `min(pct_a, pct_b)` 之和（占净值百分比，0～1）。
/// 返回 `(overlap_ratio, shared_count)`。
pub fn weighted_holdings_overlap(a: &[StockHoldingRow], b: &[StockHoldingRow]) -> (f64, usize) {
    let map_a: HashMap<&str, f64> = a
        .iter()
        .map(|r| (r.stock_code.as_str(), r.pct_nav / 100.0))
        .collect();
    let map_b: HashMap<&str, f64> = b
        .iter()
        .map(|r| (r.stock_code.as_str(), r.pct_nav / 100.0))
        .collect();
    let mut overlap = 0.0;
    let mut shared = 0usize;
    for (code, wa) in &map_a {
        if let Some(wb) = map_b.get(code) {
            overlap += wa.min(*wb);
            shared += 1;
        }
    }
    (overlap, shared)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(code: &str, pct: f64) -> StockHoldingRow {
        StockHoldingRow {
            rank: 1,
            stock_code: code.to_string(),
            stock_name: code.to_string(),
            pct_nav: pct,
            shares_wan: None,
            market_value_wan: None,
        }
    }

    #[test]
    fn weighted_overlap_sums_min_weights() {
        let a = vec![row("600000", 10.0), row("600001", 5.0)];
        let b = vec![row("600000", 8.0), row("600002", 4.0)];
        let (o, n) = weighted_holdings_overlap(&a, &b);
        assert_eq!(n, 1);
        assert!((o - 0.08).abs() < 1e-9);
    }

    #[test]
    fn no_overlap_zero() {
        let a = vec![row("600000", 10.0)];
        let b = vec![row("600001", 10.0)];
        let (o, n) = weighted_holdings_overlap(&a, &b);
        assert_eq!(n, 0);
        assert!((o - 0.0).abs() < 1e-9);
    }
}
