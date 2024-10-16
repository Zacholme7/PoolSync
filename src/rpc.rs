use alloy::network::Network;
use alloy::primitives::{Address, FixedBytes};
use alloy::providers::Provider;
use alloy::rpc::types::{Filter, Log};
use alloy::sol_types::SolEvent;
use alloy::transports::Transport;
use anyhow::Result;
use futures::future::join_all;
use log::warn;
use rand::Rng;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};
use tokio::time::{interval, Duration};

use crate::events::*;
use crate::pools::pool_builder;
use crate::pools::pool_structures::balancer_v2_structure::process_balance_data;
use crate::pools::pool_structures::v2_structure::process_sync_data;
use crate::pools::pool_structures::v3_structure::process_tick_data;
use crate::pools::PoolFetcher;
use crate::util::create_progress_bar;
use crate::{Chain, Pool, PoolInfo, PoolType};

/// The number of blocks to query in one call to get_logs
const STEP_SIZE: u64 = 10000;
const MAX_RETRIES: u32 = 5;
const INITIAL_BACKOFF: u64 = 1000; // 1 second

pub struct Rpc;
impl Rpc {
    // Fetch all pool addresses for the protocol
    pub async fn fetch_pool_addrs<P, T, N>(
        start_block: u64,
        end_block: u64,
        provider: Arc<P>,
        fetcher: Arc<dyn PoolFetcher>,
        chain: Chain,
        rate_limit: u64,
    ) -> Option<Vec<Address>>
    where
        P: Provider<T, N> + 'static,
        T: Transport + Clone + 'static,
        N: Network,
    {
        // if there is a range to fetch
        if end_block.saturating_sub(start_block) > 0 {
            // construct set of block ranges to retrieve events over
            let block_range = Rpc::get_block_range(STEP_SIZE, start_block, end_block);
            let pb_info = format!("{} Address Sync", fetcher.pool_type());
            let progress_bar = create_progress_bar(block_range.len() as u64, pb_info);

            // semaphore and interval for rate limiting
            let semaphore = Arc::new(Semaphore::new(rate_limit as usize));
            let interval = Arc::new(Mutex::new(interval(Duration::from_secs_f64(
                1.0 / rate_limit as f64,
            ))));

            // create all of the tasks
            let tasks = block_range.into_iter().map(|(from_block, to_block)| {
                // clone all of the state we need
                let provider = provider.clone();
                let sem = semaphore.clone();
                let pb = progress_bar.clone();
                let interval = interval.clone();
                let fetcher = fetcher.clone();

                // spawn the future
                tokio::spawn(async move {
                    let _permit = sem.acquire().await.unwrap();
                    interval.lock().await.tick().await;
                    // setup filter for pool addresses in block range
                    let filter = Filter::new()
                        .address(fetcher.factory_address(chain))
                        .event(fetcher.pair_created_signature())
                        .from_block(from_block)
                        .to_block(to_block);

                    // fetch the logs with retry for failure
                    let mut retry_count = 0;
                    let mut backoff = 1000; // Initial backoff of 1 second
                    loop {
                        match provider.get_logs(&filter).await {
                            Ok(logs) => {
                                // extract all of the addresses
                                let addresses: Vec<Address> = logs
                                    .iter()
                                    .map(|log| fetcher.log_to_address(&log.inner))
                                    .collect();
                                pb.inc(1);
                                return Ok(addresses); // Return the addresses on success
                            }
                            Err(e) => {
                                // if we have reached the retry count, just return
                                // should fail here, will lead to inconsistent state most likely
                                if retry_count >= MAX_RETRIES {
                                    pb.inc(1);
                                    return Err(e); // Return the error if max retries exceeded
                                }
                                // jitter the retry for best chance at success
                                let jitter = rand::thread_rng().gen_range(0..=100);
                                let sleep_duration = Duration::from_millis(backoff + jitter);
                                tokio::time::sleep(sleep_duration).await;
                                retry_count += 1;
                                backoff *= 2;
                            }
                        }
                    }
                })
            });

            // Wait for all tasks to complete and collect the results
            let results = join_all(tasks).await;

            // Collect all addresses, filtering out errors
            let all_addresses: Vec<Address> = results
                .into_iter()
                .filter_map(|result| result.ok()) // Filter out any JoinError
                .filter_map(|inner_result| inner_result.ok()) // Filter out any Err from our task
                .flatten() // Flatten the Vec<Vec<Address>> into Vec<Address>
                .collect();
            return Some(all_addresses);
        }
        None
    }

