//! CLI 入口：Clap 定义与启动 wiring。

mod dispatch;
mod dispatch_query;
mod dispatch_query_info;
mod dispatch_workflow;

use crate::api::eastmoney::{EastMoneyClient, EastMoneyClientOptions, EastMoneyError};
use crate::cache::FundCache;
use crate::config::AppConfig;
use crate::nav_cache::NavCache;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

pub fn map_client_err(e: EastMoneyError) -> anyhow::Error {
    anyhow::Error::msg(e.to_string())
}

#[derive(Parser, Debug)]
#[command(
    name = "fanalyzer",
    version,
    about = "Fanalyzer — fund analysis CLI & Web UI"
)]
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
        #[arg(
            short,
            long,
            default_value_t = 30,
            help = "分析窗口（日历天）；可被 --period 覆盖"
        )]
        days: u32,
        #[arg(
            long,
            help = "预设窗口：7d/1m/3m/6m/1y/ytd 或 rank 的 sc（1nzf/zzf 等）"
        )]
        period: Option<String>,
    },
    Compare {
        #[arg(short, long, help = "逗号分隔的基金代码或名称", value_delimiter = ',')]
        codes: Vec<String>,
        #[arg(long = "watchlist", help = "对比自选文件中所有基金")]
        pick_watchlist: bool,
        #[arg(
            short,
            long,
            default_value_t = 30,
            help = "分析窗口（日历天）；可被 --period 覆盖"
        )]
        days: u32,
        #[arg(long, help = "预设窗口：7d/1m/3m/6m/1y/ytd 或 rank 的 sc")]
        period: Option<String>,
        #[arg(
            long,
            help = "结果排序：sharpe/sortino/calmar/total-return/max-drawdown/alpha/volatility"
        )]
        sort: Option<String>,
        #[arg(short, long, help = "导出对比结果路径")]
        output: Option<PathBuf>,
        #[arg(short, long, default_value = "csv", help = "导出格式 csv 或 json")]
        format: String,
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
    /// 季报披露的行业配置（证监会行业分类，板块维度；数据源 F10 hypz）
    Sectors {
        #[arg(short, long, help = "基金代码或名称")]
        code: Option<String>,
        #[arg(long = "watchlist", help = "输出自选文件中所有基金的行业配置")]
        pick_watchlist: bool,
    },
    /// 季报股票投资明细（重仓股，`FundArchivesDatas type=jjcc`）
    Holdings {
        #[arg(short, long, help = "基金代码或名称")]
        code: Option<String>,
        #[arg(long = "watchlist", help = "输出自选文件中所有基金的重仓股")]
        pick_watchlist: bool,
        #[arg(
            short,
            long,
            default_value_t = 10,
            help = "展示条数对应接口 topline，1～50"
        )]
        top: u32,
    },
    /// 按天天基金官网排行拉取某类型全市场前 N 名（数据源需网络与 Referer）
    Rank {
        #[arg(short, long)]
        kind: String,
        #[arg(short, long, default_value_t = 100, help = "取前 N 名（≤500）")]
        top: u32,
        #[arg(
            long,
            value_name = "SC",
            default_value = "1n",
            help = "rankhandler 的排序字段 sc（默认 1n）；st 固定 desc"
        )]
        sort: String,
    },
    /// 单基金选基综合简报（分析 + 行业 + 重仓，可导出 Markdown）
    Brief {
        #[arg(short, long, help = "基金代码或名称")]
        code: Option<String>,
        #[arg(long = "watchlist", help = "对自选列表逐只生成简报")]
        pick_watchlist: bool,
        #[arg(
            short,
            long,
            default_value_t = 90,
            help = "净值分析窗口（日历天）；可被 --period 覆盖"
        )]
        days: u32,
        #[arg(long, help = "预设窗口：7d/1m/3m/6m/1y/ytd 或 rank 的 sc")]
        period: Option<String>,
        #[arg(long, default_value_t = 5, help = "行业配置展示前 N 项")]
        industry_top: u32,
        #[arg(long, default_value_t = 10, help = "重仓股展示条数（1～50）")]
        holdings_top: u32,
        #[arg(short, long, help = "同时写入 Markdown 报告路径")]
        output: Option<PathBuf>,
    },
    /// 从类型排行池中按回撤/夏普/费率筛选，并对比通过者
    Screen {
        #[arg(short, long, help = "排行类型，同 rank --kind")]
        kind: String,
        #[arg(
            long,
            value_name = "SC",
            default_value = "1n",
            help = "排行排序 sc，同 rank --sort"
        )]
        sort: String,
        #[arg(long, default_value_t = 30, help = "从排行前 N 只中扫描（5～100）")]
        rank_top: u32,
        #[arg(
            short,
            long,
            help = "deep 分析窗口（日历天）；省略时按 --sort 区间对齐"
        )]
        days: Option<u32>,
        #[arg(long, help = "预设窗口：7d/1m/3m/6m/1y/ytd 或 rank 的 sc")]
        period: Option<String>,
        #[arg(long, help = "排行区间收益下限（百分点，列与 --sort 一致）")]
        min_rank_return: Option<f64>,
        #[arg(long, help = "最大回撤上限（百分点，如 25）")]
        max_drawdown: Option<f64>,
        #[arg(long, help = "最低夏普比率")]
        min_sharpe: Option<f64>,
        #[arg(long, help = "管理费率上限（百分点，如 1.5）")]
        max_mgmt_fee: Option<f64>,
        #[arg(long, help = "最低 Alpha（百分点）")]
        min_alpha: Option<f64>,
        #[arg(long, help = "波动率上限（百分点）")]
        max_volatility: Option<f64>,
        #[arg(long, help = "区间总收益下限（百分点）")]
        min_total_return: Option<f64>,
        #[arg(
            long,
            default_value_t = 15,
            help = "deep 分析最多只数（默认 15，需 --full-scan 扫全池）"
        )]
        deep_limit: u32,
        #[arg(long, help = "对候选池全部做 deep 分析（较慢）")]
        full_scan: bool,
        #[arg(
            long,
            help = "通过筛选后按指标排序，默认 sharpe；可选 sortino/calmar/total-return 等"
        )]
        sort_by: Option<String>,
        #[arg(short, long, default_value_t = 10, help = "对比展示上限（2～30）")]
        limit: u32,
        #[arg(short, long, help = "导出对比结果路径")]
        output: Option<PathBuf>,
        #[arg(short, long, default_value = "csv", help = "导出格式 csv 或 json")]
        format: String,
    },
    /// 启动 Leptos SSR Web 界面（需编译 feature `web`）
    Serve {
        #[arg(long, default_value = "127.0.0.1", help = "监听地址")]
        host: String,
        #[arg(short, long, default_value_t = 3000, help = "监听端口")]
        port: u16,
    },
}

