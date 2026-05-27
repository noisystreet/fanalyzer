//! 单次命令运行的会话上下文（无 Clap 依赖）。

use crate::api::eastmoney::EastMoneyClient;
use crate::cache::FundCache;
use crate::nav_cache::NavCache;
use std::cell::RefCell;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

/// HTTP 客户端与缓存句柄。
pub struct Session<'a> {
    pub client: &'a EastMoneyClient,
    pub name_cache: &'a Arc<Mutex<FundCache>>,
    pub nav_store: &'a NavCache,
}

/// `json` 子命令及相关输出选项。
#[derive(Debug, Clone, Copy)]
pub struct StructuredOutput {
    pub enabled: bool,
    pub json_compact: bool,
    pub compact_series: bool,
}

impl StructuredOutput {
    pub const OFF: Self = Self {
        enabled: false,
        json_compact: false,
        compact_series: false,
    };

    pub fn new(enabled: bool, json_compact: bool, compact_series: bool) -> Self {
        Self {
            enabled,
            json_compact,
            compact_series,
        }
    }
}

/// 命令级上下文：会话 + 离线/自选路径 + 结构化输出模式。
pub struct CommandContext<'a> {
    pub session: Session<'a>,
    pub offline: bool,
    pub watchlist_path: &'a Path,
    pub structured_output: StructuredOutput,
    warnings: RefCell<Vec<String>>,
}

impl<'a> CommandContext<'a> {
    pub fn new(
        client: &'a EastMoneyClient,
        name_cache: &'a Arc<Mutex<FundCache>>,
        nav_store: &'a NavCache,
        offline: bool,
        watchlist_path: &'a Path,
        structured_output: StructuredOutput,
    ) -> Self {
        Self {
            session: Session {
                client,
                name_cache,
                nav_store,
            },
            offline,
            watchlist_path,
            structured_output,
            warnings: RefCell::new(Vec::new()),
        }
    }

    pub fn structured(&self) -> bool {
        self.structured_output.enabled
    }

    pub fn json_compact(&self) -> bool {
        self.structured_output.json_compact
    }

    pub fn compact_series(&self) -> bool {
        self.structured_output.compact_series
    }

    /// 记录结构化输出警告（同时写 stderr 日志）。
    pub fn warn(&self, message: impl Into<String>) {
        let message = message.into();
        if self.structured() {
            self.warnings.borrow_mut().push(message.clone());
        }
        tracing::warn!(message = %message, "Structured warning");
    }

    /// 取出并清空已收集的 warnings（emit 时调用）。
    pub fn take_warnings(&self) -> Vec<String> {
        self.warnings.borrow_mut().drain(..).collect()
    }
}

pub fn require_online(offline: bool, cmd: &str) -> anyhow::Result<()> {
    if offline {
        anyhow::bail!("`{cmd}` 需要访问网络，勿使用 `--offline`");
    }
    Ok(())
}

/// 解析 `--code` 或 `--watchlist` 为基金标识列表。
pub fn resolve_fund_ids(
    code: Option<String>,
    pick_watchlist: bool,
    watchlist_path: &Path,
    flag_hint: &str,
) -> anyhow::Result<Vec<String>> {
    if pick_watchlist {
        let v = crate::watchlist::load_watchlist(watchlist_path)?;
        if v.is_empty() {
            anyhow::bail!("自选列表为空或无有效项：{}", watchlist_path.display());
        }
        Ok(v)
    } else {
        let c = code.ok_or_else(|| anyhow::anyhow!("请指定 `{flag_hint}`"))?;
        Ok(vec![c])
    }
}

pub fn resolve_many_fund_ids(
    codes: Vec<String>,
    pick_watchlist: bool,
    watchlist_path: &Path,
) -> anyhow::Result<Vec<String>> {
    if pick_watchlist {
        let v = crate::watchlist::load_watchlist(watchlist_path)?;
        if v.is_empty() {
            anyhow::bail!("自选列表为空或无有效项：{}", watchlist_path.display());
        }
        Ok(v)
    } else if codes.is_empty() {
        anyhow::bail!("请提供 --codes 或使用 --watchlist")
    } else {
        Ok(codes)
    }
}

/// 数据访问门面（`Session`）；后续可替换为 mock 实现。
pub type FundRepository<'a> = Session<'a>;

#[cfg(test)]
mod tests {
    use super::{require_online, resolve_fund_ids, resolve_many_fund_ids};
    use std::path::Path;

    #[test]
    fn require_online_blocks_offline() {
        assert!(require_online(true, "rank").is_err());
        assert!(require_online(false, "rank").is_ok());
    }

    #[test]
    fn resolve_fund_ids_needs_code() {
        let err = resolve_fund_ids(None, false, Path::new("/tmp/x"), "--code").unwrap_err();
        assert!(err.to_string().contains("--code"));
    }

    #[test]
    fn resolve_many_needs_codes() {
        assert!(resolve_many_fund_ids(vec![], false, Path::new("/tmp/x")).is_err());
    }
}
