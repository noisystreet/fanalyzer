use analysis_fund::api::eastmoney::{EastMoneyClient, FundProfile};
use analysis_fund::cache::FundCache;
use analysis_fund::config::AppConfig;
use analysis_fund::models::{FundAnalysis, FundNav};
use analysis_fund::services::{BenchmarkData, FundAnalyzer, FundMetaInfo};
use clap::Parser;
use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use tokio::sync::Mutex;
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
    Info {
        #[arg(short, long, help = "Fund code to get info")]
        code: String,
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
    println!("基金名称: {}", analysis.name);
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
    println!("阿尔法 (Alpha): {:.2}%", analysis.alpha * 100.0);
    println!("贝塔 (Beta): {:.2}", analysis.beta);

    if !analysis.manager_name.is_empty() {
        println!("基金经理: {}", analysis.manager_name);
        let tenure_years = analysis.manager_tenure_days as f64 / 365.0;
        println!("经理任期: {:.1} 年", tenure_years);
        println!(
            "经理任职回报: {:.2}%",
            analysis.manager_total_return * 100.0
        );
    }

    if analysis.management_fee > 0.0 {
        println!("管理费率: {:.2}%", analysis.management_fee);
        println!("托管费率: {:.2}%", analysis.custody_fee);
    }
}

fn print_comparison(analyses: &[FundAnalysis]) {
    println!("基金对比分析");
    println!();
    println!(
        "{:<10} {:<16} {:>10} {:>12} {:>10} {:>10} {:>10} {:>10} {:>8} {:>8} {:>8}",
        "基金代码",
        "基金名称",
        "总收益率",
        "年化收益率",
        "波动率",
        "最大回撤",
        "夏普比率",
        "Alpha",
        "Beta",
        "管理费",
        "托管费"
    );
    println!("{}", "-".repeat(130));
    for a in analyses {
        let name = truncate_string(&a.name, 14);
        let mgmt_fee = if a.management_fee > 0.0 {
            format!("{:.2}%", a.management_fee)
        } else {
            "-".to_string()
        };
        let custody_fee = if a.custody_fee > 0.0 {
            format!("{:.2}%", a.custody_fee)
        } else {
            "-".to_string()
        };
        println!(
            "{:<10} {:<16} {:>9.2}% {:>11.2}% {:>9.2}% {:>9.2}% {:>10.2} {:>9.2}% {:>8.2} {:>8} {:>8}",
            a.code,
            name,
            a.total_return * 100.0,
            a.annualized_return * 100.0,
            a.volatility * 100.0,
            a.max_drawdown * 100.0,
            a.sharpe_ratio,
            a.alpha * 100.0,
            a.beta,
            mgmt_fee,
            custody_fee
        );
    }

    println!();
    println!("基金经理信息");
    println!("{}", "-".repeat(80));
    for a in analyses {
        if !a.manager_name.is_empty() {
            let tenure_years = a.manager_tenure_days as f64 / 365.0;
            println!(
                "{} {:<16} 经理: {:<10} 任期: {:>5.1}年 任职回报: {:>6.2}%",
                a.code,
                truncate_string(&a.name, 14),
                a.manager_name,
                tenure_years,
                a.manager_total_return * 100.0
            );
        }
    }
}