pub async fn run(mut cli: Cli, config: AppConfig) -> anyhow::Result<()> {
    let opts = EastMoneyClientOptions {
        timeout_secs: config.api.timeout_secs.max(1),
        user_agent: config.api.user_agent.clone(),
        proxy: config.api.proxy.clone(),
    };
    let client = EastMoneyClient::with_options(opts).map_err(map_client_err)?;
    let cache_root = config.cache_root();
    let name_cache = Arc::new(Mutex::new(FundCache::with_root(cache_root.clone())));
    let nav_store = NavCache::with_root(cache_root);

    let Some(cmd) = cli.command.take() else {
        Cli::parse_from(["fanalyzer", "--help"]);
        return Ok(());
    };

    match cmd {
        #[cfg(feature = "web")]
        Commands::Serve { host, port } => {
            return crate::web::run(&host, port, config, cli.watchlist_file).await;
        }
        #[cfg(not(feature = "web"))]
        Commands::Serve { .. } => {
            anyhow::bail!("Web 界面未编译进当前二进制；请使用: cargo run --features web -- serve");
        }
        cmd => {
            dispatch::dispatch_with_command(
                cmd,
                &client,
                &name_cache,
                &nav_store,
                cli.offline,
                &cli.watchlist_file,
            )
            .await
        }
    }
}
