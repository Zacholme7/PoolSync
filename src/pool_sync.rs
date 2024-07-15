//! PoolSync Core Implementation
//!
//! This module contains the core functionality for synchronizing pools across different
//! blockchain networks and protocols. It includes the main `PoolSync` struct and its
//! associated methods for configuring and executing the synchronization process.
//!
use alloy::dyn_abi::{DynSolType, DynSolValue};
use alloy::network::Network;
use alloy::primitives::Address;
use alloy::providers::Provider;
use alloy::providers::RootProvider;
use alloy::pubsub::PubSubFrontend;
use alloy::rpc::types::Filter;
use alloy::signers::k256::sha2::digest::KeyInit;
use alloy::sol_types::{SolCall, SolInterface};
use alloy::transports::Transport;
use futures::future::join_all;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::task::JoinHandle;

use crate::builder::PoolSyncBuilder;
use crate::cache::{read_cache_file, write_cache_file, PoolCache};
use crate::chain::Chain;
use crate::errors::*;
use crate::pools::uniswap_v2::UniswapV2DataSync;
use crate::pools::*;

/// The number of blocks to query in one call to get_logs
const STEP_SIZE: u64 = 10_000;

/// The maximum number of retries for a failed query
const MAX_RETRIES: u32 = 5;

/// The main struct for pool synchronization
pub struct PoolSync {
    /// Map of pool types to their fetcher implementations
    pub fetchers: HashMap<PoolType, Arc<dyn PoolFetcher>>,
    /// The chain to sync on
    pub chain: Chain,
    /// The rate limit of the rpc
    pub rate_limit: usize,
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
        ws: Arc<RootProvider<PubSubFrontend, N>>,
    ) -> Result<Vec<Pool>, PoolSyncError>
    where
        P: Provider<T, N> + 'static,
        T: Transport + Clone + 'static,
        N: Network,
    {
        // create the cache files
        std::fs::create_dir_all("cache").unwrap();

        // get all teh eachers
        let mut pool_caches: Vec<PoolCache> = self
            .fetchers
            .keys()
            .map(|pool_type| read_cache_file(pool_type, self.chain))
            .collect();

        let end_block = provider.get_block_number().await.unwrap();
        let rate_limiter = Arc::new(Semaphore::new(self.rate_limit));

        // go though each cache, may or may not already by synced up to some point
        for cache in &mut pool_caches {
            // start at the last block this pool synced to, will be 10_000_000 if first sync
            // go to the current block
            let start_block = cache.last_synced_block;
            let block_difference = end_block.saturating_sub(start_block);


            let pools = Vec::new();

            if block_difference > 0 {
                let (total_steps, step_size) = if block_difference < STEP_SIZE {
                    (1, block_difference)
                } else {
                    (
                        ((block_difference as f64) / (STEP_SIZE as f64)).ceil() as u64,
                        STEP_SIZE,
                    )
                };

                let progress_bar = self.create_progress_bar(total_steps);
                let fetcher = self.fetchers[&cache.pool_type].clone();

                let pool_addrs = Rpc::fetch_pool_addrs(provider.clone());
                let populated_pools = Rpc::populate_pools(pool_addrs, provider.clone());
                cache.pools = populated_pools;
            }

            cache.last_synced_block = end_block;
             write_cache_file(cache, self.chain)?;
        }

        // return all the pools
        Ok(pool_caches
            .into_iter()
            .flat_map(|cache| cache.pools)
            .collect())
    }


}

/// Creates a progress bar for visual feedback during synchronization
fn create_progress_bar(total_steps: u64) -> ProgressBar {
let pb = ProgressBar::new(total_steps);
pb.set_style(
        ProgressStyle::default_bar()
        .template("[{elapsed_precise}] All pools: tasks completed {bar:40.cyan/blue} {pos}/{len} {msg}")
        .unwrap()
        .progress_chars("##-"),
);
pb
}