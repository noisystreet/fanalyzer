//! 净值导出用例。

use super::context::CommandContext;
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

pub async fn run_export(ctx: &CommandContext<'_>, req: ExportRequest) -> anyhow::Result<()> {
    if req.pick_watchlist {
        let dir = req
            .output_dir
            .ok_or_else(|| anyhow::anyhow!("自选导出需要指定 --output-dir"))?;
        let ids = crate::watchlist::load_watchlist(ctx.watchlist_path)?;
        if ids.is_empty() {
            anyhow::bail!("自选列表为空或无有效项：{}", ctx.watchlist_path.display());
        }
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
                &resolved_code,
                path,
                req.days,
                ctx.offline,
                &req.format,
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
            oc.trim(),
            PathBuf::from(out),
            req.days,
            ctx.offline,
            &req.format,
        )
        .await
    }
}

async fn export_one(
    session: &super::context::Session<'_>,
    identifier: &str,
    output: PathBuf,
    days: u32,
    offline: bool,
    format: &str,
) -> anyhow::Result<()> {
    let (resolved_code, name) = resolve_fund_identifier(session, identifier, offline).await?;
    tracing::info!(code = %resolved_code, name = %name, days = days, output = ?output, "Export");
    let navs = fetch_nav_series(session, &resolved_code, days, offline).await?;
    if navs.is_empty() {
        tracing::warn!("No nav data {}", resolved_code);
        return Ok(());
    }
    let out_str = output.to_string_lossy();
    match format {
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
