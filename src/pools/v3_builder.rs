use alloy::{dyn_abi::{DynSolType, DynSolValue}, primitives::U128};
use rand::Rng;
use std::sync::Arc;
use alloy::providers::Provider;
use alloy::transports::Transport;
use alloy::network::Network;
use alloy::sol;
use crate::{pools::{Pool, PoolType}};//, snapshot::{v3_tick_snapshot, v3_tickbitmap_snapshot}};
use alloy::primitives::Address;

use std::time::Duration;


use super::{pool_structure::{TickInfo, UniswapV3Pool}, PoolInfo};
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
    // get initial pools populated with data
    let mut pools = populate_pool_data(provider.clone(), addresses.to_vec(), pool_type).await?;

    // populate pools with bitmpaps

    //populate_tick_bitmap(provider.clone(), &mut pools).await?;
    //populate_ticks(provider.clone(), &mut pools).await?;


    Ok(pools)
}


async fn populate_pool_data<P, T, N>(provider: Arc<P>, pool_addresses: Vec<Address>, pool_type: PoolType) -> Result<Vec<Pool>, Box<dyn std::error::Error>>
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
    let data = V3DataSync::deploy_builder(provider.clone(), pool_addresses.to_vec(), protocol).await?;
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

    // convert pools to generic pool
    let pools = pools.into_iter().map(|pool| Pool::new_v3(pool_type, pool)).collect();

    Ok(pools)
}


/* 
pub async fn populate_tick_bitmap<P, T, N>(provider: Arc<P>, pools: &mut Vec<Pool>) -> Result<(), Box<dyn std::error::Error>>
where
    P: Provider<T, N> + Sync + 'static,
    T: Transport + Sync + Clone,
    N: Network,
{
    let tick_bitmaps = v3_tickbitmap_snapshot(pools, provider).await?;
    for bitmap_snapshot in tick_bitmaps {
        let pool = pools.iter_mut().find(|pool| pool.address() == bitmap_snapshot.address).unwrap();
        match pool {
            Pool::UniswapV3(ref mut p) | Pool::SushiSwapV3(ref mut p) | Pool::PancakeSwapV3(ref mut p) => {
                p.tick_bitmap = bitmap_snapshot.word_to_map;
            }
            _ => panic!("will never reach here")
        }
    }
    Ok(())
}

pub async fn populate_ticks<P, T, N>(provider: Arc<P>, pools: &mut Vec<Pool>) -> Result<(), Box<dyn std::error::Error>>
where
    P: Provider<T, N> + Sync + 'static,
    T: Transport + Sync + Clone,
    N: Network,
{

    let ticks = v3_tick_snapshot(pools, provider).await?;
    for tick_snapshot in ticks {
        let pool = pools.iter_mut().find(|pool| pool.address() == tick_snapshot[0].address).unwrap();
        match pool {
            Pool::UniswapV3(ref mut p) | Pool::SushiSwapV3(ref mut p) | Pool::PancakeSwapV3(ref mut p) => {
                for snapshot in tick_snapshot {
                    p.ticks.insert(snapshot.tick, TickInfo {
                        liquidity_net: snapshot.liqudity_net,
                        initialized: snapshot.initialized,
                    });
                }
            }
            _ => panic!("will never reach here")
        }
    }
    Ok(())
}
    */


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
