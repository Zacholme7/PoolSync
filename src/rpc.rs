use alloy::network::Network;
use alloy::primitives::Address;
use alloy::providers::Provider;
use alloy::rpc::types::Filter;
use alloy::rpc::types::Log;
use alloy::signers::k256::elliptic_curve::bigint::modular::montgomery_reduction;
use alloy::signers::k256::elliptic_curve::rand_core::block;
use alloy::transports::Transport;
use alloy_sol_types::SolEvent;
use futures::stream;
use futures::stream::StreamExt;
use rand::Rng;
use tokio::sync::Semaphore;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::os::unix::process;
use std::sync::Arc;
use std::time::Duration;
use alloy::sol;

use crate::pools::pool_structure::UniswapV3Pool;
use crate::pools::PoolFetcher;
use crate::pools::process_tick_data;
use crate::PoolInfo;
use crate::util::create_progress_bar;
use crate::Chain;
use crate::Pool;
use crate::PoolType;

/// The number of blocks to query in one call to get_logs
const STEP_SIZE: u64 = 10_000;
const MAX_RETRIES: u32 = 5;
const INITIAL_BACKOFF: u64 = 1000; // 1 second

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    contract V3Events {
        event Burn(address indexed owner, int24 indexed tickLower, int24 indexed tickUpper, uint128 amount, uint256 amount0, uint256 amount1);
        event Mint(address sender, address indexed owner, int24 indexed tickLower, int24 indexed tickUpper, uint128 amount, uint256 amount0, uint256 amount1);
    }
);

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
                                    drop(provider);
                                    return addresses;
                                }
                                Err(e) => {
                                    if retry_count >= MAX_RETRIES {
                                        eprintln!(
                                            "Max retries reached for blocks {}-{}: {:?}",
                                            from_block, to_block, e
                                        );
                                        drop(provider);
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


                drop(provider);
            let all_addresses: Vec<Address> = results.into_iter().flatten().collect();
            println!("got all of the addresses {}", all_addresses.len());
            Some(all_addresses)
        } else {
            None
        }
    }

    pub async fn populate_pools<P, T, N>(
        start_block: u64, 
        end_block: u64,
        pool_addrs: Vec<Address>,
        provider: Arc<P>,
        archive: Arc<P>,
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
        println!("Start block {}, end block {}", start_block, end_block);
        
        let rate_limiter = Arc::new(Semaphore::new(100 as usize));

        // Map all the addresses into chunks the contract can handle
        let addr_chunks: Vec<Vec<Address>> =
            pool_addrs.chunks(40).map(|chunk| chunk.to_vec()).collect();

        let results = stream::iter(addr_chunks)
            .map(|chunk| {
                let provider = provider.clone();
                let archive = archive.clone();
                let progress_bar = progress_bar.clone();
                let pool = pool.clone();
                let rate_limiter = rate_limiter.clone();

                async move {
                    let mut retry_count = 0;
                    let mut backoff = INITIAL_BACKOFF;
                    let _permit = rate_limiter.acquire().await.unwrap();

                    loop {
                        match pool
                            .build_pools_from_addrs((start_block, end_block),provider.clone(), archive.clone(), chunk.clone())
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

        let mut populated_pools: Vec<Pool> = results.into_iter().flatten().collect();

        // populate the tick data
        Rpc::populate_tick_data(start_block, end_block, &mut populated_pools, archive.clone()).await;

        populated_pools
    }

    pub async fn populate_tick_data<P, T, N>(
        start_block: u64,
        end_block: u64,
        pools: &mut Vec<Pool>,
        provider: Arc<P>
    ) 
    where 
        P: Provider<T, N> + Sync + 'static,
        T: Transport + Sync + Clone,
        N: Network,
    {

        let block_difference = end_block.saturating_sub(start_block);
        let address_to_index: HashMap<Address, usize> = pools.iter().enumerate().map(|(i, pool)| (pool.address(), i)).collect();

        if block_difference > 0 {
            let (total_steps, step_size) = if block_difference < 5000 {
                (1, block_difference)
            } else {
                (
                    ((block_difference as f64) / (5000 as f64)).ceil() as u64,
                    5000,
                )
            };

            let info = format!("{} tick data sync", pools.len());
            let progress_bar = create_progress_bar(total_steps, info);

            let block_ranges: Vec<_> = (start_block..=end_block)
                .step_by(5000 as usize)
                .map(|from_block| {
                    let to_block = (from_block + step_size - 1).min(end_block);
                    (from_block, to_block)
                })
                .collect();

            // fetch all of the logsa and fla
            let logs  = stream::iter(block_ranges.clone())
                .map(|(from_block, to_block)| {
                    let provider = provider.clone();
                    async move {
                        // get all of the burn and mint events
                        let filter = Filter::new()
                            .event_signature(vec![
                                V3Events::Burn::SIGNATURE_HASH,
                                V3Events::Mint::SIGNATURE_HASH,
                            ])
                            .from_block(from_block)
                            .to_block(to_block);
                        println!("Getting the logs for range {}-{}", from_block, to_block);
                        let logs = provider.get_logs(&filter).await.unwrap();
                        logs
                    }
                })
                .buffer_unordered(100) // Allow some buffering for smoother operation
                .collect::<Vec<Vec<Log>>>()
                .await;
            let logs: Vec<Log> = logs.into_iter().flatten().collect();
            println!("got all of the logs");

            let mut ordered_logs:BTreeMap<u64, Vec<Log>> = BTreeMap::new();
            for log in logs {
                if let Some(block_number) = log.block_number {
                    if let Some(log_group) = ordered_logs.get_mut(&block_number) {
                        log_group.push(log);
                    } else {
                        ordered_logs.insert(block_number, vec![log]);
                    }
                }
            }

            // process all the logs for the pool
            for (_, log_group) in ordered_logs {
                for log in log_group {
                    let address = log.address();
                    if let Some(&index) = address_to_index.get(&address) {
                        if let Some(pool) = pools.get_mut(index) {  // Note: removed & before index
                            match pool {
                                Pool::UniswapV3(p) | Pool::SushiSwapV3(p) | Pool::PancakeSwapV3(p) => {
                                    process_tick_data(p, log);
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }
}
