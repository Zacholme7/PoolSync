//! PoolSync Core Implementation
//!
//! This module contains the core functionality for synchronizing pools across different
//! blockchain networks and protocols. It includes the main `PoolSync` struct and its
//! associated methods for configuring and executing the synchronization process.
//!
use alloy::dyn_abi::{DynSolType, DynSolValue};
use alloy::network::Network;
use futures::stream;

use alloy::providers::Provider;
use alloy::transports::Transport;
use std::collections::HashMap;
use std::sync::Arc;
use futures::stream::StreamExt;
use alloy::sol;
use futures::future::join_all;
use alloy::primitives::{Address, U256};

use crate::errors::PoolSyncError;
use crate::PoolInfo;
use crate::Pool;
use crate::pools::pool_structure::{UniswapV3Pool, UniswapV2Pool};

// local reserve updates
#[derive(Debug, Default, Clone)]
pub struct UniswapV2PoolState {
    pub address: Address,
    pub reserve0: u128,
    pub reserve1: u128,
}

#[derive(Debug, Default, Clone)]
pub struct UniswapV3PoolState {
    pub address: Address,
    pub liquidity: u128, 
    pub sqrt_price: U256,
    pub tick: i32,
    pub fee: u32,
    pub tick_spacing: i32,
    pub tick_bitmap: HashMap<i16, U256>,
    pub ticks: HashMap<i32, TickInfo>,
}


#[derive(Debug, Default, Clone)]
pub struct TickInfo {
    pub liquidity_net: i128,
    pub initialized: bool,
}

pub struct V3PriceState {
    address: Address,
    liquidity: u128,
    sqrt_price: U256,
    tick: i32,
    fee: u32,
    tick_spacing: i32,
}

#[derive(Debug, Default, Clone)]
pub struct V3BitmapState {
    pub address: Address,
    pub tick_bitmaps: Vec<U256>,
    pub word_positions: Vec<i16>,
    pub word_to_map: HashMap<i16, U256>,
}

#[derive(Debug, Default, Clone)]
pub struct V3TickState {
    pub address: Address,
    pub initialized: bool,
    pub tick: i32,
    pub liqudity_net: i128,
}

/// Get a snapshot of the most recent reserves for a list of pools, input is the pool addresses
pub async fn v2_pool_snapshot<P, T, N>(pool_addresses: Vec<Address>, provider: Arc<P>) -> Result<Vec<UniswapV2PoolState>, PoolSyncError>
where
    P: Provider<T, N> + 'static,
    T: Transport + Clone + 'static,
    N: Network,
{
    // snap
    sol!(
        #[derive(Debug)]
        #[sol(rpc)]
        V2ReserveUpdate,
        "src/abi/V2ReserveUpdate.json"
    );

    // Map all the addresses into chunks the contract can handle
    let addr_chunks: Vec<Vec<Address>> =
        pool_addresses.chunks(20).map(|chunk| chunk.to_vec()).collect();

    // create out futurs and get all of the results
    let results = stream::iter(addr_chunks).map(|chunk| {
        let provider = provider.clone();
        async move {
            let reserve_data: DynSolType = DynSolType::Array(Box::new(DynSolType::Tuple(vec![
                DynSolType::Address,
                DynSolType::Uint(112),
                DynSolType::Uint(112),
            ])));
            let data = V2ReserveUpdate::deploy_builder(provider.clone(), chunk.clone()).await.unwrap();
            let decoded_data = reserve_data.abi_decode_sequence(&data).unwrap();
            let mut updated_reserves = Vec::new();
            if let Some(reserve_data_arr) = decoded_data.as_array() {
                for reserve_data_tuple in reserve_data_arr {
                    if let Some(reserve_data) = reserve_data_tuple.as_tuple() {
                        let decoded_reserve = UniswapV2PoolState::from(reserve_data);
                        updated_reserves.push(decoded_reserve);
                    }
                }
            }
            return updated_reserves;
        }
    }).buffer_unordered(100 as usize * 2) // Allow some buffering for smoother operation
    .collect::<Vec<Vec<UniswapV2PoolState>>>()
    .await;

    // map into single vector
    let results: Vec<UniswapV2PoolState> = results.into_iter().flatten().collect();
    Ok(results)
}


