use alloy::providers::Provider;
use alloy::transports::Transport;
use alloy::network::Network;
use futures::future::join_all;
use tokio::sync::Semaphore;
use tokio::task::JoinHandle;
use std::sync::Arc;
use alloy::primitives::Address;
use futures::stream;
use futures::stream::StreamExt;
use std::time::Duration;
use ratelimit::Ratelimiter;
use alloy::rpc::types::Filter;

use crate::pools::PoolFetcher;
use crate::util::create_progress_bar;
use crate::Chain;
use crate::Pool;
use crate::PoolType;

/// The number of blocks to query in one call to get_logs
const STEP_SIZE: u64 = 10_000;

pub struct Rpc;
impl Rpc {
    pub async fn fetch_pool_addrs<P, T, N>(
        start_block: u64, 
        end_block: u64, 
        provider: Arc<P>, 
        fetcher: Arc<dyn PoolFetcher>, 
        chain: Chain,
        requests_per_second: u64
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
                (((block_difference as f64) / (STEP_SIZE as f64)).ceil() as u64, STEP_SIZE)
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
    
            let ratelimiter = Arc::new(
                Ratelimiter::builder(requests_per_second, Duration::from_secs(1))
                    .max_tokens(requests_per_second)
                    .initial_available(requests_per_second)
                    .build()
                    .expect("Failed to create ratelimiter")
            );
    
            let results = stream::iter(block_ranges)
                .map(|(from_block, to_block)| {
                    let provider = provider.clone();
                    let fetcher = fetcher.clone();
                    let progress_bar = progress_bar.clone();
                    let ratelimiter = ratelimiter.clone();
                    
                    async move {
                        loop {
                            match ratelimiter.try_wait() {
                                Ok(_) => break,
                                Err(sleep) => tokio::time::sleep(sleep).await,
                            }
                        }
    
                        let filter = Filter::new()
                            .address(fetcher.factory_address(chain))
                            .event(fetcher.pair_created_signature())
                            .from_block(from_block)
                            .to_block(to_block);
                        
                        let logs = provider.get_logs(&filter).await.unwrap();
                        let addresses: Vec<Address> = logs.iter()
                            .map(|log| fetcher.log_to_address(&log.inner))
                            .collect();
                        
                        progress_bar.inc(1);
                        addresses
                    }
                })
                .buffer_unordered(requests_per_second as usize * 2) // Allow some buffering for smoother operation
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
        requests_per_second: u64
    ) -> Vec<Pool> 
    where
        P: Provider<T, N> + 'static,
        T: Transport + Clone + 'static,
        N: Network,
    {
        let total_tasks = (pool_addrs.len() + 39) / 40; // Ceiling division by 40
        let info = format!("{} data sync", pool);
        let progress_bar = create_progress_bar(total_tasks as u64, info);
        
        // Map all the addresses into chunks the contract can handle
        let addr_chunks: Vec<Vec<Address>> = pool_addrs.chunks(40).map(|chunk| chunk.to_vec()).collect();
    
        let ratelimiter = Arc::new(
            Ratelimiter::builder(requests_per_second, Duration::from_secs(1))
                .max_tokens(requests_per_second)
                .initial_available(requests_per_second)
                .build()
                .expect("Failed to create ratelimiter")
        );
    
        let results = stream::iter(addr_chunks)
            .map(|chunk| {
                let provider = provider.clone();
                let ratelimiter = ratelimiter.clone();
                let progress_bar = progress_bar.clone();
                let pool = pool.clone();
                
                async move {
                    loop {
                        match ratelimiter.try_wait() {
                            Ok(_) => break,
                            Err(sleep) => tokio::time::sleep(sleep).await,
                        }
                    }
    
                    let populated_pools = pool.build_pools_from_addrs(provider, chunk).await;
                    progress_bar.inc(1);
                    populated_pools
                }
            })
            .buffer_unordered(requests_per_second as usize * 2) // Allow some buffering for smoother operation
            .collect::<Vec<Vec<Pool>>>()
            .await;
    
        let populated_pools: Vec<Pool> = results.into_iter().flatten().collect();
        populated_pools
    }


}
