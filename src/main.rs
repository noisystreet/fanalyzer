use analysis_fund::api::FundClient;
use analysis_fund::config::AppConfig;
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let config = AppConfig::load();

    let cli = Cli::parse();

    let client = FundClient::new(&config.api.base_url);

    match cli.command {
        Some(Commands::Fetch { code }) => {
            tracing::info!(code = %code, "Fetching fund data");
            match client.fetch_fund_info(&code).await {
                Ok(_fund) => println!("Fetched fund info for: {}", code),
                Err(e) => tracing::error!(error = %e, "Failed to fetch fund info"),
            }
        }
        Some(Commands::Analyze { code, days }) => {
            tracing::info!(code = %code, days = days, "Analyzing fund");
            println!("Analyzing fund {} over {} days", code, days);
        }
        None => {
            Cli::parse_from(["analysis_fund", "--help"]);
        }
    }

    Ok(())
}
