//! Web 服务共享状态。

use crate::api::eastmoney::EastMoneyClient;
use crate::application::CommandContext;
use crate::cache::FundCache;
use crate::nav_cache::NavCache;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Axum 共享状态（`Clone` 通过 `Arc`）。
#[derive(Clone)]
pub struct AppState {
    pub inner: Arc<AppStateInner>,
}

pub struct AppStateInner {
    pub client: EastMoneyClient,
    pub name_cache: Arc<Mutex<FundCache>>,
    pub nav_store: NavCache,
    pub watchlist_path: PathBuf,
    pub portfolio_path: PathBuf,
}

impl AppState {
    pub fn new(
        client: EastMoneyClient,
        name_cache: Arc<Mutex<FundCache>>,
        nav_store: NavCache,
        watchlist_path: PathBuf,
        portfolio_path: PathBuf,
    ) -> Self {
        Self {
            inner: Arc::new(AppStateInner {
                client,
                name_cache,
                nav_store,
                watchlist_path,
                portfolio_path,
            }),
        }
    }

    pub fn command_context(&self) -> CommandContext<'_> {
        CommandContext::new(
            &self.inner.client,
            &self.inner.name_cache,
            &self.inner.nav_store,
            false,
            &self.inner.watchlist_path,
        )
    }
}
