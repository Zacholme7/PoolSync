//! PoolSync Core Implementation
//!
//! This module contains the core functionality for synchronizing pools across different
//! blockchain networks and protocols. It includes the main `PoolSync` struct and its
//! associated methods for configuring and executing the synchronization process.
//!
use alloy::network::Network;
use alloy::providers::Provider;
use alloy::providers::RootProvider;
use alloy::pubsub::PubSubFrontend;
use alloy::rpc::types::Filter;
use alloy::transports::Transport;
use futures::future::join_all;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Semaphore;

use crate::builder::PoolSyncBuilder;
use crate::cache::{read_cache_file, write_cache_file, PoolCache};
use crate::chain::Chain;
use crate::errors::*;
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
        // create a cache folder if it does not exist
        let path = Path::new("cache");
        if !path.exists() {
            let _ = fs::create_dir_all(path);
        }

        // load in pools from the cache
        let mut pool_caches: Vec<PoolCache> = Vec::new();
        for fetchers in self.fetchers.iter() {
            let pool_cache = read_cache_file(fetchers.0, self.chain);
            pool_caches.push(pool_cache);
        }

        // go though each cache, may or may not already by synced up to some point
        for cache in &mut pool_caches {
            // start at the last block this pool synced to, will be 10_000_000 if first sync
            // go to the current block
            let start_block = cache.last_synced_block;
            let end_block = provider.get_block_number().await.unwrap();

            // determine the number of steps we need in STEP_SIZE increments
            // this is just for progress bar
            let block_difference = end_block.saturating_sub(start_block);
            let (total_steps, step_size) = if block_difference < STEP_SIZE {
                (1, block_difference)
            } else {
                (
                    ((block_difference) as f64 / STEP_SIZE as f64).ceil() as u64,
                    STEP_SIZE,
                )
            };
            let progress_bar = self.create_progress_bar(total_steps);

            // the rate limiter is simply a semaphore, we will spawn all the tasks but only
            // 'rate_limit' amount will be able to request at one time
            let rate_limiter = Arc::new(Semaphore::new(self.rate_limit));

            // the handles of all the rpc requests to join on
            let mut handles = vec![];

            if block_difference > 0 {
                // go through each step in our range and spawn a task for it
                for from_block in (start_block..=end_block).step_by(step_size as usize) {
                    let to_block = (from_block + step_size - 1).min(end_block);
                    let fetcher = self.fetchers[&cache.pool_type].clone();
                    let handle = tokio::task::spawn(
                        PoolSync::fetch_and_process_block_range(
                            provider.clone(),
                            rate_limiter.clone(),
                            self.chain.clone(),
                            from_block,
                            to_block,
                            fetcher
                    ));
                    handles.push(handle);
                }

                // this is all of the pools that we have found, each pool is default init with just
                // the pool address
                let mut pools = Vec::new();
                let pools_with_addr = join_all(handles).await;
                for result in pools_with_addr {
                    if let Ok(p)  = result {
                        pools.extend(p);
                    }
                }
                println!("{:?}", pools.len());

                // once we have all the pools, we will populate each pool with its data
                //let populated_pools = self.populate_pool_data(provider.clone(), pools_with_addr);
                /* 
                for result in pools_with_addr {
                    match result {
                        Ok(Ok(pools)) => cache.pools.extend(pools),
                        Err(_) =>  return Err(PoolSyncError::ProviderError("blah".to_string()))
                    }
                }
                */
            }

            cache.last_synced_block = end_block;
        }

        // write all of the caches back to file
        for pool_cache in &pool_caches {
            write_cache_file(pool_cache, self.chain);
        }

        // save all of them in one vector
        let mut all_pools: Vec<Pool> = Vec::new();
        for pool_cache in &mut pool_caches {
            all_pools.append(&mut pool_cache.pools);
        }

        Ok(all_pools)
    }



    pub async fn fetch_and_process_block_range<P, T, N>(
        provider: Arc<P>,
        semaphore: Arc<Semaphore>,
        chain: Chain,
        from_block: u64,
        to_block: u64,
        fetcher: Arc<dyn PoolFetcher>,
    ) -> Vec<Pool> 
    where
        P: Provider<T, N> + 'static,
        T: Transport + Clone + 'static,
        N: Network,
    {
        let _permit = semaphore
                .acquire()
                .await.unwrap();

        let filter = Filter::new()
            .address(fetcher.factory_address(chain))
            .event(fetcher.pair_created_signature())
            .from_block(from_block)
            .to_block(to_block);

        let logs = provider.get_logs(&filter).await.unwrap();
        let mut pools = Vec::new();

        for log in logs {
            if let Some(pool) = fetcher.from_log(&log.inner).await {
                pools.push(pool);
            }
        }

        // populate all of the pools with the rest of the inforation
        //fetcher.populate_pool_data(pools);
        pools
    }

    /// Creates a progress bar for visual feedback during synchronization
    fn create_progress_bar(&self, total_steps: u64) -> ProgressBar {
        let pb = ProgressBar::new(total_steps);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] All pools: tasks completed {bar:40.cyan/blue} {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("##-"),
        );
        pb
    }
}
