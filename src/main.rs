use analysis_fund::api::eastmoney::EastMoneyClient;
use analysis_fund::config::AppConfig;
use analysis_fund::models::{FundAnalysis, FundNav};
use analysis_fund::services::FundAnalyzer;
use clap::Parser;
use std::fs::File;
use std::io::Write;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(name = "analysis_fund", version, about = "Fund analysis tool")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
    Fetch {
        #[arg(short, long, help = "Fund code to fetch")]
        code: String,
        #[arg(short, long, default_value = "20", help = "Number of records to fetch")]
        limit: u32,
    },
    Analyze {
        #[arg(short, long, help = "Fund code to analyze")]
        code: String,
        #[arg(short, long, default_value = "30", help = "Analysis period in days")]
        days: u32,
    },
    Compare {
        #[arg(short, long, help = "Fund codes to compare", value_delimiter = ',')]
        codes: Vec<String>,
        #[arg(short, long, default_value = "30", help = "Analysis period in days")]
        days: u32,
    },
    Export {
        #[arg(short, long, help = "Fund code to export")]
        code: String,
        #[arg(short, long, default_value = "30", help = "Analysis period in days")]
        days: u32,
        #[arg(short, long, help = "Output file path")]
        output: String,
        #[arg(
            short,
            long,
            default_value = "csv",
            help = "Export format: csv or json"
        )]
        format: String,
    },
}

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .init();
}

fn print_analysis(analysis: &FundAnalysis) {
    println!("基金分析报告");
    println!("基金代码: {}", analysis.code);
    println!("分析周期: {} 天", analysis.period_days);
    println!("平均净值: {:.4}", analysis.avg_nav);
    println!("最高净值: {:.4}", analysis.max_nav);
    println!("最低净值: {:.4}", analysis.min_nav);
    println!("总收益率: {:.2}%", analysis.total_return * 100.0);
    println!("年化收益率: {:.2}%", analysis.annualized_return * 100.0);
    println!("波动率: {:.2}%", analysis.volatility * 100.0);
    println!("最大回撤: {:.2}%", analysis.max_drawdown * 100.0);
    println!("夏普比率: {:.2}", analysis.sharpe_ratio);
}

fn print_comparison(analyses: &[FundAnalysis]) {
    println!("基金对比分析");
    println!();
    println!(
        "{:<10} {:>10} {:>12} {:>10} {:>10} {:>10}",
        "基金代码", "总收益率", "年化收益率", "波动率", "最大回撤", "夏普比率"
    );
    println!("{}", "-".repeat(70));
    for a in analyses {
        println!(
            "{:<10} {:>9.2}% {:>11.2}% {:>9.2}% {:>9.2}% {:>10.2}",
            a.code,
            a.total_return * 100.0,
            a.annualized_return * 100.0,
            a.volatility * 100.0,
            a.max_drawdown * 100.0,
            a.sharpe_ratio
        );
    }
}

fn export_csv(navs: &[FundNav], path: &str) -> anyhow::Result<()> {
    let mut writer = csv::Writer::from_path(path)?;
    writer.write_record(["date", "code", "nav", "acc_nav", "daily_return"])?;
    for nav in navs {
        writer.write_record([
            nav.date.to_string(),
            nav.code.clone(),
            nav.nav.to_string(),
            nav.acc_nav.to_string(),
            nav.daily_return.map(|r| r.to_string()).unwrap_or_default(),
        ])?;
    }
    writer.flush()?;
    Ok(())
}

fn export_json(navs: &[FundNav], path: &str) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(navs)?;
    let mut file = File::create(path)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let _config = AppConfig::load();

    let cli = Cli::parse();

    let client = EastMoneyClient::new();

    match cli.command {
        Some(Commands::Fetch { code, limit }) => {
            tracing::info!(code = %code, limit = limit, "Fetching fund nav history");
            let navs = client.fetch_nav_history(&code, 1, limit).await;
            match navs {
                Ok((nav_list, total)) => {
                    tracing::info!(total = total, fetched = nav_list.len(), "Fetched nav data");
                    println!(
                        "Fetched {} records (total: {}) for fund {}",
                        nav_list.len(),
                        total,
                        code
                    );
                    for nav in &nav_list {
                        println!(
                            "  {}  NAV: {:.4}  AccNAV: {:.4}  DailyReturn: {}",
                            nav.date,
                            nav.nav,
                            nav.acc_nav,
                            nav.daily_return
                                .map(|r| format!("{:.2}%", r * 100.0))
                                .unwrap_or_else(|| "N/A".to_string())
                        );
                    }
                }
                Err(e) => {
                    tracing::error!(error = %e, "Failed to fetch nav history");
                }
            }
        }
        Some(Commands::Analyze { code, days }) => {
            tracing::info!(code = %code, days = days, "Analyzing fund");
            let result = client.fetch_nav_history_by_days(&code, days).await;
            match result {
                Ok(navs) => {
                    tracing::info!(records = navs.len(), "Fetched nav data for analysis");
                    if navs.is_empty() {
                        tracing::warn!("No nav data available for fund {}", code);
                        return Ok(());
                    }
                    match FundAnalyzer::analyze(&navs, days) {
                        Some(analysis) => print_analysis(&analysis),
                        None => tracing::warn!("Insufficient data for analysis"),
                    }
                }
                Err(e) => {
                    tracing::error!(error = %e, "Failed to fetch nav data for analysis");
                }
            }
        }
        Some(Commands::Compare { codes, days }) => {
            if codes.is_empty() {
                tracing::error!("No fund codes provided for comparison");
                return Ok(());
            }
            tracing::info!(codes = ?codes, days = days, "Comparing funds");
            let mut analyses = Vec::new();
            for code in &codes {
                match client.fetch_nav_history_by_days(code, days).await {
                    Ok(navs) => {
                        if let Some(analysis) = FundAnalyzer::analyze(&navs, days) {
                            analyses.push(analysis);
                        } else {
                            tracing::warn!("Insufficient data for fund {}", code);
                        }
                    }
                    Err(e) => {
                        tracing::error!(code = %code, error = %e, "Failed to fetch data");
                    }
                }
            }
            if analyses.len() >= 2 {
                print_comparison(&analyses);
            } else {
                tracing::warn!("Need at least 2 funds for comparison");
            }
        }
        Some(Commands::Export {
            code,
            days,
            output,
            format,
        }) => {
            tracing::info!(code = %code, days = days, output = %output, format = %format, "Exporting fund data");
            match client.fetch_nav_history_by_days(&code, days).await {
                Ok(navs) => {
                    if navs.is_empty() {
                        tracing::warn!("No nav data available for fund {}", code);
                        return Ok(());
                    }
                    match format.as_str() {
                        "csv" => {
                            if let Err(e) = export_csv(&navs, &output) {
                                tracing::error!(error = %e, "Failed to export CSV");
                            } else {
                                tracing::info!(path = %output, "Exported to CSV");
                            }
                        }
                        "json" => {
                            if let Err(e) = export_json(&navs, &output) {
                                tracing::error!(error = %e, "Failed to export JSON");
                            } else {
                                tracing::info!(path = %output, "Exported to JSON");
                            }
                        }
                        _ => {
                            tracing::error!("Unsupported format: {}", format);
                        }
                    }
                }
                Err(e) => {
                    tracing::error!(error = %e, "Failed to fetch data for export");
                }
            }
        }
        None => {
            Cli::parse_from(["analysis_fund", "--help"]);
        }
    }

    Ok(())
}
