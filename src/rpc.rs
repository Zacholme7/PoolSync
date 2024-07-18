use alloy::providers::Provider;
use alloy::transports::Transport;
use alloy::network::Network;
use futures::future::join_all;
use tokio::sync::Semaphore;
use tokio::task::JoinHandle;
use std::sync::Arc;
use alloy::primitives::Address;
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
    // Will fetch all of the pool addresses for the specified pool in the range start_block..end_block
    pub async fn fetch_pool_addrs<P, T, N>(
        start_block: u64, 
        end_block: u64, 
        provider: Arc<P>, 
        fetcher: Arc<dyn PoolFetcher>, 
        chain: Chain,
        rate_limit: usize
    ) -> Option<Vec<Address>> 
    where
        P: Provider<T, N> + 'static,
        T: Transport + Clone + 'static,
        N: Network,
    {
        let rate_limiter = Arc::new(Semaphore::new(rate_limit));
        let block_difference = end_block.saturating_sub(start_block);

        // if this is the first sync or there are new blocks
        if block_difference > 0 {
            // dertemine total steps and size
            let (total_steps, step_size) = if block_difference < STEP_SIZE {
                (1, block_difference)
            } else {
                (((block_difference as f64) / (STEP_SIZE as f64)).ceil() as u64,STEP_SIZE,)
            };

            let info = format!("{} address sync", fetcher.pool_type());
            let progress_bar = create_progress_bar(total_steps, info);

            // create all of the fetching futures
            let future_handles: Vec<JoinHandle<Vec<Address>>> = (start_block..=end_block)
                .step_by(step_size as usize)
                .map(|from_block| {
                    // state for the task,
                    // shadow arc variables
                    let to_block = (from_block + step_size -1).min(end_block);
                    let rate_limiter = rate_limiter.clone();
                    let provider = provider.clone();
                    let progress_bar = progress_bar.clone();
                    let fetcher = fetcher.clone();

                    // spawn the task that will query and process the logs
                    tokio::task::spawn(async move {
                        // if we can acquire, then we can request
                        let _permit = rate_limiter.acquire().await.unwrap();

                        // setup filter for the pool
                        let filter = Filter::new()
                            .address(fetcher.factory_address(chain))
                            .event(fetcher.pair_created_signature())
                            .from_block(from_block)
                            .to_block(to_block);

                        // fetch and process the logs
                        let logs = provider.get_logs(&filter).await.unwrap();
                        let addresses: Vec<Address> = logs.iter().map(|log| fetcher.log_to_address(&log.inner)).collect();
                        progress_bar.inc(1);
                        addresses
                    })
                }).collect();

                // await the futures and extract the addresses
                let pool_addrs = join_all(future_handles).await;
                let pool_addrs = pool_addrs.into_iter().filter_map(Result::ok).flatten().collect();
                return Some(pool_addrs)
        };
        None 
    }

    pub async fn populate_pools<P, T, N>(
        pool_addrs: Vec<Address>,
        provider: Arc<P>,
        pool: PoolType,
        rate_limit: usize
    ) -> Vec<Pool> 
    where
        P: Provider<T, N> + 'static,
        T: Transport + Clone + 'static,
        N: Network,
    {
        let total_tasks = pool_addrs.len() / 40;
        let info = format!("{} data sync", pool);
        let progress_bar = create_progress_bar(total_tasks as u64, info);
        
        // map all the addresses into chunks the contract can handle
        let addr_chuncks : Vec<Vec<Address>> = pool_addrs.chunks(40).map(|chunk| chunk.to_vec()).collect();

        let rate_limiter = Arc::new(Semaphore::new(rate_limit));

        let future_handles: Vec<JoinHandle<Vec<Pool>>> = addr_chuncks
            .into_iter()
            .map(|chunk| {
                let provider = provider.clone();
                let rate_limiter = rate_limiter.clone();
                let progress_bar = progress_bar.clone();
                tokio::task::spawn( async move {
                    let _permit = rate_limiter.acquire().await.unwrap();
                    let populated_pools = pool.build_pools_from_addrs(provider, chunk).await;
                    progress_bar.inc(1);
                    populated_pools
                })
            }).collect();

        let populated_pools = join_all(future_handles).await;
        let populated_pools: Vec<Pool> = populated_pools.into_iter().filter_map(Result::ok).flatten().collect();
        populated_pools
    }
}
