use crate::{
    pools::{Pool, PoolType}, rpc::{DataEvents, PancakeSwap, Rpc}
}; //, snapshot::{v3_tick_snapshot, v3_tickbitmap_snapshot}};
use alloy::network::Network;
use alloy::primitives::Address;
use alloy::primitives::U256;
use alloy::providers::Provider;
use alloy::sol;
use alloy::transports::Transport;
use alloy::{
    dyn_abi::{DynSolType, DynSolValue},
    primitives::U128,
    rpc::types::Log,
};
use alloy_sol_types::SolEvent;
use rand::Rng;
use std::sync::Arc;

use std::time::Duration;
use uniswap_v3_math;

use super::gen::{V2DataSync, V3DataSync};

use crate::pools::gen::ERC20;

pub const INITIAL_BACKOFF: u64 = 1000; // 1 second
pub const MAX_RETRIES: u32 = 5;

pub async fn build_pools<P, T, N>(
    provider: Arc<P>,
    addresses: Vec<Address>,
    pool_type: PoolType,
    data: DynSolType
) -> Vec<Pool>
where
    P: Provider<T, N> + Sync + 'static,
    T: Transport + Sync + Clone,
    N: Network,
{
    let mut retry_count = 0;
    let mut backoff = INITIAL_BACKOFF;

    loop {
        match populate_pool_data(provider.clone(), addresses.clone(), pool_type, data.clone()).await {
            Ok(pools) => {
                drop(provider);
                return pools;
            }
            Err(e) => {
                if retry_count >= MAX_RETRIES {
                    eprintln!("Max retries reached. Error: {:?}", e);
                    drop(provider);
                    return Vec::new();
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

async fn populate_pool_data<P, T, N>(
    provider: Arc<P>,
    pool_addresses: Vec<Address>,
    pool_type: PoolType,
    data: DynSolType,
) -> Result<Vec<Pool>, Box<dyn std::error::Error>>
where
    P: Provider<T, N> + Sync + 'static,
    T: Transport + Sync + Clone,
    N: Network,
{
    let pool_data = if pool_type.is_v2() {
        V2DataSync::deploy_builder(provider.clone(), pool_addresses.to_vec()).await?
    } else if pool_type.is_v3() {
        V3DataSync::deploy_builder(provider.clone(), pool_addresses.to_vec()).await?
    } else {
        todo!()
    };

    let decoded_data = data.abi_decode_sequence(&pool_data)?;
    let mut pools = Vec::new();

    if let Some(pool_data_arr) = decoded_data.as_array() {
        for pool_data_tuple in pool_data_arr {
            if let Some(pool_data) = pool_data_tuple.as_tuple() {
                let pool = construct_pool_from_data(pool_data, pool_type);
                //if pool.is_valid() {
                //    pools.push(pool);
                //}
            }
        }
    }

    Ok(pools)
}

fn construct_pool_from_data(pool_data: &[DynSolValue], pool_type: PoolType) -> Pool {
    todo!()
}

    /* 
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
    */