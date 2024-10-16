//use crate::{
//    pools::{Pool, PoolType}, rpc::{DataEvents, PancakeSwap, Rpc}
//}; //, snapshot::{v3_tick_snapshot, v3_tickbitmap_snapshot}};
use crate::PoolInfo;
use alloy::dyn_abi::DynSolType;
use alloy::network::Network;
use alloy::primitives::{address, Address};
use alloy::providers::Provider;
use alloy::transports::Transport;
use anyhow::Result;
use rand::Rng;
use std::sync::Arc;

use std::time::Duration;
use uniswap_v3_math;

use super::gen::{
    BalancerV2DataSync, MaverickDataSync, PancakeSwapDataSync, SlipStreamDataSync,
    TriCurveDataSync, TwoCurveDataSync, V2DataSync, V3DataSync,
};

use crate::pools::gen::ERC20;
use crate::pools::gen::{AerodromePool, AerodromeV2Factory};
use crate::pools::{Pool, PoolType};

pub const INITIAL_BACKOFF: u64 = 1000; // 1 second
pub const MAX_RETRIES: u32 = 5;

pub async fn build_pools<P, T, N>(
    provider: Arc<P>,
    addresses: Vec<Address>,
    pool_type: PoolType,
    data: DynSolType,
) -> Result<Vec<Pool>>
where
    P: Provider<T, N> + Sync + 'static,
    T: Transport + Sync + Clone,
    N: Network,
{
    let mut retry_count = 0;
    let mut backoff = INITIAL_BACKOFF;

    loop {
        match populate_pool_data(provider.clone(), addresses.clone(), pool_type, data.clone()).await
        {
            Ok(pools) => {
                drop(provider);
                return Ok(pools);
            }
            Err(e) => {
                if retry_count >= MAX_RETRIES {
                    eprintln!("Max retries reached. Error: {:?}", e);
                    drop(provider);
                    return Ok(Vec::new());
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
) -> Result<Vec<Pool>>
where
    P: Provider<T, N> + Sync + 'static,
    T: Transport + Sync + Clone,
    N: Network,
{
    let pool_data = match pool_type {
        PoolType::UniswapV2
        | PoolType::SushiSwapV2
        | PoolType::PancakeSwapV2
        | PoolType::BaseSwapV2
        | PoolType::Aerodrome
        | PoolType::AlienBaseV2
        | PoolType::SwapBasedV2
        | PoolType::DackieSwapV2 => {
            V2DataSync::deploy_builder(provider.clone(), pool_addresses.to_vec()).await?
        }
        PoolType::MaverickV1 | PoolType::MaverickV2 => {
            MaverickDataSync::deploy_builder(provider.clone(), pool_addresses.to_vec()).await?
        }
        PoolType::PancakeSwapV3 | PoolType::SwapBasedV3 | PoolType::DackieSwapV3 => {
            PancakeSwapDataSync::deploy_builder(provider.clone(), pool_addresses.to_vec()).await?
        }
        PoolType::UniswapV3
        | PoolType::SushiSwapV3
        | PoolType::BaseSwapV3
        | PoolType::AlienBaseV3 => {
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
        PoolType::CurveTriCrypto => {
            TriCurveDataSync::deploy_builder(provider.clone(), pool_addresses.to_vec()).await?
        }
        _ => panic!("Invalid pool type"),
    };

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

    // fill in missing info for the pool, this is more impl specific details. fetched by the full node, okay to not batch
    for pool in &mut pools {
        let token0_contract = ERC20::new(pool.token0_address(), provider.clone());
        if let Ok(ERC20::symbolReturn { _0: name }) = token0_contract.symbol().call().await {
            Pool::update_token0_name(pool, name);
        }

        let token1_contract = ERC20::new(pool.token1_address(), provider.clone());
        if let Ok(ERC20::symbolReturn { _0: name }) = token1_contract.symbol().call().await {
            Pool::update_token1_name(pool, name);
        }

        // If the pool is balancer, update names for the other tokens
        if pool_type == PoolType::BalancerV2 {
            let mut pool = pool.get_balancer_mut().unwrap();
            for token in &pool.additional_tokens {
                let token_contract = ERC20::new(*token, provider.clone());
                if let Ok(ERC20::symbolReturn { _0: name }) = token_contract.symbol().call().await {
                    pool.additional_token_names.push(name);
                }
            }
        }

        // if the pool is curve, update name for the third token
        if pool_type == PoolType::CurveTriCrypto {
            let pool = pool.get_curve_tri_mut().unwrap();
            let token_contract = ERC20::new(pool.token2, provider.clone());
            if let Ok(ERC20::symbolReturn { _0: name }) = token_contract.symbol().call().await {
                pool.token2_name = name;
            }
        }

        // if the pool is aerodrome, update the fee and if it is stable or not
        if pool_type == PoolType::Aerodrome {
            let factory = address!("420DD381b31aEf6683db6B902084cB0FFECe40Da");
            let pool = pool.get_v2_mut().unwrap();
            // get if it is stable or not
            let pool_contract = AerodromePool::new(pool.address, provider.clone());
            let AerodromePool::stableReturn { _0: stable } =
                pool_contract.stable().call().await.unwrap();
            pool.stable = Some(stable);

            let factory_contract = AerodromeV2Factory::new(factory, provider.clone());
            let AerodromeV2Factory::getFeeReturn { _0: fee } = factory_contract
                .getFee(pool.address, stable)
                .call()
                .await
                .unwrap();
            pool.fee = Some(fee);
        }
    }

    Ok(pools)
}
