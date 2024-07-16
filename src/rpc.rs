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
        chain: Chain
    ) -> Option<Vec<Address>> 
    where
        P: Provider<T, N> + 'static,
        T: Transport + Clone + 'static,
        N: Network,
    {
        let rate_limiter = Arc::new(Semaphore::new(25));
        let block_difference = end_block.saturating_sub(start_block);

        // if this is the first sync or there are new blocks
        if block_difference > 0 {
            // dertemine total steps and size
            let (total_steps, step_size) = if block_difference < STEP_SIZE {
                (1, block_difference)
            } else {
                (((block_difference as f64) / (STEP_SIZE as f64)).ceil() as u64,STEP_SIZE,)
            };

            let progress_bar = create_progress_bar(total_steps);

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
        pool: PoolType
    ) -> Vec<Pool> 
    where
        P: Provider<T, N> + 'static,
        T: Transport + Clone + 'static,
        N: Network,
    {
        // map all the addresses into chunks the contract can handle
        let addr_chuncks : Vec<Vec<Address>> = pool_addrs.chunks(50).map(|chunk| chunk.to_vec()).collect();

        let future_handles: Vec<JoinHandle<Vec<Pool>>> = addr_chuncks
            .into_iter()
            .map(|chunk| {
                let provider = provider.clone();
                tokio::task::spawn( async move {
                    pool.build_pools_from_addrs(provider, chunk).await
                })
            }).collect();

        let populated_pools = join_all(future_handles).await;
        let populated_pools: Vec<Pool> = populated_pools.into_iter().filter_map(Result::ok).flatten().collect();
        populated_pools
    }
}

/* 
pub async fn populate_pool_data_helper<P, T, N>(
    provider: Arc<P>,
    cache: Vec<Address>,
    semaphore: Arc<Semaphore>,
    fetcher: Arc<dyn PoolFetcher>,
) -> Vec<Pool>
where
    P: Provider<T, N> + 'static,
    T: Transport + Clone + 'static,
    N: Network,
{
    let _permit = semaphore.acquire().await.unwrap();

    let deployer = UniswapV2DataSync::deploy_builder(provider, cache);
    let res = deployer.call().await.unwrap();
    let constructor_return = DynSolType::Array(Box::new(DynSolType::Tuple(vec![
        DynSolType::Address,
        DynSolType::Address,
        DynSolType::Uint(8),
        DynSolType::String,
        DynSolType::Address,
        DynSolType::Uint(8),
        DynSolType::String,
        DynSolType::Uint(112),
        DynSolType::Uint(112),
    ])));
    let return_data_tokens = constructor_return.abi_decode_sequence(&res).unwrap();

        let mut pools = Vec::new();
        if let Some(tokens_arr) = return_data_tokens.as_array() {
            for token in tokens_arr {
                if let Some(tokens) = token.as_tuple() {
                    let pool = fetcher.construct_pool_from_data(tokens);
                    pools.push(pool);
                }
            }
        }
        pools
    }

    pub async fn populate_pool_data<P, T, N>(&self, provider: Arc<P>, cache: &mut PoolCache)
    where
        P: Provider<T, N> + 'static,
        T: Transport + Clone + 'static,
        N: Network,
    {
        // collect the pool addresses and separate them into chuncks
        let pool_addresses: Vec<Address> = cache.pools.iter().map(|p| p.address()).collect();
        
        let addr_chunks: Vec<Vec<Address>> = pool_addresses
            .chunks(5)
            .map(|chunk| chunk.to_vec())
            .collect();




        let mut handles = Vec::new();

        let rate_limiter = Arc::new(Semaphore::new(self.rate_limit));

        let fetcher = self.fetchers[&cache.pool_type].clone();
        for chunk in addr_chunks {
            let provider_clone = provider.clone();
            let handle = tokio::task::spawn(PoolSync::populate_pool_data_helper(
                provider.clone(),
                chunk,
                rate_limiter.clone(),
                fetcher.clone(),
            ));
            handles.push(handle);
        }

        let mut data_pools = Vec::new();
        let results = join_all(handles).await;
        for res in results {
            if let Ok(res) = res {
                data_pools.extend(res);
            }
        }
        println!("data_pools: {}", data_pools.len());
    }
                
                 */