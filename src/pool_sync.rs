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
use std::collections::HashMap;
use std::sync::Arc;
use futures::stream::StreamExt;
use alloy::sol;
use alloy::primitives::Address;

use crate::builder::PoolSyncBuilder;
use crate::cache::{read_cache_file, write_cache_file, PoolCache};
use crate::chain::Chain;
use crate::errors::*;
use crate::pools::*;
use crate::rpc::Rpc;

/// The maximum number of retries for a failed query
const MAX_RETRIES: u32 = 5;

// local reserve updates
#[derive(Debug)]
pub struct V2ReserveSnapshot {
    address: Address,
    reserve0: u128,
    reserve1: u128,
}

/// The main struct for pool synchronization
pub struct PoolSync {
    /// Map of pool types to their fetcher implementations
    pub fetchers: HashMap<PoolType, Arc<dyn PoolFetcher>>,
    /// The chain to sync on
    pub chain: Chain,
    /// The rate limit of the rpc
    pub rate_limit: u64,
}

impl PoolSync {
    /// Construct a new builder to configure sync parameters
    pub fn builder() -> PoolSyncBuilder {
        PoolSyncBuilder::default()
    }

    /// Synchronizes all added pools for the specified chain
    pub async fn sync_pools<P, T, N>(
        &self,
        provider: Arc<P>,
    ) -> Result<Vec<Pool>, PoolSyncError>
    where
        P: Provider<T, N> + 'static,
        T: Transport + Clone + 'static,
        N: Network,
    {
        // create the cache files
        std::fs::create_dir_all("cache").unwrap();

        // create all of the caches
        let mut pool_caches: Vec<PoolCache> = self
            .fetchers
            .keys()
            .map(|pool_type| read_cache_file(pool_type, self.chain))
            .collect();

        let end_block = provider.get_block_number().await.unwrap();

        // go though each cache, may or may not already by synced up to some point
        for cache in &mut pool_caches {
            let start_block = cache.last_synced_block;
            let fetcher = self.fetchers[&cache.pool_type].clone();

            // fetch all of the pool addresses
            let pool_addrs = Rpc::fetch_pool_addrs(
                start_block,
                end_block,
                provider.clone(),
                fetcher.clone(),
                self.chain,
                self.rate_limit,
            ).await.unwrap();

            // populate all of the pool addresses
            let populated_pools = Rpc::populate_pools(
                pool_addrs,
                provider.clone(),
                cache.pool_type,
                self.rate_limit
            ).await;

            // update the cache
            cache.pools.extend(populated_pools);
            cache.last_synced_block = end_block;
            write_cache_file(cache, self.chain);
        }

        // return all the pools
        Ok(pool_caches
            .into_iter()
            .flat_map(|cache| cache.pools)
            .collect())
    }

    pub async fn v2_pool_snapshot<P, T, N>(pool_addresses: Vec<Address>, provider: Arc<P>) -> Result<Vec<V2ReserveSnapshot>, PoolSyncError>
    where
        P: Provider<T, N> + 'static,
        T: Transport + Clone + 'static,
        N: Network,
    {
        sol!(
            #[derive(Debug)]
            #[sol(rpc)]
            V2ReserveUpdate,
            "src/abi/V2ReserveUpdate.json"
        );


        let total_tasks = (pool_addresses.len() + 39) / 40; // Ceiling division by 40
        let info = format!("{} address sync", pool_addresses.len());


        // Map all the addresses into chunks the contract can handle
        let addr_chunks: Vec<Vec<Address>> =
            pool_addresses.chunks(40).map(|chunk| chunk.to_vec()).collect();

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