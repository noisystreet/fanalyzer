//! CLI 入口：Clap 定义与启动 wiring。

mod dispatch;
mod dispatch_agent;
mod dispatch_query;
mod dispatch_query_handlers;
mod dispatch_query_info;
mod dispatch_workflow;
pub mod fund_code_arg;
mod json_commands;
pub mod structured_runner;

use fund_code_arg::FundCodeArg;
use json_commands::JsonCommands;

use crate::api::eastmoney::{into_anyhow, EastMoneyClient, EastMoneyClientOptions};
use crate::application::{CommandContext, OutputProfile, StructuredOutput};
use crate::cache::FundCache;
use crate::config::AppConfig;
use crate::nav_cache::NavCache;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

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
    /// 结构化 JSON 输出到 stdout（Agent / 自动化；日志在 stderr）
    #[command(
        name = "json",
        visible_alias = "structured",
        about = "结构化 JSON 输出（供 Agent 调用）"
    )]
    Json {
        /// 紧凑单行 JSON（无 pretty-print）
        #[arg(long)]
        compact: bool,
        /// 省略时间序列曲线以节省 token
        #[arg(long = "compact-series")]
        compact_series: bool,
        /// 输出 profile：summary（最省 token）/ standard / full
        #[arg(long, value_name = "PROFILE")]
        profile: Option<String>,
        #[command(subcommand)]
        command: JsonCommands,
    },
    Fetch {
        #[command(flatten)]
        fund_code: FundCodeArg,
        #[arg(long = "watchlist", help = "使用自选文件中的所有基金")]
        pick_watchlist: bool,
        #[arg(short, long, default_value = "20", help = "拉取记录条数")]
        limit: u32,
    },
    Analyze {
        #[command(flatten)]
        fund_code: FundCodeArg,
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
        #[arg(short, long, help = "导出分析报告路径（JSON，含时间序列）")]
        output: Option<PathBuf>,
        #[arg(
            short,
            long,
            default_value = "json",
            help = "导出格式（目前支持 json）"
        )]
        format: String,
        #[arg(
            long = "rolling-window",
            default_value_t = 60,
            help = "滚动指标窗口（交易日，10～252）"
        )]
        rolling_window: u32,
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
    /// 组合分析：加权收益、相关矩阵、重仓重叠（权重见 config/portfolio.toml）
    Portfolio {
        #[arg(
            long = "portfolio-file",
            default_value = "config/portfolio.toml",
            value_name = "PATH",
            help = "组合权重 TOML 路径"
        )]
        portfolio_file: PathBuf,
        #[arg(
            short,
            long,
            default_value_t = 90,
            help = "分析窗口（日历天）；可被 --period 覆盖"
        )]
        days: u32,
        #[arg(long, help = "预设窗口：7d/1m/3m/6m/1y/ytd 或 rank 的 sc")]
        period: Option<String>,
        #[arg(
            long,
            default_value_t = 10,
            help = "重仓重叠分析取前 N 大重仓（1～50，需联网）"
        )]
        holdings_top: u32,
        #[arg(short, long, help = "导出 JSON 报告路径")]
        output: Option<PathBuf>,
        #[arg(
            short,
            long,
            default_value = "json",
            help = "导出格式（目前支持 json）"
        )]
        format: String,
        #[arg(
            long = "rolling-window",
            default_value_t = 60,
            help = "滚动指标窗口（交易日，10～252）"
        )]
        rolling_window: u32,
    },
    Export {
        #[command(flatten)]
        fund_code: FundCodeArg,
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
        #[command(flatten)]
        fund_code: FundCodeArg,
        #[arg(long = "watchlist", help = "输出自选文件中所有基金的概况")]
        pick_watchlist: bool,
    },
    /// 季报披露的行业配置（证监会行业分类，板块维度；数据源 F10 hypz）
    Sectors {
        #[command(flatten)]
        fund_code: FundCodeArg,
        #[arg(long = "watchlist", help = "输出自选文件中所有基金的行业配置")]
        pick_watchlist: bool,
    },
    /// 季报股票投资明细（重仓股，`FundArchivesDatas type=jjcc`）
    Holdings {
        #[command(flatten)]
        fund_code: FundCodeArg,
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
        #[command(flatten)]
        fund_code: FundCodeArg,
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
    /// 列出自选基金（结构化 JSON）
    WatchlistList,
    /// 向自选追加基金代码
    WatchlistAdd {
        #[arg(required = true)]
        codes: Vec<String>,
    },
    /// 从自选移除基金代码
    WatchlistRemove {
        #[arg(required = true)]
        codes: Vec<String>,
    },
    /// 读取组合权重配置（结构化 JSON）
    PortfolioConfig {
        #[arg(
            long = "portfolio-file",
            default_value = "config/portfolio.toml",
            value_name = "PATH"
        )]
        portfolio_file: PathBuf,
    },
    /// 导出 Agent JSON Schema（Clap 工具入参 + schemars 响应模型；无需联网）
    Schema {
        #[command(subcommand)]
        command: crate::schema::SchemaCommands,
    },
    /// MCP Server（stdio，供 Cursor / Claude Desktop 集成）
    Mcp {
        #[command(subcommand)]
        command: crate::mcp::McpCommands,
    },
    /// 启动 Leptos SSR Web 界面（需编译 feature `web`）
    Serve {
        #[arg(long, default_value = "127.0.0.1", help = "监听地址")]
        host: String,
        #[arg(short, long, default_value_t = 3000, help = "监听端口")]
        port: u16,
        #[arg(
            long = "portfolio-file",
            default_value = "config/portfolio.toml",
            value_name = "PATH",
            help = "Web 组合分析默认权重文件"
        )]
        portfolio_file: PathBuf,
    },
}

