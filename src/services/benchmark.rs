//! 业绩比较基准 → 东方财富指数 secid 映射。

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IndexBenchmark {
    pub secid: &'static str,
    pub label: &'static str,
}

pub const HS300: IndexBenchmark = IndexBenchmark {
    secid: "1.000300",
    label: "沪深300",
};

const CSI500: IndexBenchmark = IndexBenchmark {
    secid: "1.000905",
    label: "中证500",
};

const CHINEXT: IndexBenchmark = IndexBenchmark {
    secid: "0.399006",
    label: "创业板指",
};

const SSE50: IndexBenchmark = IndexBenchmark {
    secid: "1.000016",
    label: "上证50",
};

const CSI1000: IndexBenchmark = IndexBenchmark {
    secid: "1.000852",
    label: "中证1000",
};

const SSE_COMPOSITE: IndexBenchmark = IndexBenchmark {
    secid: "1.000001",
    label: "上证指数",
};

const CSI_BOND: IndexBenchmark = IndexBenchmark {
    secid: "1.000832",
    label: "中证转债",
};

/// 从 F10「业绩比较基准」文案中推断指数；无法识别时返回 `None`。
pub fn resolve_index_from_text(text: &str) -> Option<IndexBenchmark> {
    let t = text;
    if contains_any(t, &["中证500", "CSI500", "500指数"]) {
        return Some(CSI500);
    }
    if contains_any(t, &["创业板", "创业板指", "399006"]) {
        return Some(CHINEXT);
    }
    if contains_any(t, &["上证50", "SSE50"]) {
        return Some(SSE50);
    }
    if contains_any(t, &["中证1000", "1000指数"]) {
        return Some(CSI1000);
    }
    if contains_any(t, &["沪深300", "HS300", "000300", "300指数"]) {
        return Some(HS300);
    }
    if contains_any(t, &["上证指数", "上证综指", "000001"]) {
        return Some(SSE_COMPOSITE);
    }
    if contains_any(t, &["中证转债", "可转债指数"]) {
        return Some(CSI_BOND);
    }
    None
}

/// 按基金类型兜底基准（契约文案不可解析时）。
pub fn default_index_for_fund_type(fund_type: &str) -> IndexBenchmark {
    let t = fund_type;
    if contains_any(t, &["债券", "货币", "理财"]) {
        return CSI_BOND;
    }
    if contains_any(t, &["指数", "ETF"]) {
        return HS300;
    }
    if contains_any(t, &["QDII"]) {
        return HS300;
    }
    HS300
}

/// 契约文案优先，否则按类型兜底。
pub fn resolve_benchmark(benchmark_text: &str, fund_type: &str) -> IndexBenchmark {
    resolve_index_from_text(benchmark_text)
        .unwrap_or_else(|| default_index_for_fund_type(fund_type))
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|n| haystack.contains(n))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_hs300() {
        let b = resolve_index_from_text("沪深300指数收益率×95%").unwrap();
        assert_eq!(b.secid, HS300.secid);
    }

    #[test]
    fn text_csi500() {
        let b = resolve_index_from_text("中证500指数").unwrap();
        assert_eq!(b.secid, CSI500.secid);
    }

    #[test]
    fn bond_type_default() {
        assert_eq!(
            default_index_for_fund_type("债券型-长债").secid,
            CSI_BOND.secid
        );
    }
}