    pub async fn populate_pools<P, T, N>(
        pool_addrs: Vec<Address>,
        provider: Arc<P>,
        pool: PoolType,
        fetcher: Arc<dyn PoolFetcher>,
        rate_limit: u64,
    ) -> Result<Vec<Pool>>
    where
        P: Provider<T, N> + 'static,
        T: Transport + Clone + 'static,
        N: Network,
    {
        // data batch size for contract calls
        let batch_size = if pool.is_balancer() { 10 } else { 50 };

        // informational and rate limiting initialization
        let total_tasks = (pool_addrs.len() + batch_size - 1) / batch_size; // Ceiling division
        let progress_bar = create_progress_bar(total_tasks as u64, format!("{} data sync", pool));
        let semaphore = Arc::new(Semaphore::new(rate_limit as usize));
        let interval = Arc::new(tokio::sync::Mutex::new(interval(Duration::from_secs_f64(
            1.0 / rate_limit as f64,
        ))));

        // break the addresses up into chunk
        let addr_chunks: Vec<Vec<Address>> = pool_addrs
            .chunks(batch_size)
            .map(|chunk| chunk.to_vec())
            .collect();

        // construct all of our tasks
        let tasks = addr_chunks.into_iter().map(|chunk| {
            // clone all state to be used in the future
            let provider = provider.clone();
            let sem = semaphore.clone();
            let pb = progress_bar.clone();
            let pool = pool.clone();
            let fetcher = fetcher.clone();
            let interval = interval.clone();

            tokio::spawn(async move {
                // make sure it is our turn to send a req
                let _permit = sem.acquire().await.unwrap();
                interval.lock().await.tick().await;
                let data = fetcher.get_pool_repr();
                let mut retry_count = 0;
                let mut backoff = 1000; // Initial backoff of 1 second

                // keep trying to fetch the result
                loop {
                    // try building pools from this set of addresses
                    match pool_builder::build_pools(
                        provider.clone(),
                        chunk.clone(),
                        pool,
                        data.clone(),
                    )
                    .await
                    {
                        Ok(populated_pools) if !populated_pools.is_empty() => {
                            pb.inc(1);
                            return anyhow::Ok::<Vec<Pool>>(populated_pools);
                        }
                        Err(e) => {
                            println!("Failed to fetche {:?}", e);
                            if retry_count >= MAX_RETRIES {
                                return Ok(Vec::new());
                            }
                            let jitter = rand::thread_rng().gen_range(0..=100);
                            let sleep_duration = Duration::from_millis(backoff + jitter);
                            tokio::time::sleep(sleep_duration).await;
                            retry_count += 1;
                            backoff *= 2; // Exponential backoff
                        }
                        _ => continue,
                    }
                }
            })
        });

        // collect all of the results and process them into a list of pools
        let results = join_all(tasks).await;
        let populated_pools: Vec<Pool> = results
            .into_iter()
            .filter_map(|result| result.ok()) // Filter out any JoinError
            .filter_map(|inner_result| inner_result.ok()) // Filter out any Err from our task
            .flatten() // Flatten the Vec<Vec<Address>> into Vec<Address>
            .collect();
        Ok(populated_pools)
    }

