//use crate::{
//    pools::{Pool, PoolType}, rpc::{DataEvents, PancakeSwap, Rpc}
//}; //, snapshot::{v3_tick_snapshot, v3_tickbitmap_snapshot}};
use alloy::network::Network;
use alloy::primitives::Address;
use alloy::providers::Provider;
use alloy::transports::Transport;
use alloy::dyn_abi::DynSolType;
use rand::Rng;
use std::sync::Arc;
use crate::PoolInfo;

use std::time::Duration;
use uniswap_v3_math;

use super::gen::{
    V2DataSync, 
    V3DataSync, 
    PancakeSwapDataSync, 
    MaverickDataSync,
    SlipStreamDataSync,
    BalancerV2DataSync,
    TwoCurveDataSync
};

use crate::pools::gen::ERC20;
use crate::pools::{Pool, PoolType};
use crate::rpc::PancakeSwap;

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
    let pool_data = match pool_type {
        PoolType::UniswapV2 | PoolType::SushiSwapV2 | 
        PoolType::PancakeSwapV2 | PoolType::BaseSwapV2 |
        PoolType::Aerodrome => {
            V2DataSync::deploy_builder(provider.clone(), pool_addresses.to_vec()).await?
        }
        PoolType::MaverickV1 | PoolType::MaverickV2 => {
            MaverickDataSync::deploy_builder(provider.clone(), pool_addresses.to_vec()).await?
        } 
        PoolType::PancakeSwapV3 => {
            PancakeSwapDataSync::deploy_builder(provider.clone(), pool_addresses.to_vec()).await?
        }
        PoolType::UniswapV3 | PoolType::SushiSwapV3 | 
        PoolType::BaseSwapV3 | PoolType::AlienBase => {
            V3DataSync::deploy_builder(provider.clone(), pool_addresses.to_vec()).await?
        }
        PoolType::Slipstream => {
            SlipStreamDataSync::deploy_builder(provider.clone(), pool_addresses.to_vec()).await?
        }
        PoolType::BalancerV2 => {
            BalancerV2DataSync::deploy_builder(provider.clone(), pool_addresses.to_vec()).await?
        }
        PoolType::CurveTwoCrypto => {
            TwoCurveDataSync::deploy_builder(provider.clone(), pool_addresses.to_vec()).await?
        }
        _=> panic!("Invalid pool type")
    };

    //println!("Raw pool data: {:?}", hex::encode(&pool_data));
    let decoded_data = data.abi_decode_sequence(&pool_data)?;
    let mut pools = Vec::new();

    if let Some(pool_data_arr) = decoded_data.as_array() {
        for pool_data_tuple in pool_data_arr {
            if let Some(pool_data) = pool_data_tuple.as_tuple() {
                let pool = pool_type.build_pool(pool_data);
                if pool.is_valid() {
                    pools.push(pool);
                }
            }
        }
    }

    // update the token names on the pools
    for pool in &mut pools {
        let token0_contract = ERC20::new(pool.token0_address(), provider.clone());
        if let Ok(ERC20::symbolReturn { _0: name }) = token0_contract.symbol().call().await {
            Pool::update_token0_name(pool, name);
        }

        let token1_contract = ERC20::new(pool.token1_address(), provider.clone());
        if let Ok(ERC20::symbolReturn { _0: name }) = token1_contract.symbol().call().await {
            Pool::update_token1_name(pool, name);
        }

        if pool_type == PoolType::BalancerV2 {
            let mut pool = pool.get_balancer_mut().unwrap();
            for token in &pool.additional_tokens {
                let token_contract = ERC20::new(*token, provider.clone());
                if let Ok(ERC20::symbolReturn { _0: name }) = token_contract.symbol().call().await {
                    pool.additional_token_names.push(name);
                }
            }
        }
    }

    Ok(pools)
}