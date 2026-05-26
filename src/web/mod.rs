//! Leptos SSR Web 界面（可选 feature `web`）。

mod chart_components;
mod charts;
mod components;
mod routes;
mod services;
mod state;

use crate::api::eastmoney::{into_anyhow, EastMoneyClient, EastMoneyClientOptions};
use crate::cache::FundCache;
use crate::config::AppConfig;
use crate::nav_cache::NavCache;
use state::AppState;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::trace::TraceLayer;

/// 启动 Web 服务。
pub async fn run(
    host: &str,
    port: u16,
    config: AppConfig,
    watchlist_path: PathBuf,
    portfolio_path: PathBuf,
) -> anyhow::Result<()> {
    let opts = EastMoneyClientOptions {
        timeout_secs: config.api.timeout_secs.max(1),
        user_agent: config.api.user_agent.clone(),
        proxy: config.api.proxy.clone(),
    };
    let client = EastMoneyClient::with_options(opts).map_err(into_anyhow)?;
    let cache_root = config.cache_root();
    let name_cache = Arc::new(Mutex::new(FundCache::with_root(cache_root.clone())));
    let nav_store = NavCache::with_root(cache_root);
    let state = AppState::new(
        client,
        name_cache,
        nav_store,
        watchlist_path,
        portfolio_path,
    );

    let app = routes::router(state).layer(TraceLayer::new_for_http());
    let addr: SocketAddr = format!("{host}:{port}").parse()?;
    tracing::info!(%addr, "Web UI listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
