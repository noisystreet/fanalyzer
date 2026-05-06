//! 净值序列按日期去重合并（同日保留后者）。

use crate::models::FundNav;

pub fn merge_navs_by_date(navs: Vec<FundNav>) -> Vec<FundNav> {
    use std::collections::BTreeMap;
    navs.into_iter()
        .map(|n| (n.date, n))
        .collect::<BTreeMap<_, _>>()
        .into_values()
        .collect()
}

#[cfg(test)]
mod merge_tests {
    use super::merge_navs_by_date;
    use crate::models::FundNav;
    use chrono::NaiveDate;

    #[test]
    fn merge_navs_same_date_keeps_last() {
        let a = FundNav {
            code: "x".into(),
            date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            nav: 1.0,
            acc_nav: 1.0,
            daily_return: None,
        };
        let b = FundNav {
            code: "x".into(),
            date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            nav: 2.0,
            acc_nav: 2.0,
            daily_return: None,
        };
        let v = merge_navs_by_date(vec![a, b]);
        assert_eq!(v.len(), 1);
        assert!((v[0].nav - 2.0).abs() < 1e-9);
    }
}