impl Commands {
    /// 子命令名（结构化 JSON 信封 `command` 字段）。
    pub fn name(&self) -> &'static str {
        match self {
            Self::Json { .. } => unreachable!("json wrapper resolved before dispatch"),
            Self::Fetch { .. } => "fetch",
            Self::Analyze { .. } => "analyze",
            Self::Compare { .. } => "compare",
            Self::Portfolio { .. } => "portfolio",
            Self::Export { .. } => "export",
            Self::Info { .. } => "info",
            Self::Sectors { .. } => "sectors",
            Self::Holdings { .. } => "holdings",
            Self::Rank { .. } => "rank",
            Self::Brief { .. } => "brief",
            Self::Screen { .. } => "screen",
            Self::WatchlistList => "watchlist",
            Self::WatchlistAdd { .. } => "watchlist",
            Self::WatchlistRemove { .. } => "watchlist",
            Self::PortfolioConfig { .. } => "portfolio_config",
            Self::Schema { .. } => "schema",
            Self::Mcp { .. } => "mcp",
            Self::Serve { .. } => "serve",
        }
    }
}

async fn execute_command(
    cmd: Commands,
    structured_output: StructuredOutput,
    offline: bool,
    watchlist_path: &std::path::Path,
    client: &EastMoneyClient,
    name_cache: &Arc<Mutex<FundCache>>,
    nav_store: &NavCache,
) -> anyhow::Result<()> {
    let cmd_name = cmd.name();
    let result = dispatch::dispatch_with_command(
        cmd,
        client,
        name_cache,
        nav_store,
        offline,
        watchlist_path,
        structured_output,
    )
    .await;
    if structured_output.enabled {
        if let Err(e) = result {
            let err_ctx = CommandContext::new(
                client,
                name_cache,
                nav_store,
                offline,
                watchlist_path,
                structured_output,
            );
            crate::presentation::print_failure_from_anyhow(&err_ctx, cmd_name, &e)?;
            std::process::exit(1);
        }
        Ok(())
    } else {
        result
    }
}

pub async fn run(mut cli: Cli, config: AppConfig) -> anyhow::Result<()> {
    let opts = EastMoneyClientOptions {
        timeout_secs: config.api.timeout_secs.max(1),
        user_agent: config.api.user_agent.clone(),
        proxy: config.api.proxy.clone(),
    };
    let client = EastMoneyClient::with_options(opts).map_err(into_anyhow)?;
    let cache_root = config.cache_root();
    let name_cache = Arc::new(Mutex::new(FundCache::with_root(cache_root.clone())));
    let nav_store = NavCache::with_root(cache_root);

    let Some(cmd) = cli.command.take() else {
        Cli::parse_from(["fanalyzer", "--help"]);
        return Ok(());
    };

    match cmd {
        #[cfg(feature = "web")]
        Commands::Serve {
            host,
            port,
            portfolio_file,
        } => {
            return crate::web::run(&host, port, config, cli.watchlist_file, portfolio_file).await;
        }
        #[cfg(not(feature = "web"))]
        Commands::Serve { .. } => {
            anyhow::bail!("Web 界面未编译进当前二进制；请使用: cargo run --features web -- serve");
        }
        Commands::Schema { command } => crate::schema::run(command).await,
        Commands::Mcp { command } => crate::mcp::run(command, config).await,
        Commands::Json {
            compact,
            compact_series,
            profile,
            command,
        } => {
            let inner: Commands = command.into();
            execute_command(
                inner,
                json_structured_output(compact, compact_series, profile)?,
                cli.offline,
                &cli.watchlist_file,
                &client,
                &name_cache,
                &nav_store,
            )
            .await
        }
        cmd => {
            execute_command(
                cmd,
                StructuredOutput::OFF,
                cli.offline,
                &cli.watchlist_file,
                &client,
                &name_cache,
                &nav_store,
            )
            .await
        }
    }
}

fn json_structured_output(
    compact: bool,
    compact_series: bool,
    profile: Option<String>,
) -> anyhow::Result<StructuredOutput> {
    if let Some(p) = profile {
        let profile = OutputProfile::parse(&p)?;
        Ok(StructuredOutput::with_profile(
            true,
            profile.json_compact(),
            profile.compact_series(),
            Some(profile),
        ))
    } else {
        Ok(StructuredOutput::new(true, compact, compact_series))
    }
}
