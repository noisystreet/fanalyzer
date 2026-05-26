//! 组合权重配置（TOML 文件 + 页面文本编辑）。

use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct PortfolioDefinition {
    pub name: String,
    /// `(基金代码或名称, 权重)`，权重已归一化至合计 1.0
    pub holdings: Vec<(String, f64)>,
}

#[derive(Debug, Deserialize)]
struct PortfolioToml {
    name: Option<String>,
    holdings: Vec<PortfolioHoldingToml>,
}

#[derive(Debug, Deserialize)]
struct PortfolioHoldingToml {
    code: String,
    weight: f64,
}

/// 读取组合配置；权重之和不为 1 时在 `weight > 0` 前提下自动归一化。
pub fn load_portfolio(path: &Path) -> anyhow::Result<PortfolioDefinition> {
    if !path.exists() {
        anyhow::bail!("组合文件不存在：{}", path.display());
    }
    let raw = std::fs::read_to_string(path)?;
    let parsed: PortfolioToml = toml::from_str(&raw)?;
    let holdings: Vec<(String, f64)> = parsed
        .holdings
        .into_iter()
        .map(|h| (h.code.trim().to_string(), h.weight))
        .collect();
    build_portfolio(
        parsed.name.as_deref().filter(|s| !s.trim().is_empty()),
        holdings,
    )
}

/// 解析页面/表单文本：每行 `代码 权重`（空格、逗号或制表符分隔）；`#` 后为注释。
pub fn parse_holdings_text(raw: &str) -> anyhow::Result<Vec<(String, f64)>> {
    let mut holdings = Vec::new();
    for line in raw.lines() {
        let line = line.split('#').next().unwrap_or(line).trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line
            .split([',', '\t', ' '])
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();
        if parts.len() < 2 {
            anyhow::bail!("无法解析行「{line}」：需要「基金代码 权重」");
        }
        let code = parts[0].to_string();
        let weight: f64 = parts[1]
            .parse()
            .map_err(|_| anyhow::anyhow!("权重无效：{}", parts[1]))?;
        if weight > 0.0 {
            holdings.push((code, weight));
        }
    }
    if holdings.len() < 2 {
        anyhow::bail!("组合至少需要 2 只有效持仓（code 非空且 weight > 0）");
    }
    Ok(holdings)
}

/// 由名称与持仓列表构造组合（含权重归一化）。
pub fn build_portfolio(
    name: Option<&str>,
    holdings: Vec<(String, f64)>,
) -> anyhow::Result<PortfolioDefinition> {
    let mut holdings: Vec<(String, f64)> = holdings
        .into_iter()
        .map(|(c, w)| (c.trim().to_string(), w))
        .filter(|(c, w)| !c.is_empty() && *w > 0.0)
        .collect();
    if holdings.len() < 2 {
        anyhow::bail!("组合至少需要 2 只有效持仓（code 非空且 weight > 0）");
    }
    let sum: f64 = holdings.iter().map(|(_, w)| w).sum();
    if sum <= 0.0 {
        anyhow::bail!("组合权重之和必须大于 0");
    }
    if (sum - 1.0).abs() > 0.01 {
        tracing::warn!(
            sum = sum,
            "组合权重之和不为 1.0，将自动归一化（各 weight / {sum})"
        );
        for (_, w) in &mut holdings {
            *w /= sum;
        }
    }
    Ok(PortfolioDefinition {
        name: name
            .filter(|s| !s.trim().is_empty())
            .unwrap_or("portfolio")
            .trim()
            .to_string(),
        holdings,
    })
}

/// 由页面提交的文本构造组合。
pub fn portfolio_from_text(name: Option<&str>, raw: &str) -> anyhow::Result<PortfolioDefinition> {
    build_portfolio(name, parse_holdings_text(raw)?)
}

/// 将持仓格式化为可编辑文本（供 Web 表单默认值）。
pub fn format_holdings_text(holdings: &[(String, f64)]) -> String {
    holdings
        .iter()
        .map(|(c, w)| format!("{c} {w}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Web 组合页默认编辑内容：优先 portfolio 文件，其次自选等权，最后示例。
pub fn default_editor_content(portfolio_path: &Path, watchlist_path: &Path) -> (String, String) {
    if let Ok(def) = load_portfolio(portfolio_path) {
        return (def.name, format_holdings_text(&def.holdings));
    }
    if let Ok(funds) = crate::watchlist::load_watchlist(watchlist_path) {
        if funds.len() >= 2 {
            let w = 1.0 / funds.len() as f64;
            let text = funds
                .iter()
                .map(|c| format!("{c} {w:.4}"))
                .collect::<Vec<_>>()
                .join("\n");
            return ("watchlist-equal".to_string(), text);
        }
    }
    (
        "my-portfolio".to_string(),
        "000001 0.5\n110011 0.5".to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn load_portfolio_normalizes_weights() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(
            f,
            r#"
name = "test"
[[holdings]]
code = "000001"
weight = 1.0
[[holdings]]
code = "110011"
weight = 1.0
"#
        )
        .unwrap();
        let p = load_portfolio(f.path()).unwrap();
        assert_eq!(p.name, "test");
        assert_eq!(p.holdings.len(), 2);
        let s: f64 = p.holdings.iter().map(|(_, w)| w).sum();
        assert!((s - 1.0).abs() < 1e-9);
    }

    #[test]
    fn load_portfolio_needs_two_holdings() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(
            f,
            r#"
[[holdings]]
code = "000001"
weight = 1.0
"#
        )
        .unwrap();
        assert!(load_portfolio(f.path()).is_err());
    }

    #[test]
    fn parse_holdings_text_accepts_commas_and_comments() {
        let raw = "# demo\n000001, 0.6\n110011 0.4";
        let p = portfolio_from_text(Some("web"), raw).unwrap();
        assert_eq!(p.name, "web");
        assert_eq!(p.holdings.len(), 2);
        assert!((p.holdings[0].1 - 0.6).abs() < 1e-9);
    }

    #[test]
    fn format_holdings_text_roundtrip() {
        let holdings = vec![("000001".into(), 0.5), ("110011".into(), 0.5)];
        let text = format_holdings_text(&holdings);
        let p = portfolio_from_text(None, &text).unwrap();
        assert_eq!(p.holdings.len(), 2);
    }
}
