use crate::Chain;
use crate::{Pool, PoolInfo};
use alloy::dyn_abi::DynSolType;
use alloy::network::Network;
use alloy::primitives::U256;
use alloy::primitives::{address, Address};
use alloy::providers::Provider;
use alloy::sol;
use alloy::transports::Transport;
use futures::stream;
use futures::stream::StreamExt;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::Deserialize;
use std::str::FromStr;
use std::sync::Arc;
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

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    LiquidityFilter,
    "src/abi/LiquidityFilter.json"
);

pub async fn fetch_top_volume_tokens(num_results: usize, chain: Chain) -> Vec<Address> {
    let top_volume_tokens = query_birdeye(num_results, chain).await;
    top_volume_tokens
        .into_iter()
        .map(|addr| Address::from_str(&addr).unwrap())
        .collect()
}

async fn query_birdeye(num_results: usize, chain: Chain) -> Vec<String> {
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    let api_key = std::env::var("BIRDEYE_KEY").unwrap();
    headers.insert("X-API-KEY", HeaderValue::from_str(&api_key).unwrap());
    if chain == Chain::Ethereum {
        headers.insert("x-chain", HeaderValue::from_static("ethereum"));
    } else if chain == Chain::Base {
        headers.insert("x-chain", HeaderValue::from_static("base"));
    }

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

pub async fn filter_pools_by_liquidity<P, T, N>(
    provider: Arc<P>,
    pools: Vec<Pool>,
    threshold: U256,
) -> Vec<Pool>
where
    P: Provider<T, N> + 'static,
    T: Transport + Clone + 'static,
    N: Network,
{
    //let addresses: Vec<Address> = pools.iter().map(|pool| pool.address()).collect();

    // temp hack...
    let factories: Vec<Address> = vec![
        //address!("5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f"),
        //address!("1F98431c8aD98523631AE4a59f267346ea31F984"),
        address!("8909Dc15e40173Ff4699343b6eB8132c65e18eC6"),
        address!("33128a8fC17869897dcE68Ed026d694621f6FDfD"),
        address!("71524B4f93c58fcbF659783284E38825f0622859"),
        address!("c35DADB65012eC5796536bD9864eD8773aBc74C4"),
        //address!("02a84c1b3BBD7401a5f7fa98a384EBC70bB5749E"),
        //address!("0BFbCF9fa4f9C56B0F40a671Ad40E0805A091865"),
        //address!("420DD381b31aEf6683db6B902084cB0FFECe40Da")
    ];

    let pool_is_v3: Vec<bool> = vec![false, true, false, true]; //, false, true, false, true, false];

    let weth = address!("4200000000000000000000000000000000000006");
    //let weth = address!("C02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2");

    let state_data: DynSolType = DynSolType::Array(Box::new(DynSolType::Uint(256)));

    let pool_chunks: Vec<Vec<Pool>> = pools.chunks(50).map(|chunk| chunk.to_vec()).collect();

    let results = stream::iter(pool_chunks)
        .map(|chunk| {
            let provider = provider.clone();
            let state_data = state_data.clone();
            let factories = factories.clone();
            let pool_is_v3 = pool_is_v3.clone();
            let threshold = threshold.clone();
            async move {
                let pools: Vec<Address> = chunk.iter().map(|pool| pool.address()).collect();

                let data = LiquidityFilter::deploy_builder(
                    provider,
                    pools.clone(),
                    factories,
                    pool_is_v3,
                    weth,
                    threshold,
                )
                .await
                .unwrap();

                let decoded_data = state_data.abi_decode_sequence(&data).unwrap();

                let mut values = Vec::new();
                if let Some(state_data_arr) = decoded_data.as_array() {
                    for data in state_data_arr {
                        if let Some(weth_value) = data.as_uint() {
                            values.push(weth_value.0);
                        }
                    }
                }

                (pools, values)
            }
        })
        .buffer_unordered(100 * 2) // Allow some buffering for smoother operation
        .collect::<Vec<(Vec<Address>, Vec<U256>)>>()
        .await;

    let flattened: Vec<(Address, U256)> = results
        .iter()
        .flat_map(|(addresses, values)| addresses.iter().cloned().zip(values.iter().cloned()))
        .collect();

    let filtered_addresses: Vec<Address> = flattened
        .into_iter()
        .filter(|(_, value)| value > &threshold)
        .map(|(address, _)| address)
        .collect();

    let filtered_pools: Vec<Pool> = pools
        .into_iter()
        .filter(|pool| filtered_addresses.contains(&pool.address()))
        .collect();

    filtered_pools
}
