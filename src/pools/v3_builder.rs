use alloy::{dyn_abi::{DynSolType, DynSolValue}, primitives::U128};
use rand::Rng;
use std::sync::Arc;
use alloy::providers::Provider;
use alloy::transports::Transport;
use alloy::network::Network;
use alloy::sol;
use crate::pools::{Pool, PoolType};
use alloy::primitives::Address;

use std::time::Duration;


use super::pool_structure::UniswapV3Pool;
use crate::pools::gen::ERC20;


sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    V3DataSync,
    "src/abi/V3DataSync.json"
);

pub const INITIAL_BACKOFF: u64 = 1000; // 1 second
pub const MAX_RETRIES: u32 = 5;

pub async fn build_pools<P, T, N>(
    provider: Arc<P>,
    addresses: Vec<Address>,
    pool_type: PoolType
) -> Vec<Pool> 
where
    P: Provider<T, N> + Sync + 'static,
    T: Transport + Sync + Clone,
    N: Network,
{
    let mut retry_count = 0;
    let mut backoff = INITIAL_BACKOFF;

    loop {
        match attempt_build_pools(provider.clone(), &addresses, pool_type).await {
            Ok(pools) => return pools,
            Err(e) => {
                if retry_count >= MAX_RETRIES {
                    eprintln!("Max retries reached. Error: {:?}", e);
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
    addresses: &Vec<Address>,
    pool_type: PoolType
) -> Result<Vec<Pool>, Box<dyn std::error::Error>>
where
    P: Provider<T, N> + Sync + 'static,
    T: Transport + Sync + Clone,
    N: Network,
{
    let v3_data = DynSolType::Array(Box::new(DynSolType::Tuple(vec![
        DynSolType::Address,
        DynSolType::Address,
        DynSolType::Uint(8),
        DynSolType::Address,
        DynSolType::Uint(8),
        DynSolType::Uint(128),
        DynSolType::Uint(160),
        DynSolType::Int(24),
        DynSolType::Int(24),
        DynSolType::Uint(24),
        DynSolType::Int(128),
    ])));

    let protocol = if pool_type == PoolType::UniswapV3 { 0_u8 } else { 1_u8 } ;
    let data = V3DataSync::deploy_builder(provider.clone(), addresses.to_vec(), protocol).await?;
    let decoded_data = v3_data.abi_decode_sequence(&data)?;

    let mut pools = Vec::new();

    if let Some(pool_data_arr) = decoded_data.as_array() {
        for pool_data_tuple in pool_data_arr {
            if let Some(pool_data) = pool_data_tuple.as_tuple() {
                let pool = UniswapV3Pool::from(pool_data);
                if pool.is_valid() {
                    pools.push(pool);
                }
            }
        }
    }

    // Fetch token names (you might want to batch this for efficiency)
    for pool in &mut pools {
        let token0_contract = ERC20::new(pool.token0, provider.clone());
        if let Ok(ERC20::symbolReturn { _0: name }) = token0_contract.symbol().call().await {
            pool.token0_name = name;
        }

        let token1_contract = ERC20::new(pool.token1, provider.clone());
        if let Ok(ERC20::symbolReturn { _0: name }) = token1_contract.symbol().call().await {
            pool.token1_name = name;
        }
    }

    let pools = pools.into_iter().map(|pool| Pool::new_v3(pool_type, pool)).collect();

    Ok(pools)
}

impl From<&[DynSolValue]> for UniswapV3Pool {
    fn from(data: &[DynSolValue]) -> Self {
        Self {
            address: data[0].as_address().unwrap(),
            token0: data[1].as_address().unwrap(),
            token0_decimals: data[2].as_uint().unwrap().0.to::<u8>(),
            token1: data[3].as_address().unwrap(),
            token1_decimals: data[4].as_uint().unwrap().0.to::<u8>(),
            liquidity: data[5].as_uint().unwrap().0.to::<U128>(),
            sqrt_price: data[6].as_uint().unwrap().0,
            tick: data[7].as_int().unwrap().0.as_i32(),
            tick_spacing: data[8].as_int().unwrap().0.as_i32(),
            fee: data[9].as_uint().unwrap().0.to::<u32>(),
            ..Default::default()
        }
    }
}
