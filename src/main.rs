use clap::Parser;
use fanalyzer::cli::{run, Cli};
use fanalyzer::config::AppConfig;

fn init_tracing() {
    use tracing_subscriber::EnvFilter;
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .with_writer(std::io::stderr)
        .init();
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let cli = Cli::parse();
    let config = AppConfig::load(cli.config.as_deref());
    run(cli, config).await
}
