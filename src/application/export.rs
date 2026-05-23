//! 净值导出用例。

use super::context::{resolve_fund_ids, CommandContext};
use super::fund_service::{fetch_nav_series, resolve_fund_identifier};
use crate::presentation::{export_csv, export_json};
use std::path::{Path, PathBuf};

pub struct ExportRequest {
    pub code: Option<String>,
    pub pick_watchlist: bool,
    pub days: u32,
    pub output: Option<String>,
    pub output_dir: Option<String>,
    pub format: String,
}

struct ExportOneParams {
    identifier: String,
    output: PathBuf,
    days: u32,
    format: String,
}

pub async fn run_export(ctx: &CommandContext<'_>, req: ExportRequest) -> anyhow::Result<()> {
    if req.pick_watchlist {
        let dir = req
            .output_dir
            .ok_or_else(|| anyhow::anyhow!("自选导出需要指定 --output-dir"))?;
        let ids = resolve_fund_ids(
            req.code,
            req.pick_watchlist,
            ctx.watchlist_path,
            "--code/--watchlist",
        )?;
        for id in ids {
            let (resolved_code, name) =
                resolve_fund_identifier(&ctx.session, &id, ctx.offline).await?;
            tracing::info!(code = %resolved_code, name = %name, "Export batch");
            let path = Path::new(&dir).join(match req.format.as_str() {
                "csv" => format!("{resolved_code}.csv"),
                "json" => format!("{resolved_code}.json"),
                other => anyhow::bail!("不支持的导出格式：{other}"),
            });
            export_one(
                &ctx.session,
                ExportOneParams {
                    identifier: resolved_code,
                    output: path,
                    days: req.days,
                    format: req.format.clone(),
                },
                ctx.offline,
            )
            .await?;
        }
        Ok(())
    } else {
        let oc = req
            .code
            .ok_or_else(|| anyhow::anyhow!("请指定 --code 或使用 --watchlist"))?;
        let out = req
            .output
            .ok_or_else(|| anyhow::anyhow!("单基金导出需要指定 --output"))?;
        export_one(
            &ctx.session,
            ExportOneParams {
                identifier: oc.trim().to_string(),
                output: PathBuf::from(out),
                days: req.days,
                format: req.format,
            },
            ctx.offline,
        )
        .await
    }
}

async fn export_one(
    session: &super::context::Session<'_>,
    params: ExportOneParams,
    offline: bool,
) -> anyhow::Result<()> {
    let (resolved_code, name) =
        resolve_fund_identifier(session, &params.identifier, offline).await?;
    tracing::info!(
        code = %resolved_code,
        name = %name,
        days = params.days,
        output = ?params.output,
        "Export"
    );
    let navs = fetch_nav_series(session, &resolved_code, params.days, offline).await?;
    if navs.is_empty() {
        tracing::warn!("No nav data {}", resolved_code);
        return Ok(());
    }
    let out_str = params.output.to_string_lossy();
    match params.format.as_str() {
        "csv" => {
            export_csv(&navs, out_str.as_ref())?;
            tracing::info!(path = %out_str, "Exported CSV");
        }
        "json" => {
            export_json(&navs, out_str.as_ref())?;
            tracing::info!(path = %out_str, "Exported JSON");
        }
        other => anyhow::bail!("Unsupported format `{other}`"),
    }
    Ok(())
}
