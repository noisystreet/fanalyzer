//! 净值导出用例。

use super::context::{resolve_fund_ids, CommandContext};
use super::fund_service::{fetch_nav_series, resolve_fund_identifier};
use crate::presentation::{
    base_meta, emit, export_csv, export_json, item_error_failed, BatchPayload, ExportMeta,
    ExportPayload,
};
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
    output: Option<PathBuf>,
    days: u32,
    format: String,
}

pub async fn run_export(ctx: &CommandContext<'_>, req: ExportRequest) -> anyhow::Result<()> {
    if ctx.structured() && req.format != "json" {
        anyhow::bail!("`--json` 模式下 export 仅支持 `--format json`");
    }
    if req.pick_watchlist {
        export_watchlist(ctx, req).await
    } else {
        export_single(ctx, req).await
    }
}

async fn export_watchlist(ctx: &CommandContext<'_>, req: ExportRequest) -> anyhow::Result<()> {
    let ids = resolve_fund_ids(
        req.code.clone(),
        req.pick_watchlist,
        ctx.watchlist_path,
        "--code/--watchlist",
    )?;
    let requested = ids.len();
    if ctx.structured() {
        let mut items = Vec::with_capacity(requested);
        let mut errors = Vec::new();
        for id in ids {
            match export_watchlist_structured(ctx, &req, &id).await {
                Ok(payload) => items.push(payload),
                Err(e) => errors.push(item_error_failed(&id, e)),
            }
        }
        if items.is_empty() {
            anyhow::bail!("全部导出失败");
        }
        if !errors.is_empty() {
            ctx.warn(format!("{} 只标的导出失败", errors.len()));
        }
        let meta = ExportMeta {
            base: base_meta(ctx),
            days: req.days,
            requested,
            succeeded: items.len(),
        };
        emit(
            ctx,
            "export",
            &BatchPayload { items, errors },
            Some(&meta),
            None,
        )
    } else {
        let dir = req
            .output_dir
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("自选导出需要指定 --output-dir"))?;
        for id in ids {
            export_watchlist_to_dir(ctx, &req, dir, &id).await?;
        }
        Ok(())
    }
}

async fn export_watchlist_structured(
    ctx: &CommandContext<'_>,
    req: &ExportRequest,
    id: &str,
) -> anyhow::Result<ExportPayload> {
    let (resolved_code, name) = resolve_fund_identifier(&ctx.session, id, ctx.offline).await?;
    tracing::info!(code = %resolved_code, name = %name, "Export batch");
    let path = req
        .output_dir
        .as_ref()
        .map(|dir| Path::new(dir).join(format!("{resolved_code}.json")));
    export_one(
        &ctx.session,
        ExportOneParams {
            identifier: resolved_code,
            output: path,
            days: req.days,
            format: req.format.clone(),
        },
        ctx.offline,
        true,
    )
    .await
}

async fn export_watchlist_to_dir(
    ctx: &CommandContext<'_>,
    req: &ExportRequest,
    dir: &str,
    id: &str,
) -> anyhow::Result<()> {
    let (resolved_code, name) = resolve_fund_identifier(&ctx.session, id, ctx.offline).await?;
    tracing::info!(code = %resolved_code, name = %name, "Export batch");
    let path = Path::new(dir).join(match req.format.as_str() {
        "csv" => format!("{resolved_code}.csv"),
        "json" => format!("{resolved_code}.json"),
        other => anyhow::bail!("不支持的导出格式：{other}"),
    });
    let _ = export_one(
        &ctx.session,
        ExportOneParams {
            identifier: resolved_code,
            output: Some(path),
            days: req.days,
            format: req.format.clone(),
        },
        ctx.offline,
        false,
    )
    .await?;
    Ok(())
}

async fn export_single(ctx: &CommandContext<'_>, req: ExportRequest) -> anyhow::Result<()> {
    let oc = req
        .code
        .ok_or_else(|| anyhow::anyhow!("请指定 --code 或使用 --watchlist"))?;
    if !ctx.structured() {
        let out = req
            .output
            .ok_or_else(|| anyhow::anyhow!("单基金导出需要指定 --output"))?;
        let _ = export_one(
            &ctx.session,
            ExportOneParams {
                identifier: oc.trim().to_string(),
                output: Some(PathBuf::from(out)),
                days: req.days,
                format: req.format,
            },
            ctx.offline,
            false,
        )
        .await?;
        Ok(())
    } else {
        let payload = export_one(
            &ctx.session,
            ExportOneParams {
                identifier: oc.trim().to_string(),
                output: req.output.as_ref().map(PathBuf::from),
                days: req.days,
                format: req.format,
            },
            ctx.offline,
            true,
        )
        .await?;
        let meta = ExportMeta {
            base: base_meta(ctx),
            days: req.days,
            requested: 1,
            succeeded: 1,
        };
        emit(
            ctx,
            "export",
            &BatchPayload {
                items: vec![payload],
                errors: vec![],
            },
            Some(&meta),
            None,
        )
    }
}

async fn export_one(
    session: &super::context::Session<'_>,
    params: ExportOneParams,
    offline: bool,
    structured: bool,
) -> anyhow::Result<ExportPayload> {
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
        anyhow::bail!("`{}` 无净值数据可导出", resolved_code);
    }
    if !structured {
        let path = params
            .output
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("导出路径缺失"))?;
        let out_str = path.to_string_lossy();
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
    } else if let Some(path) = params.output {
        export_json(&navs, path.to_string_lossy().as_ref())?;
        tracing::info!(path = %path.display(), "Exported JSON");
    }
    Ok(ExportPayload {
        code: resolved_code,
        name,
        days: params.days,
        nav: navs,
    })
}
