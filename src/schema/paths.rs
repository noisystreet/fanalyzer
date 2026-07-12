//! Schema 目录发现（供 MCP tools/list 内联 outputSchema 与 Resources）。

use std::path::{Path, PathBuf};

/// 解析仓库内 `schemas/` 根目录（CWD → 可执行文件相对路径 → 编译期 manifest）。
pub fn discover_schema_root() -> PathBuf {
    let cwd = PathBuf::from("schemas");
    if cwd.join("index.json").exists() {
        return cwd;
    }

    if let Ok(exe) = std::env::current_exe()
        && let Some(exe_dir) = exe.parent()
    {
        for rel in ["schemas", "../schemas", "../../schemas"] {
            let candidate = exe_dir.join(rel);
            if candidate.join("index.json").exists() {
                return candidate;
            }
        }
    }

    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("schemas")
}

/// 读取 outputSchema 文件并返回内联 JSON Schema。
pub fn load_output_schema(path: &str, schema_root: &Path) -> Option<serde_json::Value> {
    let rel = path.strip_prefix("schemas/").unwrap_or(path);
    let full = schema_root.join(rel);
    let raw = std::fs::read_to_string(&full).ok()?;
    serde_json::from_str(&raw).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discover_schema_root_has_index() {
        let root = discover_schema_root();
        assert!(
            root.join("index.json").exists(),
            "expected index.json under {}",
            root.display()
        );
    }
}
