//! 将 schemars / Clap 生成的 JSON Schema 写入 `schemas/` 目录。

use crate::presentation::StructuredFailureEnvelope;
use crate::schema::agent_tools::{embed_output_schemas, generate_agent_tools, write_agent_tools};
use crate::schema::responses::{
    AnalyzeSuccessEnvelope, BriefSuccessEnvelope, CompareSuccessEnvelope, ExportSuccessEnvelope,
    FetchSuccessEnvelope, HoldingsSuccessEnvelope, InfoSuccessEnvelope,
    PortfolioConfigSuccessEnvelope, PortfolioSuccessEnvelope, RankSuccessEnvelope,
    ResearchFundSuccessEnvelope, ScreenSuccessEnvelope, SectorsSuccessEnvelope,
    WatchlistSuccessEnvelope, SUCCESS_ENVELOPES,
};
use crate::schema::tools::write_tools;
use schemars::{schema_for, JsonSchema};
use serde_json::{json, Value};
use std::path::Path;

const SCHEMA_VERSION: &str = "1";

/// 导出全部 schema 到目录（工具入参 + 响应信封 + 核心模型 + 索引）。
pub fn export_all(output_dir: &Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(output_dir.join("responses"))?;
    std::fs::create_dir_all(output_dir.join("models"))?;

    write_tools(&output_dir.join("tools.v1.json"), true)?;
    write_agent_tools(&output_dir.join("tools.v1.agent.json"), true)?;

    write_schema::<StructuredFailureEnvelope>(&output_dir.join("envelope.failure.json"))?;

    write_schema::<AnalyzeSuccessEnvelope>(&output_dir.join("responses/analyze.success.json"))?;
    write_schema::<CompareSuccessEnvelope>(&output_dir.join("responses/compare.success.json"))?;
    write_schema::<PortfolioSuccessEnvelope>(&output_dir.join("responses/portfolio.success.json"))?;
    write_schema::<FetchSuccessEnvelope>(&output_dir.join("responses/fetch.success.json"))?;
    write_schema::<ExportSuccessEnvelope>(&output_dir.join("responses/export.success.json"))?;
    write_schema::<InfoSuccessEnvelope>(&output_dir.join("responses/info.success.json"))?;
    write_schema::<SectorsSuccessEnvelope>(&output_dir.join("responses/sectors.success.json"))?;
    write_schema::<HoldingsSuccessEnvelope>(&output_dir.join("responses/holdings.success.json"))?;
    write_schema::<RankSuccessEnvelope>(&output_dir.join("responses/rank.success.json"))?;
    write_schema::<BriefSuccessEnvelope>(&output_dir.join("responses/brief.success.json"))?;
    write_schema::<ScreenSuccessEnvelope>(&output_dir.join("responses/screen.success.json"))?;
    write_schema::<WatchlistSuccessEnvelope>(&output_dir.join("responses/watchlist.success.json"))?;
    write_schema::<PortfolioConfigSuccessEnvelope>(
        &output_dir.join("responses/portfolio_config.success.json"),
    )?;
    write_schema::<ResearchFundSuccessEnvelope>(
        &output_dir.join("responses/research_fund.success.json"),
    )?;

    let agent_tools = generate_agent_tools();
    let embedded = embed_output_schemas(&agent_tools, output_dir);
    std::fs::write(
        output_dir.join("tools.v1.agent.embedded.json"),
        format!(
            "{}\n",
            serde_json::to_string_pretty(&json!({
                "v": SCHEMA_VERSION,
                "generator": "fanalyzer schema export",
                "tools": embedded,
            }))?
        ),
    )?;

    write_schema::<crate::models::FundAnalysis>(&output_dir.join("models/fund_analysis.json"))?;
    write_schema::<crate::models::FundAnalysisReport>(
        &output_dir.join("models/fund_analysis_report.json"),
    )?;
    write_schema::<crate::models::PortfolioReport>(
        &output_dir.join("models/portfolio_report.json"),
    )?;
    write_schema::<crate::models::FundBrief>(&output_dir.join("models/fund_brief.json"))?;
    write_schema::<crate::models::FundOverview>(&output_dir.join("models/fund_overview.json"))?;
    write_schema::<crate::models::FundNav>(&output_dir.join("models/fund_nav.json"))?;

    write_index(output_dir)?;
    Ok(())
}

fn write_schema<T: JsonSchema>(path: &Path) -> anyhow::Result<()> {
    let schema = schema_for!(T);
    let json = serde_json::to_string_pretty(&schema)?;
    std::fs::write(path, format!("{json}\n"))?;
    Ok(())
}

fn write_index(output_dir: &Path) -> anyhow::Result<()> {
    let success: serde_json::Map<String, Value> = SUCCESS_ENVELOPES
        .iter()
        .map(|(cmd, file)| (cmd.to_string(), json!(file)))
        .collect();

    let index = json!({
        "v": SCHEMA_VERSION,
        "generator": "fanalyzer schema export",
        "tools": "tools.v1.json",
        "agent_tools": "tools.v1.agent.json",
        "agent_tools_embedded": "tools.v1.agent.embedded.json",
        "failure_envelope": "envelope.failure.json",
        "success_envelopes": success,
        "models": {
            "fund_analysis": "models/fund_analysis.json",
            "fund_analysis_report": "models/fund_analysis_report.json",
            "portfolio_report": "models/portfolio_report.json",
            "fund_brief": "models/fund_brief.json",
            "fund_overview": "models/fund_overview.json",
            "fund_nav": "models/fund_nav.json",
        },
        "usage": {
            "tools": "CLI 完整 MCP 入参 schema（自 Clap 生成）",
            "agent_tools": "Agent 专用入参 schema（剥离 compact 等内部参数）",
            "agent_tools_embedded": "Agent 工具 + 内联 outputSchema",
            "responses": "各 json 子命令 stdout 成功信封",
            "failure_envelope": "命令失败时 stdout 信封",
            "models": "data 字段常用子结构"
        }
    });
    std::fs::write(
        output_dir.join("index.json"),
        format!("{}\n", serde_json::to_string_pretty(&index)?),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn export_all_writes_index_and_tools() {
        let dir = TempDir::new().unwrap();
        export_all(dir.path()).unwrap();
        assert!(dir.path().join("index.json").exists());
        assert!(dir.path().join("tools.v1.json").exists());
        assert!(dir.path().join("tools.v1.agent.json").exists());
        assert!(dir.path().join("tools.v1.agent.embedded.json").exists());
        assert!(dir.path().join("responses/analyze.success.json").exists());
        assert!(dir.path().join("responses/watchlist.success.json").exists());
        assert!(dir.path().join("models/fund_analysis.json").exists());
    }
}
