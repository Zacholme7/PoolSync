//! PoolSync Core Implementation
//!
//! This module contains the core functionality for synchronizing pools across different
//! blockchain networks and protocols. It includes the main `PoolSync` struct and its
//! associated methods for configuring and executing the synchronization process.
//!
use alloy::network::Network;
use alloy::providers::Provider;
use alloy::rpc::types::Filter;
use alloy::transports::Transport;
use futures::future::try_join_all;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
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
    ///
    /// This method performs the following steps:
    /// 1. Creates a cache folder if it doesn't exist
    /// 2. Reads the cache for each pool type
    /// 3. Synchronizes new data for each pool type
    /// 4. Updates and writes back the cache
    /// 5. Combines all synchronized pools into a single vector
    ///
    /// # Arguments
    ///
    /// * `provider` - An Arc-wrapped provider for interacting with the blockchain
    ///
    /// # Returns
    ///
    /// A Result containing a vector of all synchronized pools or a PoolSyncError
    pub async fn sync_pools<P, T, N>(&self, provider: Arc<P>) -> Result<Vec<Pool>, PoolSyncError>
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

        let mut pool_caches: Vec<PoolCache> = Vec::new(); // cache for each pool specified
                                                          // go through all the pools we want to sync
        for fetchers in self.fetchers.iter() {
            let pool_cache = read_cache_file(fetchers.0, self.chain);
            pool_caches.push(pool_cache);
        }

        for cache in &mut pool_caches {
            let start_block = cache.last_synced_block;
            let end_block = provider.get_block_number().await.unwrap();
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
            let rate_limiter = Arc::new(Semaphore::new(self.rate_limit));
            let mut handles = vec![];

            if block_difference > 0 {
                for from_block in (start_block..=end_block).step_by(step_size as usize) {
                    let to_block = (from_block + step_size - 1).min(end_block);
                    let handle = self.spawn_block_range_task(
                        provider.clone(),
                        rate_limiter.clone(),
                        self.fetchers.clone(),
                        from_block,
                        to_block,
                        progress_bar.clone(),
                        self.chain,
                    );
                    handles.push(handle);
                }

                for handle in handles {
                    let pools = handle
                        .await
                        .map_err(|e| PoolSyncError::ProviderError(e.to_string()))??;
                    cache.pools.extend(pools);
                }
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


    /// Spawns a task to process a range of blocks
    ///
    /// This method creates a new asynchronous task for processing a specific range of blocks.
    /// It uses a semaphore for rate limiting and updates a progress bar.
    ///
    /// # Arguments
    ///
    /// * `provider` - The blockchain provider
    /// * `semaphore` - A semaphore for rate limiting
    /// * `fetchers` - The pool fetchers
    /// * `from_block` - The starting block number
    /// * `to_block` - The ending block number
    /// * `progress_bar` - A progress bar for visual feedback
    /// * `chain` - The blockchain being synced
    ///
    /// # Returns
    ///
    /// A JoinHandle for the spawned task
    fn spawn_block_range_task<P, T, N>(
        &self,
        provider: Arc<P>,
        semaphore: Arc<Semaphore>,
        fetchers: HashMap<PoolType, Arc<dyn PoolFetcher>>,
        from_block: u64,
        to_block: u64,
        progress_bar: ProgressBar,
        chain: Chain,
    ) -> tokio::task::JoinHandle<Result<Vec<Pool>, PoolSyncError>>
    where
        P: Provider<T, N> + 'static,
        T: Transport + Clone + 'static,
        N: Network,
    {
        tokio::spawn(async move {
            let result = Self::process_block_range(
                provider,
                semaphore,
                fetchers,
                from_block,
                to_block,
                MAX_RETRIES,
                chain,
            )
            .await;
            progress_bar.inc(1);
            result
        })
    }

    /// Processes a range of blocks to find and decode pool creation events
    ///
    /// This method queries the blockchain for logs within the specified block range,
    /// decodes the logs into pool objects, and implements a retry mechanism for failed queries.
    ///
    /// # Arguments
    ///
    /// * `provider` - The blockchain provider
    /// * `semaphore` - A semaphore for rate limiting
    /// * `fetchers` - The pool fetchers
    /// * `from_block` - The starting block number
    /// * `to_block` - The ending block number
    /// * `max_retries` - The maximum number of retries for failed queries
    /// * `chain` - The blockchain being synced
    ///
    /// # Returns
    ///
    /// A Result containing a vector of found pools or a PoolSyncError
    async fn process_block_range<P, T, N>(
        provider: Arc<P>,
        semaphore: Arc<Semaphore>,
        fetchers: HashMap<PoolType, Arc<dyn PoolFetcher>>,
        from_block: u64,
        to_block: u64,
        max_retries: u32,
        chain: Chain,
    ) -> Result<Vec<Pool>, PoolSyncError>
    where
        P: Provider<T, N>,
        T: Transport + Clone + 'static,
        N: Network,
    {
        let mut retries = 0;
        loop {
            let _permit = semaphore
                .acquire()
                .await
                .map_err(|e| PoolSyncError::ProviderError(e.to_string()))?;

            let filters: Vec<Filter> = fetchers
                .values()
                .map(|fetcher| {
                    Filter::new()
                        .address(fetcher.factory_address(chain))
                        .event(fetcher.pair_created_signature())
                        .from_block(from_block)
                        .to_block(to_block)
                })
                .collect();

            let log_futures = filters.iter().map(|filter| provider.get_logs(filter));
            match try_join_all(log_futures).await {
                Ok(all_logs) => {
                    let mut pools = Vec::new();
                    for (logs, fetcher) in all_logs.into_iter().zip(fetchers.values()) {
                        for log in logs {
                            if let Some(pool) = fetcher.from_log(&log.inner).await {
                                pools.push(pool);
                            }
                        }
                    }
                    return Ok(pools);
                }
                Err(e) => {
                    if retries >= max_retries {
                        return Err(PoolSyncError::ProviderError(e.to_string()));
                    }
                    retries += 1;
                    let delay = 2u64.pow(retries) * 1000;
                    tokio::time::sleep(Duration::from_millis(delay)).await;
                }
            }
        }
    }

    /// Creates a progress bar for visual feedback during synchronization
    ///
    /// # Arguments
    ///
    /// * `total_steps` - The total number of steps in the synchronization process
    ///
    /// # Returns
    ///
    /// A configured ProgressBar instance
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
