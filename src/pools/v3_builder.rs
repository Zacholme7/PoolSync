use crate::{
    pools::{Pool, PoolType}, rpc::{DataEvents, Rpc}
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

use super::{
    pool_structure::{TickInfo, UniswapV2Pool, UniswapV3Pool},
    PoolInfo,
};
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
    pool_type: PoolType,
) -> Vec<Pool>
where
    P: Provider<T, N> + Sync + 'static,
    T: Transport + Sync + Clone,
    N: Network,
{
    let mut retry_count = 0;
    let mut backoff = INITIAL_BACKOFF;

    loop {
        match populate_pool_data(provider.clone(), addresses.clone(), pool_type).await {
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

    let protocol = if pool_type == PoolType::UniswapV3 {
        0_u8
    } else {
        1_u8
    };
    let data =
        V3DataSync::deploy_builder(provider.clone(), pool_addresses.to_vec(), protocol).await?;
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

    let pools: Vec<Pool> = pools
        .into_iter()
        .map(|pool| Pool::new_v3(pool_type, pool))
        .collect();
    Ok(pools)
}

pub fn process_tick_data(pool: &mut UniswapV3Pool, log: Log) {
    let event_sig = log.topic0().unwrap();

    if *event_sig == DataEvents::Burn::SIGNATURE_HASH {
        process_burn(pool, log);
    } else if *event_sig == DataEvents::Mint::SIGNATURE_HASH {
        process_mint(pool, log);
    } else if *event_sig == DataEvents::Swap::SIGNATURE_HASH {
        process_swap(pool, log);
    }
}

fn process_burn(pool: &mut UniswapV3Pool, log: Log) {
    let burn_event = DataEvents::Burn::decode_log(log.as_ref(), true).unwrap();
    modify_position(
        pool,
        burn_event.tickLower,
        burn_event.tickUpper,
        -(burn_event.amount as i128)
    );
}

fn process_mint(pool: &mut UniswapV3Pool, log: Log) {
    let mint_event = DataEvents::Mint::decode_log(log.as_ref(), true).unwrap();
    modify_position(
        pool,
        mint_event.tickLower,
        mint_event.tickUpper,
        mint_event.amount as i128
    );
}

fn process_swap(pool: &mut UniswapV3Pool, log: Log) {
    let swap_event = DataEvents::Swap::decode_log(log.as_ref(), true).unwrap();
    pool.tick = swap_event.tick;
    pool.sqrt_price = swap_event.sqrtPriceX96;
    pool.liquidity = swap_event.liquidity;
}

/// Modifies a positions liquidity in the pool.
pub fn modify_position(
    pool: &mut UniswapV3Pool,
    tick_lower: i32,
    tick_upper: i32,
    liquidity_delta: i128,
) {
    //We are only using this function when a mint or burn event is emitted,
    //therefore we do not need to checkTicks as that has happened before the event is emitted
    update_position(pool, tick_lower, tick_upper, liquidity_delta);

    if liquidity_delta != 0 {
        //if the tick is between the tick lower and tick upper, update the liquidity between the ticks
        if pool.tick > tick_lower && pool.tick < tick_upper {
            pool.liquidity = if liquidity_delta < 0 {
                pool.liquidity - ((-liquidity_delta) as u128)
            } else {
                pool.liquidity + (liquidity_delta as u128)
            }
        }
    }
}

pub fn update_position(
    pool: &mut UniswapV3Pool,
    tick_lower: i32,
    tick_upper: i32,
    liquidity_delta: i128,
) {
    let mut flipped_lower = false;
    let mut flipped_upper = false;

    if liquidity_delta != 0 {
        flipped_lower = update_tick(pool, tick_lower, liquidity_delta, false);
        flipped_upper = update_tick(pool, tick_upper, liquidity_delta, true);
        if flipped_lower {
            flip_tick(pool, tick_lower, pool.tick_spacing);
        }
        if flipped_upper {
            flip_tick(pool, tick_upper, pool.tick_spacing);
        }
    }

    if liquidity_delta < 0 {
        if flipped_lower {
            pool.ticks.remove(&tick_lower);
        }

        if flipped_upper {
            pool.ticks.remove(&tick_upper);
        }
    }
}

pub fn update_tick(
    pool: &mut UniswapV3Pool,
    tick: i32,
    liquidity_delta: i128,
    upper: bool,
) -> bool {
    let info = match pool.ticks.get_mut(&tick) {
        Some(info) => info,
        None => {
            pool.ticks.insert(tick, TickInfo::default());
            pool.ticks
                .get_mut(&tick)
                .expect("Tick does not exist in ticks")
        }
    };

    let liquidity_gross_before = info.liquidity_gross;

    let liquidity_gross_after = if liquidity_delta < 0 {
        liquidity_gross_before - ((-liquidity_delta) as u128)
    } else {
        liquidity_gross_before + (liquidity_delta as u128)
    };

    // we do not need to check if liqudity_gross_after > maxLiquidity because we are only calling update tick on a burn or mint log.
    // this should already be validated when a log is
    let flipped = (liquidity_gross_after == 0) != (liquidity_gross_before == 0);

    if liquidity_gross_before == 0 {
        info.initialized = true;
    }

    info.liquidity_gross = liquidity_gross_after;

    info.liquidity_net = if upper {
        info.liquidity_net - liquidity_delta
    } else {
        info.liquidity_net + liquidity_delta
    };

    flipped
}

pub fn flip_tick(pool: &mut UniswapV3Pool, tick: i32, tick_spacing: i32) {
    let (word_pos, bit_pos) = uniswap_v3_math::tick_bitmap::position(tick / tick_spacing);
    let mask = U256::from(1) << bit_pos;

    if let Some(word) = pool.tick_bitmap.get_mut(&word_pos) {
        *word ^= mask;
    } else {
        pool.tick_bitmap.insert(word_pos, mask);
    }
}

impl From<&[DynSolValue]> for UniswapV3Pool {
    fn from(data: &[DynSolValue]) -> Self {
        Self {
            address: data[0].as_address().unwrap(),
            token0: data[1].as_address().unwrap(),
            token0_decimals: data[2].as_uint().unwrap().0.to::<u8>(),
            token1: data[3].as_address().unwrap(),
            token1_decimals: data[4].as_uint().unwrap().0.to::<u8>(),
            sqrt_price: data[6].as_uint().unwrap().0,
            tick: data[7].as_int().unwrap().0.as_i32(),
            tick_spacing: data[8].as_int().unwrap().0.as_i32(),
            fee: data[9].as_uint().unwrap().0.to::<u32>(),
            ..Default::default()
        }
    }
}
