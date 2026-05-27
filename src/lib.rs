pub mod api;
pub mod application;
pub mod cache;
pub mod cli;
pub mod config;
pub mod domain;
pub mod insight_config;
pub mod mcp;
pub mod models;
pub mod nav_cache;
pub mod portfolio;
pub mod presentation;
pub mod schema;
/// 兼容层：新代码请使用 `domain` / `application`。
pub mod services;
pub mod watchlist;

#[cfg(feature = "web")]
pub mod web;
