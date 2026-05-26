//! 东方财富相关 HTTP / 解析错误（供 `eastmoney` 与排行等子模块共用）。

use thiserror::Error;

#[derive(Error, Debug)]
pub enum EastMoneyError {
    #[error("HTTP request failed: {0}")]
    HttpFailed(#[from] reqwest::Error),
    #[error("API returned error code: {0}")]
    ApiError(i32),
    #[error("Failed to parse value: {0}")]
    ParseFailed(String),
    #[error("HTTP client configuration failed: {0}")]
    ClientBuildFailed(String),
}

/// 应用层 / Web 入口将 `EastMoneyError` 转为 `anyhow::Error`。
pub fn into_anyhow(e: EastMoneyError) -> anyhow::Error {
    anyhow::Error::msg(e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn into_anyhow_preserves_message() {
        let err = into_anyhow(EastMoneyError::ParseFailed("bad json".into()));
        assert!(err.to_string().contains("bad json"));
    }
}
