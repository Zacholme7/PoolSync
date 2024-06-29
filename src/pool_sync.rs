use alloy::providers::Provider;
use alloy::rpc::types::Filter;
use alloy::network::Network;
use alloy::transports::Transport;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use futures::future::try_join_all;

use crate::cache::{PoolCache, read_cache_file, write_cache_file};
use crate::pools::*;
use crate::chain::Chain;
use crate::builder::PoolSyncBuilder;
use crate::errors::*;


// The amount of blocks we want to query in one call to get_logs
const STEP_SIZE: u64 = 10_000;
// How many time we want to retry a query if it fails
const MAX_RETRIES: u32 = 5;
// The number of requests to send off at one time, this is to protect
// against public rpc rate limits
const CONCURRENT_REQUESTS: usize = 10;

pub struct PoolSync {
    // map a pool type to its fetcher implementation
    pub fetchers: HashMap<PoolType, Arc<dyn PoolFetcher>>,
    /// the chain that we want to sync on
    pub chain: Chain ,
}

impl PoolSync {
    /// Construct a new builder to configure sync parameters
    pub fn builder() -> PoolSyncBuilder {
        PoolSyncBuilder::default()
    }

    /// After configuring the builder, sync all added pools for the specified chain
    pub async fn sync_pools<P, T, N>(&self, provider: Arc<P>) -> Result<Vec<Pool>, PoolSyncError>
    where
        P: Provider<T, N> + 'static,
        T: Transport + Clone + 'static,
        N: Network
    {
        let mut all_pools: HashSet<Pool> = HashSet::new(); // all of the synced pools
        let mut pool_caches : Vec<PoolCache> = Vec::new(); // cache for each pool specified

        // go through all the pools we want to sync
        for fetchers in self.fetchers.iter() {
                let pool_cache = read_cache_file(fetchers.0);
                pool_caches.push(pool_cache);
        }

        // go through each cache and sync th epools
        for cache in &mut pool_caches {
                // setup steps
                let start_block = cache.last_synced_block;
                let end_block = provider.get_block_number().await.unwrap();
                let total_steps = ((end_block - start_block) as f64 / STEP_SIZE as f64).ceil() as u64;

                // progress bar for sync feedback
                let progress_bar = self.create_progress_bar(total_steps);

                // create all of the handles for the current sync
                let rate_limiter = Arc::new(Semaphore::new(CONCURRENT_REQUESTS));
                let mut handles = vec![];
                for from_block in (start_block..end_block).step_by(STEP_SIZE as usize) {
                        let to_block = (from_block + STEP_SIZE - 1).min(end_block);
                        let handle = self.spawn_block_range_task(
                                provider.clone(),
                                rate_limiter.clone(),
                                self.fetchers.clone(),
                                from_block,
                                to_block,
                                progress_bar.clone(),
                        );
                        handles.push(handle);
                }

                // sync all 
                for handle in handles {
                        let pools = handle.await.map_err(|e| PoolSyncError::ProviderError(e.to_string()))??;
                        cache.pools.extend(pools);
                }
        }



        // write all of the caches back to file
        for pool_cache in &pool_caches {
                write_cache_file(pool_cache);
        }

        // save all of them in one vector
        let mut all_pools : Vec<Pool> = Vec::new();
        for pool_cache in &mut pool_caches {
                all_pools.append(&mut pool_cache.pools);
        }

        Ok(all_pools)
    }

    fn spawn_block_range_task<P, T, N>(
        &self,
        provider: Arc<P>,
        semaphore: Arc<Semaphore>,
        fetchers: HashMap<PoolType, Arc<dyn PoolFetcher>>,
        from_block: u64,
        to_block: u64,
        progress_bar: ProgressBar,
    ) -> tokio::task::JoinHandle<Result<Vec<Pool>, PoolSyncError>>
    where
        P: Provider<T, N> + 'static,
        T: Transport + Clone + 'static,
        N: Network
    {
        tokio::spawn(async move {
            let result = Self::process_block_range(
                provider,
                semaphore,
                fetchers,
                from_block,
                to_block,
                MAX_RETRIES,
            )
            .await;
            progress_bar.inc(1);
            result
        })
    }

    async fn process_block_range<P, T, N>(
        provider: Arc<P>,
        semaphore: Arc<Semaphore>,
        fetchers: HashMap<PoolType, Arc<dyn PoolFetcher>>,
        from_block: u64,
        to_block: u64,
        max_retries: u32,
    ) -> Result<Vec<Pool>, PoolSyncError>
    where
        P: Provider<T, N>,
        T: Transport + Clone + 'static,
        N: Network
    {
        let mut retries = 0;
        loop {
            let _permit = semaphore.acquire().await.map_err(|e| PoolSyncError::ProviderError(e.to_string()))?;

            let filters: Vec<Filter> = fetchers
                .values()
                .map(|fetcher| {
                    Filter::new()
                        .address(fetcher.factory_address())
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
