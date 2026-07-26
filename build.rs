//! 构建时生成 Shell 自动补全脚本和 man page（供 `cargo deb` 打包用）。
//! CI 中 release 二进制会覆盖为精确输出。

// build.rs 为独立构建脚本，Command 树需要枚举所有子命令，
// 函数偏长是结构性原因，拆分反而降低可维护性。
#![allow(clippy::too_many_lines)]

use std::env;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("completions");
    std::fs::create_dir_all(&out_dir).unwrap();

    let mut cmd = build_cli();

    // Shell 补全
    clap_complete::generate(
        clap_complete::Shell::Bash,
        &mut cmd,
        "fanalyzer",
        &mut std::fs::File::create(out_dir.join("fanalyzer.bash")).unwrap(),
    );
    clap_complete::generate(
        clap_complete::Shell::Zsh,
        &mut cmd,
        "fanalyzer",
        &mut std::fs::File::create(out_dir.join("_fanalyzer")).unwrap(),
    );
    clap_complete::generate(
        clap_complete::Shell::Fish,
        &mut cmd,
        "fanalyzer",
        &mut std::fs::File::create(out_dir.join("fanalyzer.fish")).unwrap(),
    );

    // man page
    let man = clap_mangen::Man::new(cmd);
    man.render(&mut std::fs::File::create(out_dir.join("fanalyzer.1")).unwrap())
        .unwrap();

    println!("cargo:rerun-if-changed=build.rs");
}

