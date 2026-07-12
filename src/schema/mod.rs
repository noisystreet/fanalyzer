//! Agent / MCP JSON Schema 生成（Clap 工具入参 + schemars 响应模型）。

mod agent_tools;
mod export;
mod paths;
mod responses;
mod tools;

pub use agent_tools::{
    ToolTier, agent_tools_json, embed_output_schemas, filter_agent_tools, generate_agent_tools,
    resolve_output_schema, write_agent_tools,
};
pub use export::export_all;
pub use paths::{discover_schema_root, load_output_schema};
pub use tools::{generate_tools, tools_json, write_tools};

use std::path::PathBuf;

/// `fanalyzer schema` 子命令。
#[derive(clap::Subcommand, Debug)]
pub enum SchemaCommands {
    /// 从 Clap 定义导出 MCP / function-calling 工具 schema（JSON）
    Tools {
        /// 写入文件；省略则 stdout
        #[arg(short, long, value_name = "PATH")]
        output: Option<PathBuf>,
        /// pretty-print JSON
        #[arg(long, default_value_t = true)]
        pretty: bool,
    },
    /// 导出全部 schema 到目录（tools + 响应信封 + 核心模型 + index.json）
    Export {
        #[arg(long, default_value = "schemas", value_name = "DIR")]
        output_dir: PathBuf,
    },
}

pub async fn run(cmd: SchemaCommands) -> anyhow::Result<()> {
    match cmd {
        SchemaCommands::Tools { output, pretty } => {
            let json = tools_json(pretty)?;
            if let Some(path) = output {
                write_tools(&path, pretty)?;
                tracing::info!(path = %path.display(), "Wrote tool schemas");
            } else {
                print!("{json}");
            }
        }
        SchemaCommands::Export { output_dir } => {
            export_all(&output_dir)?;
            tracing::info!(dir = %output_dir.display(), "Exported JSON schemas");
        }
    }
    Ok(())
}
