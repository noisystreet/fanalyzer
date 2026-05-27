//! 集成测试共享 fixture（离线缓存 + 临时配置）。

use fanalyzer::models::FundNav;
use fanalyzer::nav_cache::NavCache;
use std::fs;
use std::path::{Path, PathBuf};

/// 生成从 `points` 天前到今天的线性增长净值序列。
pub fn linear_nav_series(code: &str, points: usize) -> Vec<FundNav> {
    let today = chrono::Local::now().date_naive();
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

/// 写入离线分析所需的净值缓存与名称映射。
pub fn write_offline_cache(cache_root: &Path, code: &str, name: &str, navs: &[FundNav]) {
    let nav_store = NavCache::with_root(cache_root.to_path_buf());
    nav_store.save_merged(code, navs).unwrap();
    let names_path = cache_root.join("fund_names.json");
    if let Some(parent) = names_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let mapping = serde_json::json!({ code: name });
    fs::write(names_path, serde_json::to_string_pretty(&mapping).unwrap()).unwrap();
}

/// 生成指向 `cache_root` 的临时配置文件路径。
pub fn write_config_with_cache_root(dir: &Path, cache_root: &Path) -> PathBuf {
    let cfg_path = dir.join("test-config.toml");
    let content = format!(
        r#"
[api]
base_url = "https://example.invalid"

[log]
level = "info"

[cache]
root = "{}"
"#,
        cache_root.display()
    );
    fs::write(&cfg_path, content).unwrap();
    cfg_path
}