/// 构建与 src/cli/mod.rs 同步的 Command 树。
fn build_cli() -> clap::Command {
    use clap::{Arg, Command};

    Command::new("fanalyzer")
        .version("0.1.0")
        .about("Fanalyzer — 基金分析 CLI")
        // 全局选项
        .arg(
            Arg::new("config")
                .long("config")
                .env("FANALYZER_CONFIG")
                .value_name("PATH")
                .help("配置文件路径"),
        )
        .arg(
            Arg::new("offline")
                .long("offline")
                .global(true)
                .help("仅从本地净值缓存读取数据"),
        )
        .arg(
            Arg::new("watchlist-file")
                .long("watchlist-file")
                .value_name("PATH")
                .default_value("config/watchlist.toml")
                .help("自选基金列表 TOML 文件"),
        )
        // 子命令
        .subcommand(
            Command::new("json")
                .visible_alias("structured")
                .about("结构化 JSON 输出（供 Agent 调用）")
                .arg(Arg::new("compact").long("compact").help("紧凑单行 JSON"))
                .arg(
                    Arg::new("compact-series")
                        .long("compact-series")
                        .help("省略时间序列曲线"),
                )
                .arg(
                    Arg::new("profile")
                        .long("profile")
                        .value_name("PROFILE")
                        .help("输出 profile"),
                ),
        )
        .subcommand(
            Command::new("fetch")
                .about("拉取基金净值历史数据")
                .arg(
                    Arg::new("code")
                        .short('c')
                        .long("code")
                        .value_name("CODE")
                        .help("基金代码"),
                )
                .arg(
                    Arg::new("name")
                        .short('n')
                        .long("name")
                        .value_name("NAME")
                        .help("基金名称"),
                )
                .arg(
                    Arg::new("watchlist")
                        .long("watchlist")
                        .help("使用自选文件中的所有基金"),
                )
                .arg(
                    Arg::new("limit")
                        .short('l')
                        .long("limit")
                        .default_value("20")
                        .help("拉取记录条数"),
                ),
        )
        .subcommand(
            Command::new("analyze")
                .about("综合分析：收益、风险、Alpha/Beta、经理评价")
                .arg(Arg::new("code").short('c').long("code").value_name("CODE"))
                .arg(Arg::new("name").short('n').long("name").value_name("NAME"))
                .arg(
                    Arg::new("watchlist")
                        .long("watchlist")
                        .help("分析自选文件中所有基金"),
                )
                .arg(
                    Arg::new("days")
                        .short('d')
                        .long("days")
                        .default_value("30")
                        .help("分析窗口（日历天）"),
                )
                .arg(Arg::new("period").long("period").help("预设窗口"))
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .value_name("PATH")
                        .help("导出路径"),
                )
                .arg(
                    Arg::new("format")
                        .short('f')
                        .long("format")
                        .default_value("json")
                        .help("导出格式"),
                ),
        )
        .subcommand(
            Command::new("compare")
                .about("多基金横向对比")
                .arg(
                    Arg::new("codes")
                        .short('c')
                        .long("codes")
                        .help("逗号分隔的基金代码"),
                )
                .arg(
                    Arg::new("watchlist")
                        .long("watchlist")
                        .help("对比自选文件中所有基金"),
                )
                .arg(Arg::new("days").short('d').long("days").default_value("30"))
                .arg(Arg::new("period").long("period").help("预设窗口"))
                .arg(Arg::new("sort").long("sort").help("结果排序"))
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .value_name("PATH"),
                )
                .arg(
                    Arg::new("format")
                        .short('f')
                        .long("format")
                        .default_value("csv"),
                ),
        )
        .subcommand(
            Command::new("portfolio")
                .about("组合分析")
                .arg(
                    Arg::new("portfolio-file")
                        .long("portfolio-file")
                        .value_name("PATH")
                        .help("组合权重 TOML 路径（默认 XDG）"),
                )
                .arg(Arg::new("days").short('d').long("days").default_value("90"))
                .arg(Arg::new("period").long("period"))
                .arg(
                    Arg::new("holdings-top")
                        .long("holdings-top")
                        .default_value("10"),
                )
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .value_name("PATH"),
                )
                .arg(
                    Arg::new("format")
                        .short('f')
                        .long("format")
                        .default_value("json"),
                ),
        )
        .subcommand(
            Command::new("export")
                .about("导出净值数据为 CSV / JSON")
                .arg(Arg::new("code").short('c').long("code").value_name("CODE"))
                .arg(Arg::new("name").short('n').long("name").value_name("NAME"))
                .arg(Arg::new("watchlist").long("watchlist"))
                .arg(Arg::new("days").short('d').long("days").default_value("30"))
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .value_name("PATH"),
                )
                .arg(
                    Arg::new("format")
                        .short('f')
                        .long("format")
                        .default_value("csv"),
                ),
        )
        .subcommand(
            Command::new("info")
                .about("基金概况：类型、规模、经理、费率等")
                .arg(Arg::new("code").short('c').long("code").value_name("CODE"))
                .arg(Arg::new("name").short('n').long("name").value_name("NAME"))
                .arg(Arg::new("watchlist").long("watchlist")),
        )
        .subcommand(
            Command::new("sectors")
                .about("季报行业配置（证监会行业分类）")
                .arg(Arg::new("code").short('c').long("code").value_name("CODE"))
                .arg(Arg::new("name").short('n').long("name").value_name("NAME"))
                .arg(Arg::new("watchlist").long("watchlist")),
        )
        .subcommand(
            Command::new("holdings")
                .about("季报重仓股明细")
                .arg(Arg::new("code").short('c').long("code").value_name("CODE"))
                .arg(Arg::new("name").short('n').long("name").value_name("NAME"))
                .arg(Arg::new("watchlist").long("watchlist"))
                .arg(
                    Arg::new("top")
                        .short('t')
                        .long("top")
                        .default_value("10")
                        .help("展示条数"),
                ),
        )
        .subcommand(
            Command::new("rank")
                .about("按类型排行：取全市场前 N 名")
                .arg(
                    Arg::new("kind")
                        .short('k')
                        .long("kind")
                        .required(true)
                        .help("排行类型"),
                )
                .arg(
                    Arg::new("top")
                        .short('t')
                        .long("top")
                        .default_value("100")
                        .help("取前 N 名"),
                )
                .arg(
                    Arg::new("sort")
                        .long("sort")
                        .default_value("1n")
                        .help("排序字段 sc"),
                ),
        )
        .subcommand(
            Command::new("brief")
                .about("选基简报：分析 + 行业 + 重仓")
                .arg(Arg::new("code").short('c').long("code").value_name("CODE"))
                .arg(Arg::new("name").short('n').long("name").value_name("NAME"))
                .arg(Arg::new("watchlist").long("watchlist"))
                .arg(Arg::new("days").short('d').long("days").default_value("90"))
                .arg(Arg::new("period").long("period"))
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .value_name("PATH"),
                ),
        )
        .subcommand(
            Command::new("screen")
                .about("多条件筛选 + 深度分析")
                .arg(Arg::new("kind").short('k').long("kind").required(true))
                .arg(Arg::new("sort").long("sort").default_value("1n"))
                .arg(Arg::new("rank-top").long("rank-top").default_value("30"))
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .value_name("PATH"),
                ),
        )
        .subcommand(Command::new("watchlist-list").about("列出自选基金"))
        .subcommand(
            Command::new("watchlist-add")
                .about("向自选追加基金代码")
                .arg(Arg::new("codes").required(true).num_args(1..)),
        )
        .subcommand(
            Command::new("watchlist-remove")
                .about("从自选移除基金代码")
                .arg(Arg::new("codes").required(true).num_args(1..)),
        )
        .subcommand(
            Command::new("portfolio-config")
                .about("读取组合权重配置")
                .arg(
                    Arg::new("portfolio-file")
                        .long("portfolio-file")
                        .value_name("PATH")
                        .help("组合权重 TOML 路径（默认 XDG）"),
                ),
        )
        .subcommand(
            Command::new("completions")
                .about("生成 Shell 自动补全脚本")
                .arg(
                    Arg::new("shell")
                        .required(true)
                        .value_parser(["bash", "zsh", "fish"])
                        .help("目标 Shell"),
                ),
        )
        .subcommand(Command::new("man").about("生成 man page（roff 格式）"))
        .subcommand(
            Command::new("schema")
                .about("导出 Agent JSON Schema")
                .subcommand(Command::new("export").about("导出 schema 文件"))
                .subcommand(Command::new("tools").about("输出 Agent 工具列表")),
        )
        .subcommand(
            Command::new("mcp").about("MCP Server").subcommand(
                Command::new("serve")
                    .about("启动 MCP stdio Server")
                    .arg(Arg::new("offline").long("offline").help("离线模式"))
                    .arg(
                        Arg::new("profile")
                            .long("profile")
                            .value_name("PROFILE")
                            .default_value("summary"),
                    )
                    .arg(
                        Arg::new("tools")
                            .long("tools")
                            .value_name("TIER")
                            .default_value("full"),
                    ),
            ),
        )
}
