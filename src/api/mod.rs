use crate::models::{Fund, FundNav, ModelError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("HTTP request failed: {0}")]
    HttpFailed(#[from] reqwest::Error),
    #[error("Model error: {0}")]
    Model(#[from] ModelError),
    #[error("API rate limit exceeded")]
    RateLimited,
}

#[allow(dead_code)]
pub struct FundClient {
    client: reqwest::Client,
    base_url: String,
}

impl FundClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.to_string(),
        }
    }

    pub async fn fetch_fund_info(&self, _code: &str) -> Result<Fund, ApiError> {
        Err(ApiError::RateLimited)
    }

    pub async fn fetch_nav_history(&self, _code: &str) -> Result<Vec<FundNav>, ApiError> {
        Err(ApiError::RateLimited)
    }
}
