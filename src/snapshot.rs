//! PoolSync Core Implementation
//!
//! This module contains the core functionality for synchronizing pools across different
//! blockchain networks and protocols. It includes the main `PoolSync` struct and its
//! associated methods for configuring and executing the synchronization process.
//!
use alloy::dyn_abi::{DynSolType, DynSolValue};
use alloy::network::Network;
use alloy::signers::k256::elliptic_curve::bigint::modular::montgomery_reduction;
use futures::stream;

use alloy::providers::Provider;
use alloy::transports::Transport;
use reqwest::Url;
use std::str::FromStr;
use std::collections::HashMap;
use std::sync::Arc;
use futures::stream::StreamExt;
use alloy::sol;
use alloy::primitives::{Address, U256};
use alloy::providers::ProviderBuilder;

use crate::errors::PoolSyncError;
// local reserve updates
#[derive(Debug)]
pub struct V2ReserveSnapshot {
    pub address: Address,
    pub reserve0: u128,
    pub reserve1: u128,
}

#[derive(Debug)]
pub struct V3StateSnapshot {
    pub address: Address,
    pub liquidity: u128,
    pub sqrt_price: U256,
    pub tick: i32,
}

pub struct V3TickSnapshot {
    pub initialized: bool,
    pub tick: i32,
    pub liqudity_net: i128,
}


/// Get a snapshot of the most recent reserves for a list of pools
pub async fn v2_pool_snapshot<P, T, N>(pool_addresses: Vec<Address>, provider: Arc<P>) -> Result<Vec<V2ReserveSnapshot>, PoolSyncError>
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
        pool_addresses.chunks(40).map(|chunk| chunk.to_vec()).collect();

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
                        let decoded_reserve = V2ReserveSnapshot::from(reserve_data);
                        updated_reserves.push(decoded_reserve);
                    }
                }
            }
            return updated_reserves;
        }
    }).buffer_unordered(100 as usize * 2) // Allow some buffering for smoother operation
    .collect::<Vec<Vec<V2ReserveSnapshot>>>()
    .await;

    // map into single vector
    let results: Vec<V2ReserveSnapshot> = results.into_iter().flatten().collect();
    Ok(results)
}

pub async fn v3_pool_snapshot<P, T, N>(pool_addresses: Vec<Address>, provider: Arc<P>) -> Result<Vec<V3StateSnapshot>, PoolSyncError>
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

    // Map all the addresses into chunks the contract can handle
    let addr_chunks: Vec<Vec<Address>> =
        pool_addresses.chunks(40).map(|chunk| chunk.to_vec()).collect();

    // update general pool state
    let results = stream::iter(addr_chunks).map(|chunk| {
        let provider = provider.clone();
        async move {
            let state_data: DynSolType = DynSolType::Array(Box::new(DynSolType::Tuple(vec![
                DynSolType::Address,
                DynSolType::Uint(128),
                DynSolType::Uint(160),
                DynSolType::Int(24),
            ])));

            let data = V3StateUpdate::deploy_builder(provider.clone(), chunk.clone()).await.unwrap();
            let decoded_data = state_data.abi_decode_sequence(&data).unwrap();
            let mut updated_states = Vec::new();
            if let Some(state_data_arr) = decoded_data.as_array() {
                for state_data_tuple in state_data_arr {
                    if let Some(state_data) = state_data_tuple.as_tuple() {
                        let decoded_state = V3StateSnapshot::from(state_data);
                        updated_states.push(decoded_state);
                    }
                }
            }
            return updated_states;
        }
    }).buffer_unordered(100 as usize * 2) // Allow some buffering for smoother operation
        .collect::<Vec<Vec<V3StateSnapshot>>>()
        .await;

    let results: Vec<V3StateSnapshot> = results.into_iter().flatten().collect();
    Ok(results)
}



pub async fn v3_tickbitmap_snapshot<P, T, N>(pool_addresses: Vec<Address>, provider: Arc<P>) -> Result<Vec<V3TickBitmapSnapshot>, PoolSyncError>
where
    P: Provider<T, N> + 'static,
    T: Transport + Clone + 'static,
    N: Network,
{
    todo!()
}

/* 

pub async fn v3_tick_snapshot<P, T, N>(pool_addresses: Vec<Address>, provider: Arc<P>) -> Result<Vec<V3TickSnapshot>, PoolSyncError>
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


    let results = stream::iter(pool_addresses).map(|pool_address| {
        let provider = provider.clone();
        async move {
            let tick_data: DynSolType = DynSolType::Array(Box::new(DynSolType::Tuple(vec![
                DynSolType::Bool,
                DynSolType::Int(24),
                DynSolType::Int(24),
                DynSolType::Int(128),
            ])));

            let data = V3TickUpdate::deploy_builder(
                provider.clone(), 
                pool_address

            ).await.unwrap();
        }

    });
    todo!()

}

*/

// Data parsers


impl From<&[DynSolValue]> for V3StateSnapshot {
    fn from(data: &[DynSolValue]) -> Self {
        Self {
            address: data[0].as_address().unwrap(),
            liquidity: data[1].as_uint().unwrap().0.to::<u128>(),
            sqrt_price: data[2].as_uint().unwrap().0,
            tick: data[3].as_int().unwrap().0.as_i32(),
        }
    }
}



impl From<&[DynSolValue]> for V2ReserveSnapshot {
    fn from(data: &[DynSolValue]) -> Self {
        Self {
            address: data[0].as_address().unwrap(),
            reserve0: data[1].as_uint().unwrap().0.to::<u128>(),
            reserve1: data[2].as_uint().unwrap().0.to::<u128>(),
        }
    }
}