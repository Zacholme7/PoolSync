use alloy::network::Network;
use alloy::primitives::Address;
use alloy::providers::Provider;
use alloy::rpc::types::Filter;
use alloy::transports::Transport;
use futures::future::join_all;
use futures::stream;
use futures::stream::StreamExt;
use rand::Rng;
use ratelimit::Ratelimiter;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::task::JoinHandle;

use crate::pools::PoolFetcher;
use crate::util::create_progress_bar;
use crate::Chain;
use crate::Pool;
use crate::PoolType;

/// The number of blocks to query in one call to get_logs
const STEP_SIZE: u64 = 10_000;
const MAX_RETRIES: u32 = 5;
const INITIAL_BACKOFF: u64 = 1000; // 1 second

pub struct Rpc;
impl Rpc {
    pub async fn fetch_pool_addrs<P, T, N>(
        start_block: u64,
        end_block: u64,
        provider: Arc<P>,
        fetcher: Arc<dyn PoolFetcher>,
        chain: Chain,
        requests_per_second: u64,
    ) -> Option<Vec<Address>>
    where
        P: Provider<T, N> + 'static,
        T: Transport + Clone + 'static,
        N: Network,
    {
        let block_difference = end_block.saturating_sub(start_block);

        if block_difference > 0 {
            let (total_steps, step_size) = if block_difference < STEP_SIZE {
                (1, block_difference)
            } else {
                (
                    ((block_difference as f64) / (STEP_SIZE as f64)).ceil() as u64,
                    STEP_SIZE,
                )
            };

            let info = format!("{} address sync", fetcher.pool_type());
            let progress_bar = create_progress_bar(total_steps, info);

            let block_ranges: Vec<_> = (start_block..=end_block)
                .step_by(step_size as usize)
                .map(|from_block| {
                    let to_block = (from_block + step_size - 1).min(end_block);
                    (from_block, to_block)
                })
                .collect();

            let results = stream::iter(block_ranges)
                .map(|(from_block, to_block)| {
                    let provider = provider.clone();
                    let fetcher = fetcher.clone();
                    let progress_bar = progress_bar.clone();

                    async move {
                        let mut retry_count = 0;
                        let mut backoff = INITIAL_BACKOFF;

                        loop {
                            let filter = Filter::new()
                                .address(fetcher.factory_address(chain))
                                .event(fetcher.pair_created_signature())
                                .from_block(from_block)
                                .to_block(to_block);

                            match provider.get_logs(&filter).await {
                                Ok(logs) => {
                                    let addresses: Vec<Address> = logs
                                        .iter()
                                        .map(|log| fetcher.log_to_address(&log.inner))
                                        .collect();

                                    progress_bar.inc(1);
                                    return addresses;
                                }
                                Err(e) => {
                                    if retry_count >= MAX_RETRIES {
                                        eprintln!(
                                            "Max retries reached for blocks {}-{}: {:?}",
                                            from_block, to_block, e
                                        );
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
                })
                .buffer_unordered(requests_per_second as usize)
                .collect::<Vec<Vec<Address>>>()
                .await;

            let all_addresses: Vec<Address> = results.into_iter().flatten().collect();
            Some(all_addresses)
        } else {
            None
        }
    }

    pub async fn populate_pools<P, T, N>(
        pool_addrs: Vec<Address>,
        provider: Arc<P>,
        pool: PoolType,
        requests_per_second: u64,
    ) -> Vec<Pool>
    where
        P: Provider<T, N> + 'static,
        T: Transport + Clone + 'static,
        N: Network,
    {
        let total_tasks = (pool_addrs.len() + 39) / 40; // Ceiling division by 40
        let info = format!("{} data sync", pool);
        let progress_bar = create_progress_bar(total_tasks as u64, info);
        println!("V3 addresses len: {}", pool_addrs.len());

        // Map all the addresses into chunks the contract can handle
        let addr_chunks: Vec<Vec<Address>> =
            pool_addrs.chunks(40).map(|chunk| chunk.to_vec()).collect();

        let results = stream::iter(addr_chunks)
            .map(|chunk| {
                let provider = provider.clone();
                let progress_bar = progress_bar.clone();
                let pool = pool.clone();

                async move {
                    let mut retry_count = 0;
                    let mut backoff = INITIAL_BACKOFF;

                    loop {
                        match pool
                            .build_pools_from_addrs(provider.clone(), chunk.clone())
                            .await
                        {
                            populated_pools if !populated_pools.is_empty() => {
                                progress_bar.inc(1);
                                return populated_pools;
                            }
                            _ => {
                                if retry_count >= MAX_RETRIES {
                                    eprintln!("Max retries reached for chunk");
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
            })
            .buffer_unordered(requests_per_second as usize * 2) // Allow some buffering for smoother operation
            .collect::<Vec<Vec<Pool>>>()
            .await;

        let populated_pools: Vec<Pool> = results.into_iter().flatten().collect();
        populated_pools
    }
}
