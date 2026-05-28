//! 净值 JSON 缓存（离线分析、减少重复请求）。

use crate::models::FundNav;
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
struct NavFile {
    fund_code: String,
    /// RFC3339
    updated_at: String,
    records: Vec<FundNav>,
}

pub struct NavCache {
    root: PathBuf,
}

impl Default for NavCache {
    fn default() -> Self {
        Self::new()
    }
}

impl NavCache {
    pub fn new() -> Self {
        let root = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from(".cache"))
            .join("fanalyzer")
            .join("nav");
        Self { root }
    }

    pub fn with_root(cache_root: PathBuf) -> Self {
        Self {
            root: cache_root.join("nav"),
        }
    }

    pub fn dir(&self) -> &PathBuf {
        &self.root
    }

    fn path_for(&self, fund_code: &str) -> PathBuf {
        self.root.join(format!("{fund_code}.json"))
    }

    pub fn exists(&self, fund_code: &str) -> bool {
        self.path_for(fund_code).exists()
    }

    pub fn load(&self, fund_code: &str) -> anyhow::Result<Vec<FundNav>> {
        let path = self.path_for(fund_code);
        let raw = fs::read_to_string(&path)
            .with_context(|| format!("读取净值缓存 {}", path.display()))?;
        let nf: NavFile = serde_json::from_str(&raw).context("解析净值缓存 JSON")?;
        Ok(nf.records)
    }

    /// 若缓存覆盖 `days` 窗口则返回截断后的净值，否则 `None`。
    pub fn load_covering_days(&self, fund_code: &str, days: u32) -> Option<Vec<FundNav>> {
        if !self.exists(fund_code) {
            return None;
        }
        let loaded = self.load(fund_code).ok()?;
        let trimmed = filter_covering_calendar_days(loaded, days);
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    }

    /// 与已有缓存按日期去重合并后写回。
    pub fn save_merged(&self, fund_code: &str, incoming: &[FundNav]) -> anyhow::Result<()> {
        let merged = if self.exists(fund_code) {
            let existing = self.load(fund_code).unwrap_or_default();
            merge_nav_slices(&existing, incoming)
        } else {
            incoming.to_vec()
        };

        let path = self.path_for(fund_code);
        if let Some(p) = path.parent() {
            fs::create_dir_all(p)?;
        }
        let nf = NavFile {
            fund_code: fund_code.to_string(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            records: sort_navs(merged),
        };
        fs::write(&path, serde_json::to_string_pretty(&nf)?)?;
        Ok(())
    }
}

fn sort_navs(mut v: Vec<FundNav>) -> Vec<FundNav> {
    v.sort_by_key(|n| n.date);
    v
}

fn merge_nav_slices(existing: &[FundNav], incoming: &[FundNav]) -> Vec<FundNav> {
    let mut m: BTreeMap<chrono::NaiveDate, FundNav> = BTreeMap::new();
    for n in existing.iter().cloned() {
        m.insert(n.date, n);
    }
    for n in incoming.iter().cloned() {
        m.insert(n.date, n);
    }
    m.into_values().collect()
}

/// 按自然日截断窗口（含 cutoff 当日及之后）。
pub fn filter_covering_calendar_days(mut navs: Vec<FundNav>, days: u32) -> Vec<FundNav> {
    if days == 0 {
        return Vec::new();
    }
    let today = chrono::Local::now().date_naive();
    let cutoff = today
        .checked_sub_signed(chrono::Duration::days(days as i64))
        .unwrap_or(today);
    navs.retain(|n| n.date >= cutoff);
    navs.sort_by_key(|n| n.date);
    navs
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn merge_prefers_incoming_on_same_date() {
        let a = FundNav {
            code: "000001".to_string(),
            date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            nav: 1.0,
            acc_nav: 1.0,
            daily_return: None,
        };
        let b = FundNav {
            code: "000001".to_string(),
            date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            nav: 1.1,
            acc_nav: 1.1,
            daily_return: Some(0.01),
        };
        let m = merge_nav_slices(std::slice::from_ref(&a), std::slice::from_ref(&b));
        assert_eq!(m.len(), 1);
        assert!((m[0].nav - 1.1).abs() < 1e-9);
    }

    #[test]
    fn filter_days_keeps_window() {
        let d0 = chrono::Local::now().date_naive();
        let d_old = d0 - chrono::Duration::days(400);
        let navs = vec![
            FundNav {
                code: "x".to_string(),
                date: d_old,
                nav: 1.0,
                acc_nav: 1.0,
                daily_return: None,
            },
            FundNav {
                code: "x".to_string(),
                date: d0,
                nav: 2.0,
                acc_nav: 2.0,
                daily_return: None,
            },
        ];
        let f = filter_covering_calendar_days(navs, 30);
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].date, d0);
    }

    #[test]
    fn load_covering_days_returns_none_when_stale() {
        let dir = tempfile::tempdir().unwrap();
        let cache = NavCache::with_root(dir.path().to_path_buf());
        let d_old = chrono::Local::now().date_naive() - chrono::Duration::days(400);
        cache
            .save_merged(
                "000001",
                &[FundNav {
                    code: "000001".to_string(),
                    date: d_old,
                    nav: 1.0,
                    acc_nav: 1.0,
                    daily_return: None,
                }],
            )
            .unwrap();
        assert!(cache.load_covering_days("000001", 30).is_none());
    }
}
