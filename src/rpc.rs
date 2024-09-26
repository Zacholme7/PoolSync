use alloy::network::Network;
use alloy::primitives::{Address, FixedBytes};
use alloy::providers::Provider;
use alloy::rpc::types::{Filter, Log};
use alloy::sol_types::SolEvent;
use alloy::transports::Transport;
use anyhow::Result;
use log::warn;
use rand::Rng;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;

use crate::events::*;
use crate::pools::pool_builder;
use crate::pools::pool_structures::balancer_v2_structure::process_balance_data;
use crate::pools::pool_structures::v2_structure::process_sync_data;
use crate::pools::pool_structures::v3_structure::process_tick_data;
use crate::pools::PoolFetcher;
use crate::util::create_progress_bar;
use crate::Chain;
use crate::Pool;
use crate::PoolInfo;
use crate::PoolType;

/// The number of blocks to query in one call to get_logs
const STEP_SIZE: u64 = 10000;
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
        // if there is a range to fetch
        if end_block.saturating_sub(start_block) > 0 {
            // construct set of block ranges to retrieve events over
            let block_ranges = Rpc::get_block_range(start_block, end_block, STEP_SIZE);

            // construct the progress bar
            let info = format!("{} address sync", fetcher.pool_type());
            let progress_bar = create_progress_bar(block_ranges.len().try_into().unwrap(), info);

            // semaphore to rate limit requests
            let rate_limiter = Arc::new(Semaphore::new(requests_per_second.try_into().unwrap()));
            
            // fetch all retsuls
            let results = stream::iter(block_ranges)
                .map(|(from_block, to_block)| {
                    let provider = provider.clone();
                    let fetcher = fetcher.clone();
                    let progress_bar = progress_bar.clone();
                    let rate_limiter = rate_limiter.clone();

                    async move {
                        let mut retry_count = 0;
                        let mut backoff = INITIAL_BACKOFF;
                        let _permit = rate_limiter.acquire().await.unwrap();

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
                .buffer_unordered(10 as usize)
                .collect::<Vec<Vec<Address>>>()
                .await;

            drop(provider);
            let all_addresses: Vec<Address> = results.into_iter().flatten().collect();
            Some(all_addresses)
        } else {
            Some(vec![])
        }
    }

    pub async fn populate_pools<P, T, N>(
        pool_addrs: Vec<Address>,
        provider: Arc<P>,
        pool: PoolType,
        fetcher: Arc<dyn PoolFetcher>,
        requests_per_second: u64,
    ) -> Vec<Pool>
    where
        P: Provider<T, N> + 'static,
        T: Transport + Clone + 'static,
        N: Network,
    {
        let BATCH_SIZE = if pool.is_balancer() { 10 } else { 40 };
        let total_tasks = (pool_addrs.len() + 39) / BATCH_SIZE; // Ceiling division by 40
        let info = format!("{} data sync", pool);
        let progress_bar = create_progress_bar(total_tasks as u64, info);

        // Map all the addresses into chunks the contract can handle
        let addr_chunks: Vec<Vec<Address>> = pool_addrs
            .chunks(BATCH_SIZE)
            .map(|chunk| chunk.to_vec())
            .collect();

        let results = stream::iter(addr_chunks)
            .map(|chunk| {
                let provider = provider.clone();
                let progress_bar = progress_bar.clone();
                let pool = pool.clone();
                let fetcher = fetcher.clone();

                async move {
                    let mut retry_count = 0;
                    let mut backoff = INITIAL_BACKOFF;
                    let data = fetcher.get_pool_repr();

                    loop {
                        match pool_builder::build_pools(
                            provider.clone(),
                            chunk.clone(),
                            pool,
                            data.clone(),
                        )
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
            .buffer_unordered(1000 as usize) // Allow some buffering for smoother operation
            .collect::<Vec<Vec<Pool>>>()
            .await;

        let populated_pools: Vec<Pool> = results.into_iter().flatten().collect();
        populated_pools
    }

    pub async fn populate_liquidity<P, T, N>(
        start_block: u64,
        end_block: u64,
        pools: &mut Vec<Pool>,
        provider: Arc<P>,
        pool_type: PoolType,
    ) -> Result<()>
    where
        P: Provider<T, N> + Sync + 'static,
        T: Transport + Sync + Clone,
        N: Network,
    {
        if pools.len() == 0 {
            return Ok(());
        }
        // get the block difference
        let block_difference = end_block.saturating_sub(start_block);
        let address_to_index: HashMap<Address, usize> = pools
            .iter()
            .enumerate()
            .map(|(i, pool)| (pool.address(), i))
            .collect();

        if block_difference > 0 {
            // create the progress bar

            let mut new_logs: Vec<Log> = Vec::new();
            if pool_type.is_v3() {
                // fetch all mint/burn/swap logs
                new_logs.extend(
                    Rpc::fetch_tick_logs(start_block, end_block, provider.clone(), pool_type)
                        .await
                        .unwrap(),
                );
                if start_block > 10_000_000 {
                    // make sure we dont fetch swap events after initial sync
                    new_logs.extend(
                        Rpc::fetch_swap_logs(start_block, end_block, provider.clone(), pool_type)
                            .await
                            .unwrap(),
                    );
                }
            } else if pool_type.is_balancer() {
                if start_block > 10_000_000 {
                    new_logs.extend(
                        Rpc::fetch_balance_logs(
                            start_block,
                            end_block,
                            provider.clone(),
                            pool_type,
                        )
                        .await
                        .unwrap(),
                    );
                }
            } else {
                // fetch all sync logs
                if start_block > 10_000_000 {
                    new_logs.extend(
                        Rpc::fetch_sync_logs(start_block, end_block, provider.clone(), pool_type)
                            .await
                            .unwrap(),
                    );
                }
            }

            // order all of the logs by block number
            let mut ordered_logs: BTreeMap<u64, Vec<Log>> = BTreeMap::new();
            for log in new_logs {
                if let Some(block_number) = log.block_number {
                    if let Some(log_group) = ordered_logs.get_mut(&block_number) {
                        log_group.push(log);
                    } else {
                        ordered_logs.insert(block_number, vec![log]);
                    }
                }
            }

            // process all of the logs
            for (_, log_group) in ordered_logs {
                for log in log_group {
                    let address = log.address();
                    if let Some(&index) = address_to_index.get(&address) {
                        if let Some(pool) = pools.get_mut(index) {
                            // Note: removed & before index
                            if pool_type.is_v3() {
                                process_tick_data(pool.get_v3_mut().unwrap(), log, pool_type);
                            } else if pool_type.is_balancer() {
                                process_balance_data(pool.get_balancer_mut().unwrap(), log);
                            } else {
                                process_sync_data(pool.get_v2_mut().unwrap(), log, pool_type);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }


    // fetch all the logs for a specific event set
    pub async fn fetch_logs<T, N, P>(
        start_block: u64, 
        end_block: u64, 
        step_size: u64,
        events: Vec<FixedBytes<32>>, 
        provider: Arc<P>,
        pb_info: String,
        rate_limit: u64
    )  -> Result<Vec<Log>>
    where
        T: Transport + Clone,
        N: Network,
        P: Provider<T, N>
    {
        // generate the block range for the sync and setup progress bar
        let block_range = Rpc::get_block_range(step_size, start_block, end_block);
        let progress_bar = create_progress_bar(block_range.len() as u64, pb_info);

        // setup tokio stream and determine throttle duration
        let steps = tokio::stream::iter(block_range);
        let duration = Duration::from_millis(1000 / rate_limit);

        let logs = tokio::time::throttle(steps, duration)
            .map(|(from_block, to_block)| {
                // shadow outer
                let provider = provider.clone();
                let pb = progress_bar.clone();
                let events = events.clone();
                async move {
                    // construct our filter for events in the block range
                    let filter = Filter::new()
                        .event_signature(events)
                        .from_block(from_block).to_block(to_block);

                    // fetch all the logs
                    let logs = match provider.get_logs(&filter).await {
                        Ok(logs) => logs,
                        Err(_) => {
                            warn!("Failed to get logs for the block range {}..{}", from_block, to_block);
                            vec![]
                        }
                    };
                    pb.inc(1);
                    drop(provider);
                    logs
                }
            })
            .buffer_unordered(rate_limit) 
            .collect::<Vec<Vec<Log>>>()
            .await;

        // collect and return all the new relevant logs
        let new_logs: Vec<Log> = logs.into_iter().flatten().collect();
        Ok(new_logs)
    }


    // Generate a range of blocks of step size distance
    pub fn get_block_range(step_size: u64, start_block: u64, end_block: u64) -> Vec<(u64, u64)> {
        let block_difference = end_block.saturating_sub(start_block);
        let (_, step_size) = if block_difference < step_size {
            (1, block_difference)
        } else {
            (
                ((block_difference as f64) / (step_size as f64)).ceil() as u64,
                step_size,
            )
        };
        let block_ranges: Vec<(u64, u64)> = (start_block..=end_block)
            .step_by(step_size as usize)
            .map(|from_block| {
                let to_block = (from_block + step_size - 1).min(end_block);
                (from_block, to_block)
            })
            .collect();
        block_ranges
    }
}