    pub async fn populate_liquidity<P, T, N>(
        start_block: u64,
        end_block: u64,
        pools: &mut [Pool],
        provider: Arc<P>,
        pool_type: PoolType,
    ) -> Result<()>
    where
        P: Provider<T, N> + Sync + 'static,
        T: Transport + Sync + Clone,
        N: Network,
    {
        if pools.is_empty() {
            return Ok(());
        }

        // get the block difference
        let block_difference = end_block.saturating_sub(start_block);
        let address_to_index: HashMap<Address, usize> = pools
            .iter()
            .enumerate()
            .map(|(i, pool)| (pool.address(), i))
            .collect();

        // if we have blocks to get information from
        if block_difference > 0 {
            // create the progress bar

            let mut new_logs: Vec<Log> = Vec::new();

            // Need to fetch different events based on the type of pool we are syncing.
            // This function will allow us to process events since the last sync to maintain proper
            // state
            //
            // For uniswapv2, this function is called when we are finished with a first run sync
            // and we have to catch up with missed state/this is the first time running in a while.
            //
            // For uniswapv3, we need to reconstruct the state from the start of the chain so this
            // will always be called.

            if pool_type.is_v3() {
                // fetch all mint/burn/swap logs
                new_logs.extend(
                    Rpc::fetch_event_logs(
                        start_block,
                        end_block,
                        1500,
                        vec![
                            DataEvents::Burn::SIGNATURE_HASH,
                            DataEvents::Mint::SIGNATURE_HASH,
                        ],
                        provider.clone(),
                        String::from("Tick sync"),
                        10, // have to adjust this
                    )
                    .await
                    .unwrap(),
                );
                if start_block > 10_000_000 {
                    // make sure we dont fetch swap events after initial sync
                    new_logs.extend(
                        Rpc::fetch_event_logs(
                            start_block,
                            end_block,
                            250,
                            vec![
                                PancakeSwapEvents::Swap::SIGNATURE_HASH,
                                DataEvents::Swap::SIGNATURE_HASH,
                            ],
                            provider.clone(),
                            String::from("Swap sync"),
                            10, // have to adjust this
                        )
                        .await
                        .unwrap(),
                    );
                }
            } else if pool_type.is_balancer() {
                if start_block > 10_000_000 {
                    new_logs.extend(
                        Rpc::fetch_event_logs(
                            start_block,
                            end_block,
                            5000,
                            vec![BalancerV2Event::Swap::SIGNATURE_HASH],
                            provider.clone(),
                            String::from("hello world"),
                            10, // have to ajdust some rate limit
                        )
                        .await
                        .unwrap(),
                    );
                }
            } else {
                // v2 sync events. Initially populated from the contract so this is just used when
                // updating state from missed blocks since the last sync
                if start_block > 10_000_000 {
                    new_logs.extend(
                        Rpc::fetch_event_logs(
                            start_block,
                            end_block,
                            100,
                            vec![
                                AerodromeSync::Sync::SIGNATURE_HASH,
                                DataEvents::Sync::SIGNATURE_HASH,
                            ],
                            provider.clone(),
                            String::from("hello world"),
                            10, // have to ajdust some rate limit
                        )
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
    pub async fn fetch_event_logs<T, N, P>(
        start_block: u64,
        end_block: u64,
        step_size: u64,
        events: Vec<FixedBytes<32>>,
        provider: Arc<P>,
        pb_info: String,
        rate_limit: u64,
    ) -> Result<Vec<Log>>
    where
        T: Transport + Clone,
        N: Network,
        P: Provider<T, N> + 'static,
    {
        // generate the block range for the sync and setup progress bar
        let block_range = Rpc::get_block_range(step_size, start_block, end_block);
        let progress_bar = create_progress_bar(block_range.len() as u64, pb_info);

        // semaphore and interval for rate limiting
        let semaphore = Arc::new(Semaphore::new(rate_limit as usize));
        let interval = Arc::new(Mutex::new(interval(Duration::from_secs_f64(
            1.0 / rate_limit as f64,
        ))));

        // generate all the tasks
        let tasks = block_range.into_iter().map(|(from_block, to_block)| {
            let provider = provider.clone();
            let events = events.clone();
            let sem = semaphore.clone();
            let pb = progress_bar.clone();
            let interval = interval.clone();

            tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();
                interval.lock().await.tick().await;

                let filter = Filter::new()
                    .event_signature(events)
                    .from_block(from_block)
                    .to_block(to_block);

                let result = provider.get_logs(&filter).await;
                pb.inc(1);

                match result {
                    Ok(logs) => logs,
                    Err(_) => {
                        warn!(
                            "Failed to get logs for the block range {}..{}",
                            from_block, to_block
                        );
                        vec![]
                    }
                }
            })
        });

        // fetch all the logs
        let results = join_all(tasks).await;
        let new_logs: Vec<Log> = results
            .into_iter()
            .filter_map(|r| r.ok())
            .flatten()
            .collect();

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
