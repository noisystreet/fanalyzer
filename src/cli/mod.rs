//! CLI 编排：自选、离线缓存、`AppConfig` → HTTP 客户端。

mod handlers;
mod output;

pub use handlers::map_client_err;

use crate::api::eastmoney::{EastMoneyClient, EastMoneyClientOptions};
use crate::cache::FundCache;
use crate::config::AppConfig;
use crate::nav_cache::NavCache;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Parser, Debug)]
#[command(name = "analysis_fund", version, about = "Fund analysis tool")]
pub struct Cli {
    /// 仅从本地净值缓存读取数据（须曾在线抓取并写入缓存目录）
    #[arg(long, global = true)]
    pub offline: bool,
    #[arg(
        long,
        global = true,
        default_value = "config/watchlist.toml",
        value_name = "PATH"
    )]
    pub watchlist_file: PathBuf,
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Fetch {
        #[arg(short, long, help = "基金代码或名称")]
        code: Option<String>,
        #[arg(long = "watchlist", help = "使用自选文件中的所有基金")]
        pick_watchlist: bool,
        #[arg(short, long, default_value = "20", help = "拉取记录条数")]
        limit: u32,
    },
    Analyze {
        #[arg(short, long, help = "基金代码或名称")]
        code: Option<String>,
        #[arg(long = "watchlist", help = "分析自选文件中所有基金")]
        pick_watchlist: bool,
        #[arg(short, long, default_value = "30", help = "分析窗口（日历天）")]
        days: u32,
    },
    Compare {
        #[arg(short, long, help = "逗号分隔的基金代码或名称", value_delimiter = ',')]
        codes: Vec<String>,
        #[arg(long = "watchlist", help = "对比自选文件中所有基金")]
        pick_watchlist: bool,
        #[arg(short, long, default_value = "30", help = "分析窗口（日历天）")]
        days: u32,
    },
    Export {
        #[arg(short, long, help = "基金代码或名称")]
        code: Option<String>,
        #[arg(long = "watchlist", help = "导出自选文件中所有基金")]
        pick_watchlist: bool,
        #[arg(short, long, default_value = "30", help = "日历天窗口")]
        days: u32,
        #[arg(short, long, help = "单基金导出路径")]
        output: Option<String>,
        #[arg(long, help = "自选导出目录（每项生成 {代码}.{csv|json}）")]
        output_dir: Option<String>,
        #[arg(short, long, default_value = "csv", help = "csv 或 json")]
        format: String,
    },
    Info {
        #[arg(short, long, help = "基金代码或名称")]
        code: Option<String>,
        #[arg(long = "watchlist", help = "输出自选文件中所有基金的概况")]
        pick_watchlist: bool,
    },
    /// 按天天基金官网排行拉取某类型全市场前 N 名（数据源需网络与 Referer）
    Rank {
        /// 类型：gp/hh/zq/zs/qdii/fof，或 股票/混合/债券/指数
        #[arg(short, long)]
        kind: String,
        #[arg(short, long, default_value_t = 100, help = "取前 N 名（≤500）")]
        top: u32,
        #[arg(
            long,
            default_value = "1n",
            help = "官网排序参数 sc，如 1n 近一年、zzf 默认口径，详见天天基金排行页"
        )]
        sort: String,
    },
}

pub async fn run(cli: Cli, config: AppConfig) -> anyhow::Result<()> {
    let opts = EastMoneyClientOptions {
        timeout_secs: config.api.timeout_secs.max(1),
        user_agent: config.api.user_agent.clone(),
        proxy: config.api.proxy.clone(),
    };
    let client = EastMoneyClient::with_options(opts).map_err(map_client_err)?;
    let cache = Arc::new(Mutex::new(FundCache::new()));
    let nav_store = NavCache::new();

    handlers::execute(cli, &client, &cache, &nav_store).await
}
