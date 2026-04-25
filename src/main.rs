use analysis_fund::api::eastmoney::EastMoneyClient;
use analysis_fund::config::AppConfig;
use analysis_fund::models::FundAnalysis;
use analysis_fund::services::FundAnalyzer;
use clap::Parser;
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
}

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .init();
}

fn print_analysis(analysis: &FundAnalysis) {
    println!("╔══════════════════════════════════════════════╗");
    println!("║  Fund Analysis Report                       ║");
    println!("╠══════════════════════════════════════════════╣");
    println!("║  Code:             {:<24}║", analysis.code);
    println!("║  Period:           {:>20} days ║", analysis.period_days);
    println!("╠══════════════════════════════════════════════╣");
    println!("║  Avg NAV:          {:>22.4} ║", analysis.avg_nav);
    println!("║  Max NAV:          {:>22.4} ║", analysis.max_nav);
    println!("║  Min NAV:          {:>22.4} ║", analysis.min_nav);
    println!("╠══════════════════════════════════════════════╣");
    println!(
        "║  Total Return:     {:>21.2}% ║",
        analysis.total_return * 100.0
    );
    println!(
        "║  Annualized Return:{:>21.2}% ║",
        analysis.annualized_return * 100.0
    );
    println!(
        "║  Volatility:       {:>21.2}% ║",
        analysis.volatility * 100.0
    );
    println!(
        "║  Max Drawdown:     {:>21.2}% ║",
        analysis.max_drawdown * 100.0
    );
    println!("╚══════════════════════════════════════════════╝");
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
        None => {
            Cli::parse_from(["analysis_fund", "--help"]);
        }
    }

    Ok(())
}
