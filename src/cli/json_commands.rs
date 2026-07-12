//! `json` 子命令组：与顶层业务子命令同参，输出结构化 JSON 信封。

use super::Commands;
use super::fund_code_arg::FundCodeArg;
use clap::Subcommand;
use std::path::PathBuf;

/// `fanalyzer json <子命令>` 可嵌套的业务命令（不含 `serve`）。
#[derive(Subcommand, Debug)]
pub enum JsonCommands {
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
        #[arg(short, long, help = "额外写入 JSON 文件路径")]
        output: Option<PathBuf>,
        #[arg(
            short,
            long,
            default_value = "json",
            help = "额外导出格式（目前支持 json）"
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
        #[arg(short, long, help = "额外写入 JSON 文件路径")]
        output: Option<PathBuf>,
        #[arg(short, long, default_value = "json", help = "额外导出格式 json")]
        format: String,
    },
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
        #[arg(short, long, help = "额外写入 JSON 文件路径")]
        output: Option<PathBuf>,
        #[arg(
            short,
            long,
            default_value = "json",
            help = "额外导出格式（目前支持 json）"
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
        #[arg(short, long, help = "单基金额外写入路径")]
        output: Option<String>,
        #[arg(long, help = "自选额外写入目录（每项生成 {代码}.json）")]
        output_dir: Option<String>,
        #[arg(short, long, default_value = "json", help = "额外文件格式（json）")]
        format: String,
    },
    Info {
        #[command(flatten)]
        fund_code: FundCodeArg,
        #[arg(long = "watchlist", help = "输出自选文件中所有基金的概况")]
        pick_watchlist: bool,
    },
    Sectors {
        #[command(flatten)]
        fund_code: FundCodeArg,
        #[arg(long = "watchlist", help = "输出自选文件中所有基金的行业配置")]
        pick_watchlist: bool,
    },
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
        #[arg(short, long, help = "忽略（json 模式 stdout 已为 JSON）")]
        output: Option<PathBuf>,
    },
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
        #[arg(short, long, help = "额外写入 JSON 文件路径")]
        output: Option<PathBuf>,
        #[arg(short, long, default_value = "json", help = "额外导出格式 json")]
        format: String,
    },
    /// 自选列表管理
    Watchlist {
        #[command(subcommand)]
        action: WatchlistAction,
    },
    /// 读取组合权重配置
    PortfolioConfig {
        #[arg(
            long = "portfolio-file",
            default_value = "config/portfolio.toml",
            value_name = "PATH"
        )]
        portfolio_file: PathBuf,
    },
}

#[derive(Subcommand, Debug)]
pub enum WatchlistAction {
    /// 列出自选基金
    List,
    /// 追加基金代码
    Add {
        #[arg(required = true)]
        codes: Vec<String>,
    },
    /// 移除基金代码
    Remove {
        #[arg(required = true)]
        codes: Vec<String>,
    },
}

// 字段逐一映射，属机械式 boilerplate。
#[allow(clippy::too_many_lines)]
impl From<JsonCommands> for Commands {
    fn from(value: JsonCommands) -> Self {
        match value {
            JsonCommands::Fetch {
                fund_code,
                pick_watchlist,
                limit,
            } => Commands::Fetch {
                fund_code,
                pick_watchlist,
                limit,
            },
            JsonCommands::Analyze {
                fund_code,
                pick_watchlist,
                days,
                period,
                output,
                format,
                rolling_window,
            } => Commands::Analyze {
                fund_code,
                pick_watchlist,
                days,
                period,
                output,
                format,
                rolling_window,
            },
            JsonCommands::Compare {
                codes,
                pick_watchlist,
                days,
                period,
                sort,
                output,
                format,
            } => Commands::Compare {
                codes,
                pick_watchlist,
                days,
                period,
                sort,
                output,
                format,
            },
            JsonCommands::Portfolio {
                portfolio_file,
                days,
                period,
                holdings_top,
                output,
                format,
                rolling_window,
            } => Commands::Portfolio {
                portfolio_file,
                days,
                period,
                holdings_top,
                output,
                format,
                rolling_window,
            },
            JsonCommands::Export {
                fund_code,
                pick_watchlist,
                days,
                output,
                output_dir,
                format,
            } => Commands::Export {
                fund_code,
                pick_watchlist,
                days,
                output,
                output_dir,
                format,
            },
            JsonCommands::Info {
                fund_code,
                pick_watchlist,
            } => Commands::Info {
                fund_code,
                pick_watchlist,
            },
            JsonCommands::Sectors {
                fund_code,
                pick_watchlist,
            } => Commands::Sectors {
                fund_code,
                pick_watchlist,
            },
            JsonCommands::Holdings {
                fund_code,
                pick_watchlist,
                top,
            } => Commands::Holdings {
                fund_code,
                pick_watchlist,
                top,
            },
            JsonCommands::Rank { kind, top, sort } => Commands::Rank { kind, top, sort },
            JsonCommands::Brief {
                fund_code,
                pick_watchlist,
                days,
                period,
                industry_top,
                holdings_top,
                output,
            } => Commands::Brief {
                fund_code,
                pick_watchlist,
                days,
                period,
                industry_top,
                holdings_top,
                output,
            },
            JsonCommands::Screen {
                kind,
                sort,
                rank_top,
                days,
                period,
                min_rank_return,
                max_drawdown,
                min_sharpe,
                max_mgmt_fee,
                min_alpha,
                max_volatility,
                min_total_return,
                deep_limit,
                full_scan,
                sort_by,
                limit,
                output,
                format,
            } => map_json_screen(
                kind,
                sort,
                rank_top,
                days,
                period,
                min_rank_return,
                max_drawdown,
                min_sharpe,
                max_mgmt_fee,
                min_alpha,
                max_volatility,
                min_total_return,
                deep_limit,
                full_scan,
                sort_by,
                limit,
                output,
                format,
            ),
            JsonCommands::Watchlist { action } => match action {
                WatchlistAction::List => Commands::WatchlistList,
                WatchlistAction::Add { codes } => Commands::WatchlistAdd { codes },
                WatchlistAction::Remove { codes } => Commands::WatchlistRemove { codes },
            },
            JsonCommands::PortfolioConfig { portfolio_file } => {
                Commands::PortfolioConfig { portfolio_file }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn map_json_screen(
    kind: String,
    sort: String,
    rank_top: u32,
    days: Option<u32>,
    period: Option<String>,
    min_rank_return: Option<f64>,
    max_drawdown: Option<f64>,
    min_sharpe: Option<f64>,
    max_mgmt_fee: Option<f64>,
    min_alpha: Option<f64>,
    max_volatility: Option<f64>,
    min_total_return: Option<f64>,
    deep_limit: u32,
    full_scan: bool,
    sort_by: Option<String>,
    limit: u32,
    output: Option<PathBuf>,
    format: String,
) -> Commands {
    Commands::Screen {
        kind,
        sort,
        rank_top,
        days,
        period,
        min_rank_return,
        max_drawdown,
        min_sharpe,
        max_mgmt_fee,
        min_alpha,
        max_volatility,
        min_total_return,
        deep_limit,
        full_scan,
        sort_by,
        limit,
        output,
        format,
    }
}
