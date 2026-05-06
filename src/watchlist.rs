//! 自选列表（TOML）。

use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct WatchlistToml {
    funds: Vec<String>,
}

/// 读取 `funds` 字符串列表，忽略空项。
pub fn load_watchlist(path: &Path) -> anyhow::Result<Vec<String>> {
    if !path.exists() {
        anyhow::bail!("自选文件不存在：{}", path.display());
    }
    let raw = std::fs::read_to_string(path)?;
    let w: WatchlistToml = toml::from_str(&raw)?;
    let funds: Vec<String> = w
        .funds
        .into_iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    Ok(funds)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn load_watchlist_ok() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(f, r#"funds = ["000001", "  110011  "]"#).unwrap();
        let v = load_watchlist(f.path()).unwrap();
        assert_eq!(v, vec!["000001", "110011"]);
    }

    #[test]
    fn load_watchlist_empty_vec() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(f, "funds = []").unwrap();
        let v = load_watchlist(f.path()).unwrap();
        assert!(v.is_empty());
    }
}
