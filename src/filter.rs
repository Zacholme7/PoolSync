use crate::{Pool, PoolInfo};
use alloy::primitives::Address;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::Deserialize;
use std::collections::HashSet;
use std::str::FromStr;
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

pub async fn filter_top_volume(pools: Vec<Pool>, num_results: usize) -> Vec<Address> {
    let top_volume_tokens = query_birdeye(num_results).await;
    top_volume_tokens
        .into_iter()
        .map(|addr| Address::from_str(&addr).unwrap())
        .collect()
}

async fn query_birdeye(num_results: usize) -> Vec<String> {
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    let api_key = std::env::var("BIRDEYE_KEY").unwrap();
    headers.insert("X-API-KEY", HeaderValue::from_str(&api_key).unwrap());
    headers.insert("x-chain", HeaderValue::from_static("ethereum"));

    let mut query_params: Vec<(usize, usize)> = Vec::new();

    if num_results < 50 {
        query_params.push((0, num_results));
    } else {
        for offset in (0..num_results).step_by(50) {
            query_params.push((offset, 50));
        }
    }

    let mut addresses: Vec<String> = Vec::new();
    for (offset, num) in query_params {
        let response = client
            .get("https://public-api.birdeye.so/defi/tokenlist")
            .headers(headers.clone())
            .query(&[
                ("sort_by", "v24hUSD"),
                ("sort_type", "desc"),
                ("offset", &offset.to_string()),
                ("limit", &num.to_string()),
            ])
            .send()
            .await
            .unwrap();
        if response.status().is_success() {
            let birdeye_response: BirdeyeResponse = response.json().await.unwrap();
            let results: Vec<String> = birdeye_response
                .data
                .tokens
                .into_iter()
                .map(|token| token.address)
                .collect();
            addresses.extend(results);
        }
    }

    addresses
}
