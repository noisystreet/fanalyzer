use clap::Parser;
use fanalyzer::cli::{run, Cli};
use fanalyzer::config::AppConfig;

fn init_tracing() {
    use tracing_subscriber::EnvFilter;
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
    run(cli, config).await
}
