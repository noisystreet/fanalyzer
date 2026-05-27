//! 应用层测试辅助（golden 信封、离线缓存 fixture）。

use crate::cache::FundCache;
use crate::models::FundNav;
use crate::nav_cache::NavCache;
use chrono::Local;
use serde_json::Value;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

/// 生成从 `points` 天前到今天的线性增长净值序列。
pub fn linear_nav_series(code: &str, points: usize) -> Vec<FundNav> {
    let today = Local::now().date_naive();
    (0..points)
        .map(|i| {
            let date = today - chrono::Duration::days((points - 1 - i) as i64);
            let nav = 1.0 + i as f64 * 0.001;
            FundNav {
                code: code.to_string(),
                date,
                nav,
                acc_nav: nav,
                daily_return: None,
            }
        })
        .collect()
}

/// 去掉信封中随运行变化的 meta 字段，便于 golden 断言。
pub fn strip_volatile_envelope_fields(mut v: Value) -> Value {
    if let Some(meta) = v.get_mut("meta").and_then(|m| m.as_object_mut()) {
        meta.remove("generated_at");
        meta.remove("duration_ms");
    }
    v
}

/// 写入双基金离线缓存（净值 + 名称映射）。
pub async fn seed_offline_two_funds(
    cache_root: &Path,
    funds: &[(&str, &str)],
) -> (NavCache, Arc<Mutex<FundCache>>) {
    let nav_store = NavCache::with_root(cache_root.to_path_buf());
    for (code, _) in funds {
        nav_store
            .save_merged(code, &linear_nav_series(code, 91))
            .expect("seed nav");
    }
    let name_cache = Arc::new(Mutex::new(FundCache::with_root(cache_root.to_path_buf())));
    {
        let mut guard = name_cache.lock().await;
        for (code, name) in funds {
            guard.set_mapping(code, name);
        }
    }
    (nav_store, name_cache)
}