pub async fn v3_pool_snapshot<P, T, N>(pools: &Vec<Address>, provider: Arc<P>) -> Result<Vec<UniswapV3PoolState>, PoolSyncError> 
where 
    P: Provider<T, N> + 'static,
    T: Transport + Clone + 'static,
    N: Network,
{

    let mut results: Vec<UniswapV3PoolState> = Vec::new();
    // fetch all the state
    let price_state = v3_price_snapshot(pools, provider.clone()).await?;
    let bitmap_state = v3_bitmap_snapshot(pools, provider.clone()).await?;
    let pool_info: Vec<(Address, i32, i32)> = price_state.iter().map(|state| (state.address, state.tick, state.tick_spacing)).collect();
    let tick_state = v3_tick_snapshot(pool_info, provider).await?;


    // maps for the state
    let price_map: HashMap<Address, V3PriceState> = price_state.into_iter().map(|state| (state.address, state)).collect();
    let bitmap_map: HashMap<Address, V3BitmapState> = bitmap_state.into_iter().map(|state| (state.address, state)).collect();
    let tick_map: HashMap<Address, HashMap<i32, TickInfo>> = tick_state.into_iter().map(|state| (state.0, state.1)).collect();


    for address in pools {
        if let (Some(price), Some(bitmap), Some(ticks)) = (price_map.get(address), bitmap_map.get(address), tick_map.get(address)) {
            results.push(UniswapV3PoolState {
                address: address.clone(),
                liquidity: price.liquidity,
                sqrt_price: price.sqrt_price,
                tick: price.tick,
                tick_spacing: price.tick_spacing,
                fee: price.fee,
                tick_bitmap: bitmap.word_to_map.clone(),
                ticks: ticks.clone(),
            })
        }
    }

    Ok(results)
}


async fn v3_price_snapshot<P, T, N>(addresses: &Vec<Address>, provider: Arc<P>) -> Result<Vec<V3PriceState>, PoolSyncError>
where
    P: Provider<T, N> + 'static,
    T: Transport + Clone + 'static,
    N: Network,
{
    sol!(
        #[derive(Debug)]
        #[sol(rpc)]
        V3StateUpdate,
        "src/abi/V3StateUpdate.json"
    );

    let state_data: DynSolType = DynSolType::Array(Box::new(DynSolType::Tuple(vec![
        DynSolType::Address,
        DynSolType::Uint(128),
        DynSolType::Uint(160),
        DynSolType::Int(24),
        DynSolType::Uint(32),
        DynSolType::Int(24),
    ])));


    let address_chunks: Vec<Vec<Address>> = addresses.chunks(20).map(|chunk| {
        chunk.to_vec()
    }).collect();

    // update general pool state
    let results = stream::iter(address_chunks).map(|chunk| {
        let provider = provider.clone();
        let state_data = state_data.clone();
        async move {
            let data = V3StateUpdate::deploy_builder(provider.clone(), chunk.clone()).await.unwrap();
            let decoded_data = state_data.abi_decode_sequence(&data).unwrap();
            let mut updated_states = Vec::new();
            if let Some(state_data_arr) = decoded_data.as_array() {
                for state_data_tuple in state_data_arr {
                    if let Some(state_data) = state_data_tuple.as_tuple() {
                        let decoded_state = V3PriceState::from(state_data);
                        updated_states.push(decoded_state);
                    }
                }
            }
            return updated_states;
        }
    }).buffer_unordered(100 as usize * 2) // Allow some buffering for smoother operation
        .collect::<Vec<Vec<V3PriceState>>>()
        .await;

    let results: Vec<V3PriceState> = results.into_iter().flatten().collect();
    Ok(results)
}







pub async fn v3_bitmap_snapshot<P, T, N>(addresses: &Vec<Address>, provider: Arc<P>) -> Result<Vec<V3BitmapState>, PoolSyncError>
where
    P: Provider<T, N> + 'static,
    T: Transport + Clone + 'static,
    N: Network,
{
    sol!(
        #[derive(Debug)]
        #[sol(rpc)]
        V3TickBitmapUpdate,
        "src/abi/V3TickBitmapUpdate.json"
    );

    let state_data: DynSolType = DynSolType::Array(Box::new(DynSolType::Tuple(vec![
        DynSolType::Address,
        DynSolType::Array(Box::new(DynSolType::Uint(256))),
        DynSolType::Array(Box::new(DynSolType::Int(16))),
    ])));

    let address_chunks: Vec<Vec<Address>> = addresses.chunks(10).map(|chunk| {
        chunk.to_vec()
    }).collect();

    let results = stream::iter(address_chunks).map(|chunk| {
        let provider = provider.clone();
        let state_data = state_data.clone();
        async move {
            let data = V3TickBitmapUpdate::deploy_builder(provider.clone(), chunk.clone()).await.unwrap();
            let decoded_data = state_data.abi_decode_sequence(&data).unwrap();

            let mut updated_bitmaps: Vec<V3BitmapState> = Vec::new();
            if let Some(state_data_arr) = decoded_data.as_array() {
                for state_data_tuple in state_data_arr {
                    if let Some(state_data) = state_data_tuple.as_tuple() {
                        let decoded_state = V3BitmapState::from(state_data);
                        updated_bitmaps.push(decoded_state);
                    }
                }
            }

            updated_bitmaps
        }
    }).buffer_unordered(100 * 2) // Allow some buffering for smoother operation
        .collect::<Vec<Vec<V3BitmapState>>>()
        .await;
    let mut results: Vec<V3BitmapState> = results.into_iter().flatten().collect();

    // TODO CHECK THIS
    for result in &mut results {
        result.word_to_map = result.word_positions.iter()
            .zip(result.tick_bitmaps.iter())
            .map(|(word_position, bitmap)| {
                (*word_position, *bitmap)
            }).collect();
    }

    Ok(results)
}


