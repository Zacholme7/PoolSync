use std::collections::HashSet;
use reqwest::header::{HeaderMap, HeaderValue};
use crate::{Pool, PoolInfo};
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Deserialize)]
struct BirdeyeResponse {
    data: ResponseData,
}

#[derive(Debug, Deserialize)]
struct ResponseData {
    tokens: Vec<Token>,
}

#[derive(Debug, Deserialize)]
struct Token {
    address: String,
}

#[derive(Error, Debug)]
pub enum FilterError {
    #[error("API request failed: {0}")]
    ApiError(#[from] reqwest::Error),
    #[error("Environment variable not set: {0}")]
    EnvVarError(#[from] std::env::VarError),
    #[error("Invalid header value: {0}")]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),
}

pub async fn filter_top_volume(pools: Vec<Pool>, num_results: usize) -> Result<Vec<Pool>, FilterError> {
    let top_volume_tokens = query_birdeye(num_results).await?;
    let token_set: HashSet<_> = top_volume_tokens.into_iter().collect();

    let filtered_pools: Vec<Pool> = pools
        .into_iter()
        .filter(|pool| token_set.contains(&pool.token0().to_string()) && token_set.contains(&pool.token1().to_string()))
        .collect();

    Ok(filtered_pools)
}

async fn query_birdeye(num_results: usize) -> Result<Vec<String>, FilterError> {
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    let api_key = std::env::var("BIRDEYE_KEY")?;
    headers.insert("X-API-KEY", HeaderValue::from_str(&api_key)?);
    headers.insert("x-chain", HeaderValue::from_static("base"));

    let response = client
        .get("https://public-api.birdeye.so/defi/tokenlist")
        .headers(headers)
        .query(&[
            ("sort_by", "v24hUSD"),
            ("sort_type", "desc"),
            ("limit", &num_results.to_string()),
        ])
        .send()
        .await?;

    if response.status().is_success() {
        let birdeye_response: BirdeyeResponse = response.json().await?;
        Ok(birdeye_response.data.tokens.into_iter().map(|token| token.address).collect())
    } else {
        Err(FilterError::ApiError(response.error_for_status().unwrap_err()))
    }
}