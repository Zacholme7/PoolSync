


use alloy::dyn_abi::{DynSolType, DynSolValue};
use rand::Rng;
use std::sync::Arc;
use alloy::providers::Provider;
use alloy::transports::Transport;
use alloy::network::Network;
use std::future::Future;
use alloy::sol_types::SolEvent;
use alloy::sol;
use crate::pools::{Pool, PoolFetcher, PoolType};
use alloy::primitives::Address;
use async_trait::async_trait;

use std::time::Duration;

use super::pool_structure::UniswapV2Pool;

pub const INITIAL_BACKOFF: u64 = 1000; // 1 second
pub const MAX_RETRIES: u32 = 5;

async fn build_pools<P, T, N>(
    provider: Arc<P>,
    addresses: &[Address],
    data_type: &DynSolType,
    contract_call: impl Fn(Arc<P>, Vec<Address>) -> Box<dyn Future<Output = Result<Vec<u8>, Box<dyn std::error::Error>>> + Send>,
    parse_pool: impl Fn(&[DynSolValue]) -> UniswapV2Pool,
) -> Result<Vec<Pool>, Box<dyn std::error::Error>>
where
    P: Provider<T, N> + Sync + 'static,
    T: Transport + Sync + Clone,
    N: Network,
{
    let mut retry_count = 0;
    let mut backoff = INITIAL_BACKOFF;

    loop {
        match attempt_build_pools(provider.clone(), addresses, data_type, &contract_call, &parse_pool).await {
            Ok(pools) => return Ok(pools),
            Err(e) => {
                if retry_count >= MAX_RETRIES {
                    eprintln!("Max retries reached. Error: {:?}", e);
                    return Err(e);
                }

                let jitter = rand::thread_rng().gen_range(0..=100);
                let sleep_duration = Duration::from_millis(backoff + jitter);
                tokio::time::sleep(sleep_duration).await;

                retry_count += 1;
                backoff *= 2; // Exponential backoff
            }
        }
    }
}

async fn attempt_build_pools<P, T, N>(
    provider: Arc<P>,
    addresses: &[Address],
    data_type: &DynSolType,
    contract_call: &impl Fn(Arc<P>, Vec<Address>) -> Box<dyn Future<Output = Result<Vec<u8>, Box<dyn std::error::Error>>> + Send>,
    parse_pool: &impl Fn(&[DynSolValue]) -> UniswapV2Pool,
) -> Result<Vec<Pool>, Box<dyn std::error::Error>>
where
    P: Provider<T, N> + Sync + 'static,
    T: Transport + Sync + Clone,
    N: Network,
{
    let data = contract_call(provider.clone(), addresses.to_vec()).await?;
    let decoded_data = data_type.abi_decode_sequence(&data)?;

    let mut pools = Vec::new();

    if let Some(pool_data_arr) = decoded_data.as_array() {
        for pool_data_tuple in pool_data_arr {
            if let Some(pool_data) = pool_data_tuple.as_tuple() {
                let pool = parse_pool(pool_data);
                if pool.is_valid() {
                    pools.push(pool);
                }
            }
        }
    }

    // Fetch token names (you might want to batch this for efficiency)
    for pool in &mut pools {
        let token0_contract = ERC20::new(pool.token0, provider.clone());
        if let Ok(ERC20::symbolReturn { name }) = token0_contract.symbol().call().await {
            pool.token0_name = name;
        }

        let token1_contract = ERC20::new(pool.token1, provider.clone());
        if let Ok(ERC20::symbolReturn { name }) = token1_contract.symbol().call().await {
            pool.token1_name = name;
        }
    }

    Ok(pools)
}