fn truncate_string(s: &str, max_chars: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max_chars {
        s.to_string()
    } else {
        format!("{}..", chars[..max_chars].iter().collect::<String>())
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

async fn resolve_fund_identifier(
    client: &EastMoneyClient,
    cache: &Arc<Mutex<FundCache>>,
    identifier: &str,
) -> (String, String) {
    let is_likely_code = identifier.chars().all(|c| c.is_ascii_digit()) && identifier.len() == 6;

    if is_likely_code {
        let name = get_fund_name(client, cache, identifier).await;
        return (identifier.to_string(), name);
    }

    {
        let cache_guard = cache.lock().await;
        if let Some(code) = cache_guard.get_code(identifier) {
            return (code, identifier.to_string());
        }
    }

    match client.search_fund(identifier).await {
        Ok(results) => {
            if let Some((code, name)) = results.first() {
                let mut cache_guard = cache.lock().await;
                cache_guard.set_mapping(code, name);
                return (code.clone(), name.clone());
            }
        }
        Err(e) => {
            tracing::warn!(identifier = %identifier, error = %e, "Failed to search fund");
        }
    }

    (identifier.to_string(), identifier.to_string())
}

async fn get_benchmark_data(client: &EastMoneyClient, days: u32) -> Option<BenchmarkData> {
    const HS300_CODE: &str = "1.000300";

    match client.fetch_index_history(HS300_CODE, 1, days * 2).await {
        Ok((data, _)) => {
            let mut dates = Vec::new();
            let mut returns = Vec::new();

            for i in 1..data.len() {
                let prev = &data[i - 1];
                let curr = &data[i];
                let daily_return = if prev.close != 0.0 {
                    (curr.close - prev.close) / prev.close
                } else {
                    0.0
                };
                dates.push(curr.date.date_naive());
                returns.push(daily_return);
            }

            Some(BenchmarkData { dates, returns })
        }
        Err(e) => {
            tracing::warn!(error = %e, "Failed to fetch benchmark data");
            None
        }
    }
}

async fn get_fund_name(
    client: &EastMoneyClient,
    cache: &Arc<Mutex<FundCache>>,
    code: &str,
) -> String {
    {
        let cache_guard = cache.lock().await;
        if let Some(name) = cache_guard.get_name(code) {
            return name;
        }
    }

    match client.fetch_fund_name(code).await {
        Ok(name) => {
            let mut cache_guard = cache.lock().await;
            cache_guard.set_mapping(code, &name);
            name
        }
        Err(e) => {
            tracing::warn!(code = %code, error = %e, "Failed to fetch fund name");
            code.to_string()
        }
    }
}

async fn get_fund_meta(client: &EastMoneyClient, code: &str) -> Option<FundMetaInfo> {
    let manager = match client.fetch_fund_manager(code).await {
        Ok(m) => m,
        Err(e) => {
            tracing::warn!(code = %code, error = %e, "Failed to fetch fund manager");
            return None;
        }
    };

    let fee = match client.fetch_fund_fee(code).await {
        Ok(f) => f,
        Err(e) => {
            tracing::warn!(code = %code, error = %e, "Failed to fetch fund fee");
            return None;
        }
    };

    Some(FundMetaInfo {
        manager_name: manager.name,
        manager_tenure_days: manager.tenure_days,
        manager_total_return: manager.total_return,
        management_fee: fee.management_fee,
        custody_fee: fee.custody_fee,
    })
}

async fn cmd_fetch(
    client: &EastMoneyClient,
    cache: &Arc<Mutex<FundCache>>,
    code: String,
    limit: u32,
) -> anyhow::Result<()> {
    let (resolved_code, name) = resolve_fund_identifier(client, cache, &code).await;
    tracing::info!(code = %resolved_code, name = %name, limit = limit, "Fetching fund nav history");
    let navs = client.fetch_nav_history(&resolved_code, 1, limit).await;
    match navs {
        Ok((nav_list, total)) => {
            tracing::info!(total = total, fetched = nav_list.len(), "Fetched nav data");
            println!(
                "Fetched {} records (total: {}) for fund {} ({})",
                nav_list.len(),
                total,
                resolved_code,
                name
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
    Ok(())
}

async fn cmd_analyze(
    client: &EastMoneyClient,
    cache: &Arc<Mutex<FundCache>>,
    code: String,
    days: u32,
) -> anyhow::Result<()> {
    let (resolved_code, name) = resolve_fund_identifier(client, cache, &code).await;
    tracing::info!(code = %resolved_code, name = %name, days = days, "Analyzing fund");
    let result = client.fetch_nav_history_by_days(&resolved_code, days).await;
    let benchmark = get_benchmark_data(client, days).await;
    let meta = get_fund_meta(client, &resolved_code).await;
    match result {
        Ok(navs) => {
            tracing::info!(records = navs.len(), "Fetched nav data for analysis");
            if navs.is_empty() {
                tracing::warn!("No nav data available for fund {}", code);
                return Ok(());
            }
            match FundAnalyzer::analyze(&navs, days, &name, benchmark.as_ref(), meta.as_ref()) {
                Some(analysis) => print_analysis(&analysis),
                None => tracing::warn!("Insufficient data for analysis"),
            }
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to fetch nav data for analysis");
        }
    }
    Ok(())
}

async fn cmd_compare(
    client: &EastMoneyClient,
    cache: &Arc<Mutex<FundCache>>,
    codes: Vec<String>,
    days: u32,
) -> anyhow::Result<()> {
    if codes.is_empty() {
        tracing::error!("No fund codes provided for comparison");
        return Ok(());
    }
    tracing::info!(codes = ?codes, days = days, "Comparing funds");
    let benchmark = get_benchmark_data(client, days).await;
    let mut analyses = Vec::new();
    for identifier in &codes {
        let (resolved_code, name) = resolve_fund_identifier(client, cache, identifier).await;
        let meta = get_fund_meta(client, &resolved_code).await;
        match client.fetch_nav_history_by_days(&resolved_code, days).await {
            Ok(navs) => {
                if let Some(analysis) =
                    FundAnalyzer::analyze(&navs, days, &name, benchmark.as_ref(), meta.as_ref())
                {
                    analyses.push(analysis);
                } else {
                    tracing::warn!("Insufficient data for fund {}", resolved_code);
                }
            }
            Err(e) => {
                tracing::error!(code = %resolved_code, error = %e, "Failed to fetch data");
            }
        }
    }
    if analyses.len() >= 2 {
        print_comparison(&analyses);
    } else {
        tracing::warn!("Need at least 2 funds for comparison");
    }
    Ok(())
}

async fn cmd_export(
    client: &EastMoneyClient,
    cache: &Arc<Mutex<FundCache>>,
    code: String,
    days: u32,
    output: String,
    format: String,
) -> anyhow::Result<()> {
    let (resolved_code, name) = resolve_fund_identifier(client, cache, &code).await;
    tracing::info!(code = %resolved_code, name = %name, days = days, output = %output, format = %format, "Exporting fund data");
    match client.fetch_nav_history_by_days(&resolved_code, days).await {
        Ok(navs) => {
            if navs.is_empty() {
                tracing::warn!("No nav data available for fund {}", resolved_code);
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
    Ok(())
}

async fn cmd_info(
    client: &EastMoneyClient,
    cache: &Arc<Mutex<FundCache>>,
    code: String,
) -> anyhow::Result<()> {
    let (resolved_code, _name) = resolve_fund_identifier(client, cache, &code).await;
    tracing::info!(code = %resolved_code, "Fetching fund info");
    match client.fetch_fund_profile(&resolved_code).await {
        Ok(profile) => print_fund_profile(&profile),
        Err(e) => {
            tracing::error!(error = %e, "Failed to fetch fund info");
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let _config = AppConfig::load();

    let cli = Cli::parse();

    let client = EastMoneyClient::new();
    let cache = Arc::new(Mutex::new(FundCache::new()));

    match cli.command {
        Some(Commands::Fetch { code, limit }) => cmd_fetch(&client, &cache, code, limit).await,
        Some(Commands::Analyze { code, days }) => cmd_analyze(&client, &cache, code, days).await,
        Some(Commands::Compare { codes, days }) => cmd_compare(&client, &cache, codes, days).await,
        Some(Commands::Export {
            code,
            days,
            output,
            format,
        }) => cmd_export(&client, &cache, code, days, output, format).await,
        Some(Commands::Info { code }) => cmd_info(&client, &cache, code).await,
        None => {
            Cli::parse_from(["analysis_fund", "--help"]);
            Ok(())
        }
    }
}

fn print_fund_profile(profile: &FundProfile) {
    println!("基金概况");
    println!("{}", "=".repeat(60));

    // 基本信息
    if !profile.full_name.is_empty() {
        println!("基金全称: {}", profile.full_name);
    }
    println!("基金简称: {}", profile.name);
    println!("基金代码: {}", profile.code);
    if !profile.fund_type.is_empty() {
        println!("基金类型: {}", profile.fund_type);
    }
    if !profile.establishment_date.is_empty() {
        println!("成立日期: {}", profile.establishment_date);
    }
    if !profile.asset_size.is_empty() {
        println!("资产规模: {}", profile.asset_size);
    }
    if !profile.company.is_empty() {
        println!("管理公司: {}", profile.company);
    }

    // 业绩比较基准
    if !profile.benchmark.is_empty() {
        println!();
        println!("业绩比较基准");
        println!("{}", "-".repeat(60));
        println!("{}", profile.benchmark);
    }

    println!();
    println!("基金经理");
    println!("{}", "-".repeat(60));
    println!("姓名: {}", profile.manager_name);
    let tenure_years = profile.manager_tenure_days as f64 / 365.0;
    println!("任期: {:.1} 年", tenure_years);
    println!("任职回报: {:.2}%", profile.manager_total_return * 100.0);

    println!();
    println!("费率信息");
    println!("{}", "-".repeat(60));
    println!("管理费率: {:.2}%", profile.management_fee);
    if profile.custody_fee > 0.0 {
        println!("托管费率: {:.2}%", profile.custody_fee);
    }

    // 投资目标
    if !profile.investment_target.is_empty() {
        println!();
        println!("投资目标");
        println!("{}", "-".repeat(60));
        println!("{}", profile.investment_target);
    }

    // 投资范围
    if !profile.investment_scope.is_empty() {
        println!();
        println!("投资范围");
        println!("{}", "-".repeat(60));
        // 投资范围通常较长，需要换行显示
        let scope = &profile.investment_scope;
        if scope.len() > 80 {
            // 按句子分割显示
            for sentence in scope.split('。').filter(|s| !s.is_empty()) {
                println!("{}", sentence.trim());
            }
        } else {
            println!("{}", scope);
        }
    }
}