pub async fn v3_tick_snapshot<P, T, N>(pool_info: Vec<(Address, i32, i32)>, provider: Arc<P>) -> Result<Vec<(Address, HashMap<i32, TickInfo>)>, PoolSyncError>
where
    P: Provider<T, N> + 'static,
    T: Transport + Clone + 'static,
    N: Network,
{

    sol!(
        #[derive(Debug)]
        #[sol(rpc)]
        V3TickUpdate,
        "src/abi/V3TickUpdate.json"
    );


    let results = stream::iter(pool_info).map(|(pool, tick, tick_spacing)| {
        let provider = provider.clone();
        async move {
            let constructor_return = DynSolType::Tuple(vec![
                DynSolType::Array(Box::new(DynSolType::Tuple(vec![
                    DynSolType::Bool,
                    DynSolType::Int(24),
                    DynSolType::Int(128),
                ]))),
                DynSolType::Uint(32),
            ]);
            // fetch 
            let zero_to_one_tick_data = V3TickUpdate::deploy_builder(
                provider.clone(),
                pool,
                true,
                tick,
                15,
                tick_spacing,
            ).await.unwrap();
            let zero_to_one_tick_decoded = constructor_return.abi_decode_sequence(&zero_to_one_tick_data).unwrap();

            let one_to_zero_tick_data = V3TickUpdate::deploy_builder(
                provider.clone(),
                pool,
                false,
                tick,
                15,
                tick_spacing
            ).await.unwrap();
            let one_to_zero_tick_decoded = constructor_return.abi_decode_sequence(&one_to_zero_tick_data).unwrap();

            let decoded_data = vec![zero_to_one_tick_decoded, one_to_zero_tick_decoded];

            let mut updated_ticks = Vec::new();
            for data in decoded_data {
                if let Some(state_data_tuple) = data.as_tuple() {
                    if let Some(state_data_arr) = state_data_tuple[0].as_array() {
                        for tokens in state_data_arr {
                            if let Some(tick_data_tuple) = tokens.as_tuple() {
                                let mut decoded_state = V3TickState::from(tick_data_tuple);
                                decoded_state.address = pool;
                                updated_ticks.push(decoded_state);
                            }
                        }
                    }
                }

            }
            updated_ticks
        }
    }).buffer_unordered(100 * 2) // Allow some buffering for smoother operation
        .collect::<Vec<Vec<V3TickState>>>()
        .await;

    let results: Vec<(Address, HashMap<i32, TickInfo>)> = results.into_iter().map(|tick_states| {
        let mut tick_map = HashMap::new();
        let address = tick_states[0].address.clone();
        for tick_state in tick_states {
            tick_map.insert(tick_state.tick, TickInfo {
                liquidity_net: tick_state.liqudity_net,
                initialized: tick_state.initialized,
            });
        }
        (address, tick_map)
    }).collect();

    Ok(results)
}


// Data parsers
impl From<&[DynSolValue]> for V3TickState {
    fn from(data: &[DynSolValue]) -> Self {
        Self {
            initialized: data[0].as_bool().unwrap(),
            tick: data[1].as_int().unwrap().0.as_i32(),
            liqudity_net: data[2].as_int().unwrap().0.try_into().unwrap(),
            ..Default::default()
        }
    }
}

impl From<&[DynSolValue]> for V3BitmapState {
    fn from(data: &[DynSolValue]) -> Self {
        Self {
            address: data[0].as_address().unwrap(),
            tick_bitmaps: data[1].as_array().unwrap().iter()
                .map(|value| value.as_uint().unwrap().0)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
            word_positions: data[2].as_array().unwrap().iter()
                .map(|value| value.as_int().unwrap().0.as_i16())
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
            ..Default::default()
        }
    }
}

impl From<&[DynSolValue]> for V3PriceState {
    fn from(data: &[DynSolValue]) -> Self {
        Self {
            address: data[0].as_address().unwrap(),
            liquidity: data[1].as_uint().unwrap().0.to::<u128>(),
            sqrt_price: data[2].as_uint().unwrap().0,
            tick: data[3].as_int().unwrap().0.as_i32(),
            fee: data[4].as_uint().unwrap().0.try_into().unwrap(),
            tick_spacing: data[5].as_int().unwrap().0.as_i32(),
        }
    }
}

impl From<&[DynSolValue]> for UniswapV2PoolState {
    fn from(data: &[DynSolValue]) -> Self {
        Self {
            address: data[0].as_address().unwrap(),
            reserve0: data[1].as_uint().unwrap().0.to::<u128>(),
            reserve1: data[2].as_uint().unwrap().0.to::<u128>(),
        }
    }
}