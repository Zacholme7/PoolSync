use alloy::{
    network::Ethereum,
    providers::Provider,
    rpc::types::Filter,
    transports::Transport,
};
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
    sync::Arc,
    time::Duration,
};
use thiserror::Error;
use tokio::sync::Semaphore;

use crate::pools::{
    uniswap_v2::UniswapV2Fetcher, 
    uniswap_v3::UniswapV3Fetcher,
    sushiswap::SushiSwapFetcher,
    Pool, 
    PoolFetcher, 
    PoolType
};
use futures::future::try_join_all;

const CACHE_FILE: &str = "pool_sync_cache.json";
const DEFAULT_START_BLOCK: u64 = 10_000_000;
const STEP_SIZE: u64 = 10_000;
const MAX_RETRIES: u32 = 5;
const CONCURRENT_REQUESTS: usize = 25;

#[derive(Error, Debug)]
pub enum PoolSyncError {
    #[error("Provider error: {0}")]
    ProviderError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

#[derive(Serialize, Deserialize)]
struct CacheData {
    last_synced_block: u64,
    pools: Vec<Pool>,
}

pub struct PoolSync {
    fetchers: HashMap<PoolType, Arc<dyn PoolFetcher>>,
}

impl PoolSync {
    pub fn builder() -> PoolSyncBuilder {
        PoolSyncBuilder::default()
    }

    pub async fn sync_pools<P, T>(&self, provider: Arc<P>) -> Result<Vec<Pool>, PoolSyncError>
    where
        P: Provider<T, Ethereum> + 'static,
        T: Transport + Clone + 'static,
    {
        let mut all_pools: HashSet<Pool> = HashSet::new();
        let (start_block, cached_pools) = self.read_cache(CACHE_FILE)?;
        all_pools.extend(cached_pools);

        let latest_block = provider.get_block_number().await.map_err(|e| PoolSyncError::ProviderError(e.to_string()))?;
        let total_steps = ((latest_block - start_block) as f64 / STEP_SIZE as f64).ceil() as u64;

        let progress_bar = self.create_progress_bar(total_steps);

        let rate_limiter = Arc::new(Semaphore::new(CONCURRENT_REQUESTS));
        let mut handles = vec![];

        for from_block in (start_block..latest_block).step_by(STEP_SIZE as usize) {
            let to_block = (from_block + STEP_SIZE - 1).min(latest_block);
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

        for handle in handles {
            let pools = handle.await.map_err(|e| PoolSyncError::ProviderError(e.to_string()))??;
            all_pools.extend(pools);
        }

        let unique_pools: Vec<Pool> = all_pools.into_iter().collect();
        self.write_cache(CACHE_FILE, &unique_pools, latest_block)?;

        Ok(unique_pools)
    }

    fn spawn_block_range_task<P, T>(
        &self,
        provider: Arc<P>,
        semaphore: Arc<Semaphore>,
        fetchers: HashMap<PoolType, Arc<dyn PoolFetcher>>,
        from_block: u64,
        to_block: u64,
        progress_bar: ProgressBar,
    ) -> tokio::task::JoinHandle<Result<Vec<Pool>, PoolSyncError>>
    where
        P: Provider<T, Ethereum> + 'static,
        T: Transport + Clone + 'static,
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

    async fn process_block_range<P, T>(
        provider: Arc<P>,
        semaphore: Arc<Semaphore>,
        fetchers: HashMap<PoolType, Arc<dyn PoolFetcher>>,
        from_block: u64,
        to_block: u64,
        max_retries: u32,
    ) -> Result<Vec<Pool>, PoolSyncError>
    where
        P: Provider<T, Ethereum>,
        T: Transport + Clone + 'static,
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

    fn read_cache(&self, cache_file: &str) -> Result<(u64, Vec<Pool>), PoolSyncError> {
        if Path::new(cache_file).exists() {
            let file_content = fs::read_to_string(cache_file)?;
            let cache_data: CacheData = serde_json::from_str(&file_content)?;
            Ok((cache_data.last_synced_block, cache_data.pools))
        } else {
            Ok((DEFAULT_START_BLOCK, Vec::new()))
        }
    }

    fn write_cache(&self, cache_file: &str, pools: &[Pool], last_synced_block: u64) -> Result<(), PoolSyncError> {
        let cache_data = CacheData {
            last_synced_block,
            pools: pools.to_vec(),
        };
        let json = serde_json::to_string(&cache_data)?;
        fs::write(cache_file, json)?;
        Ok(())
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

#[derive(Default)]
pub struct PoolSyncBuilder {
    fetchers: HashMap<PoolType, Arc<dyn PoolFetcher>>,
}

impl PoolSyncBuilder {
    pub fn add_pool(mut self, pool_type: PoolType) -> Self {
        match pool_type {
            PoolType::UniswapV2 => {
                self.fetchers
                    .insert(PoolType::UniswapV2, Arc::new(UniswapV2Fetcher));
            }
            PoolType::UniswapV3 => {
                self.fetchers
                    .insert(PoolType::UniswapV3, Arc::new(UniswapV3Fetcher));
            }
            PoolType::SushiSwap => {
                self.fetchers
                    .insert(PoolType::SushiSwap, Arc::new(SushiSwapFetcher));
            }
        }
        self
    }

    pub fn build(self) -> PoolSync {
        PoolSync {
            fetchers: self.fetchers,
        }
    }
}
