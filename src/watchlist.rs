//! 自选列表读写。

use serde::Deserialize;
use std::collections::BTreeSet;
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
    parse_watchlist_toml(&raw)
}

fn parse_watchlist_toml(raw: &str) -> anyhow::Result<Vec<String>> {
    let w: WatchlistToml = toml::from_str(raw)?;
    Ok(normalize_funds(w.funds))
}

fn normalize_funds(funds: Vec<String>) -> Vec<String> {
    funds
        .into_iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// 写入自选列表（覆盖）。
pub fn save_watchlist(path: &Path, funds: &[String]) -> anyhow::Result<()> {
    let funds = normalize_funds(funds.to_vec());
    let body = format!(
        "# 自选基金列表\nfunds = [{}]\n",
        funds
            .iter()
            .map(|c| format!("\"{c}\""))
            .collect::<Vec<_>>()
            .join(", ")
    );
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, body)?;
    Ok(())
}

/// 追加基金代码（去重，保持顺序）。
pub fn add_to_watchlist(path: &Path, codes: &[String]) -> anyhow::Result<Vec<String>> {
    let mut funds = if path.exists() {
        load_watchlist(path)?
    } else {
        Vec::new()
    };
    let mut seen: BTreeSet<String> = funds.iter().cloned().collect();
    for code in normalize_funds(codes.to_vec()) {
        if seen.insert(code.clone()) {
            funds.push(code);
        }
    }
    save_watchlist(path, &funds)?;
    Ok(funds)
}

/// 移除基金代码。
pub fn remove_from_watchlist(path: &Path, codes: &[String]) -> anyhow::Result<Vec<String>> {
    let remove: BTreeSet<String> = normalize_funds(codes.to_vec()).into_iter().collect();
    let funds: Vec<String> = load_watchlist(path)?
        .into_iter()
        .filter(|c| !remove.contains(c))
        .collect();
    save_watchlist(path, &funds)?;
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

    #[test]
    fn add_and_remove_watchlist() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("watchlist.toml");
        add_to_watchlist(&path, &["000001".into(), "110011".into()]).unwrap();
        let funds = add_to_watchlist(&path, &["000001".into(), "161725".into()]).unwrap();
        assert_eq!(funds, vec!["000001", "110011", "161725"]);
        let funds = remove_from_watchlist(&path, &["110011".into()]).unwrap();
        assert_eq!(funds, vec!["000001", "161725"]);
    }
}
