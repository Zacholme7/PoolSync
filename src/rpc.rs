use alloy::network::Network;
use alloy::primitives::Address;
use alloy::providers::Provider;
use alloy::rpc::types::{Filter, Log};
use alloy::sol_types::SolEvent;
use alloy::transports::Transport;
use anyhow::anyhow;
use anyhow::Result;
use futures::StreamExt;
use indicatif::ProgressBar;
use log::info;
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

// Retry constants
const MAX_RETRIES: u32 = 5;
const INITIAL_BACKOFF: u64 = 1000; // 1 second

// Define event configurations
#[derive(Debug)]
struct EventConfig {
    events: &'static [&'static str],
    step_size: u64,
    description: &'static str,
    requires_initial_sync: bool,
}

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
    ) -> Result<Vec<Address>>
    where
        P: Provider<T, N> + 'static,
        T: Transport + Clone + 'static,
        N: Network,
    {
        // fetch all of the logs
        let filter = Filter::new()
            .address(fetcher.factory_address(chain))
            .event(fetcher.pair_created_signature());

        let step_size: u64 = 10000;
        let num_tasks = end_block / step_size;
        let pb_info = format!(
            "{} Address Sync. Block range {}-{}",
            fetcher.pool_type(),
            start_block,
            end_block
        );
        let progress_bar = Arc::new(create_progress_bar(num_tasks, pb_info));

        // fetch all of the logs
        let logs = Rpc::fetch_event_logs(
            start_block,
            end_block,
            10000,
            provider,
            rate_limit,
            progress_bar,
            filter,
        )
        .await?;

        // extract the addresses from the logs
        let addresses: Vec<Address> = logs
            .iter()
            .map(|log| fetcher.log_to_address(&log.inner))
            .collect();
        anyhow::Ok(addresses)
    }

    pub async fn populate_pools<P, T, N>(
        pool_addrs: Vec<Address>,
        provider: Arc<P>,
        pool: PoolType,
        fetcher: Arc<dyn PoolFetcher>,
        rate_limit: u64,
        chain: Chain
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

        let mut stream = futures::stream::iter(addr_chunks.into_iter().map(|chunk| {
            let provider = provider.clone();
            let sem = semaphore.clone();
            let pb = progress_bar.clone();
            let fetcher = fetcher.clone();
            let interval = interval.clone();
            let data = fetcher.get_pool_repr();

            async move {
                let _permit = sem.acquire().await.unwrap();
                interval.lock().await.tick().await;
                let mut retry_count = 0;
                let mut backoff = 1000; // Initial backoff of 1 second
                loop {
                    // try building pools from this set of addresses
                    match pool_builder::build_pools(
                        &provider,
                        chunk.clone(),
                        pool,
                        data.clone(),
                        chain
                    )
                    .await
                    {
                        Ok(populated_pools) if !populated_pools.is_empty() => {
                            pb.inc(1);
                            drop(provider);
                            return anyhow::Ok::<Vec<Pool>>(populated_pools);
                        }
                        Err(e) => {
                            if retry_count >= MAX_RETRIES {
                                info!("Failed to populate pools data: {}", e);
                                drop(provider);
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
            }
        }))
        .buffer_unordered(rate_limit as usize);

        let mut all_pools = Vec::new();

        while let Some(pool_res) = stream.next().await {
            match pool_res {
                Ok(pool) => all_pools.extend(pool),
                Err(e) => return Err(e),
            }
        }

        Ok(all_pools)
    }

    pub async fn populate_liquidity<P, T, N>(
        start_block: u64,
        end_block: u64,
        pools: &mut [Pool],
        provider: Arc<P>,
        pool_type: PoolType,
        rate_limit: u64,
        is_initial_sync: bool,
    ) -> anyhow::Result<()>
    where
        P: Provider<T, N> + Sync + 'static,
        T: Transport + Sync + Clone,
        N: Network,
    {
        if pools.is_empty() {
            return anyhow::Ok(());
        }

        let address_to_index: HashMap<Address, usize> = pools
            .iter()
            .enumerate()
            .map(|(i, pool)| (pool.address(), i))
            .collect();

        let batch_size = 1_000_000;
        let mut current_block = start_block;

        // get the configuration for this sync and config we should sync
        let config = Rpc::get_event_config(pool_type, is_initial_sync);
        if is_initial_sync && config.requires_initial_sync {
            return anyhow::Ok(());
        }

        // construct the progress bar
        let num_tasks = (end_block - start_block) / config.step_size;
        let pb_info = format!(
            "{} {}. Block range {}-{}",
            pool_type, config.description, current_block, end_block
        );
        let progress_bar = Arc::new(create_progress_bar(num_tasks, pb_info));

        // sync in batches
        while current_block <= end_block {
            let batch_end = (current_block + batch_size).min(end_block);

            let logs = Rpc::fetch_logs_for_config(
                &config,
                current_block,
                batch_end,
                provider.clone(),
                progress_bar.clone(),
                rate_limit,
            )
            .await?;

            // create pb for block processing
            let processing_pb_info = format!(
                "Processing logs batch for blocks {}-{}",
                start_block, end_block
            );
            let processing_progress_bar =
                create_progress_bar(logs.len().try_into().unwrap(), processing_pb_info);

            // Process logs immediately after fetching
            let mut ordered_logs: BTreeMap<u64, Vec<Log>> = BTreeMap::new();
            for log in logs {
                if let Some(block_number) = log.block_number {
                    ordered_logs.entry(block_number).or_default().push(log);
                }
            }

            // Process logs in order
            for (_, log_group) in ordered_logs {
                for log in log_group {
                    let address = log.address();
                    if let Some(&index) = address_to_index.get(&address) {
                        if let Some(pool) = pools.get_mut(index) {
                            if pool_type.is_v3() {
                                process_tick_data(
                                    pool.get_v3_mut().unwrap(),
                                    log,
                                    pool_type,
                                    is_initial_sync,
                                );
                            } else if pool_type.is_balancer() {
                                process_balance_data(pool.get_balancer_mut().unwrap(), log);
                            } else {
                                process_sync_data(pool.get_v2_mut().unwrap(), log, pool_type);
                            }
                        }
                    }
                    processing_progress_bar.inc(1);
                }
            }

            processing_progress_bar.finish_and_clear();
            current_block = batch_end + 1;
        }
        anyhow::Ok(())
    }

    pub async fn fetch_event_logs<T, N, P>(
        start_block: u64,
        end_block: u64,
        step_size: u64,
        provider: Arc<P>,
        rate_limit: u64,
        progress_bar: Arc<ProgressBar>,
        filter: Filter,
    ) -> anyhow::Result<Vec<Log>>
    where
        T: Transport + Clone,
        N: Network,
        P: Provider<T, N> + 'static,
    {
        // generate the block range for the sync and setup progress bar
        let block_range = Rpc::get_block_range(step_size, start_block, end_block);

        // semaphore and interval for rate limiting
        let semaphore = Arc::new(Semaphore::new(rate_limit as usize));
        let interval = Arc::new(Mutex::new(interval(Duration::from_secs_f64(
            1.0 / rate_limit as f64,
        ))));

        // Create a stream of futures
        let mut stream =
            futures::stream::iter(block_range.into_iter().map(|(from_block, to_block)| {
                let provider = provider.clone();
                let sem = semaphore.clone();
                let pb = progress_bar.clone();
                let interval = interval.clone();
                let filter = filter.clone();

                async move {
                    let _permit = sem.acquire().await.unwrap();
                    interval.lock().await.tick().await;

                    let filter = filter.from_block(from_block).to_block(to_block);
                    let logs = Rpc::get_logs_with_retry(provider, &filter).await;
                    if logs.is_ok() {
                        pb.inc(1);
                    }
                    logs
                }
            }))
            .buffer_unordered(rate_limit as usize); // Process up to rate_limit tasks concurrently

        let mut all_logs = Vec::new();

        // Process results as they complete
        while let Some(result) = stream.next().await {
            match result {
                Ok(logs) => all_logs.extend(logs),
                Err(e) => return Err(e),
            }
        }

        Ok(all_logs)
    }

    // Given a config and a range, fetch all the logs for it
    // This is a top level call which will delegate to individual fetching
    // functions to get the logs and to ensure retries on failure
    async fn fetch_logs_for_config<P, T, N>(
        config: &EventConfig,
        start_block: u64,
        end_block: u64,
        provider: Arc<P>,
        progress_bar: Arc<ProgressBar>,
        rate_limit: u64,
    ) -> Result<Vec<Log>>
    where
        P: Provider<T, N> + 'static,
        T: Transport + Clone + 'static,
        N: Network,
    {
        let filter = Filter::new().events(config.events.iter().copied());
        Rpc::fetch_event_logs(
            start_block,
            end_block,
            config.step_size,
            provider,
            rate_limit,
            progress_bar,
            filter,
        )
        .await
    }

    // Fetch logs with retry functionality
    async fn get_logs_with_retry<P, T, N>(
        provider: Arc<P>,
        filter: &Filter,
    ) -> anyhow::Result<Vec<Log>>
    where
        P: Provider<T, N> + 'static,
        T: Transport + Clone + 'static,
        N: Network,
    {
        let mut retry_count = 0;
        let mut backoff = INITIAL_BACKOFF;

        loop {
            match provider.get_logs(filter).await {
                Ok(logs) => {
                    return anyhow::Ok(logs);
                }
                Err(e) => {
                    if retry_count >= MAX_RETRIES {
                        return Err(anyhow!(e));
                    }
                    let jitter = rand::thread_rng().gen_range(0..=100);
                    let sleep_duration = Duration::from_millis(backoff + jitter);
                    tokio::time::sleep(sleep_duration).await;
                    retry_count += 1;
                    backoff *= 2;
                }
            }
        }
    }

    fn get_event_config(pool_type: PoolType, is_initial_sync: bool) -> EventConfig {
        match pool_type {
            pt if pt.is_v3() => {
                if is_initial_sync {
                    EventConfig {
                        events: &[DataEvents::Mint::SIGNATURE, DataEvents::Burn::SIGNATURE],
                        step_size: 1500,
                        description: "Tick sync",
                        requires_initial_sync: false, // Always fetch these
                    }
                } else {
                    EventConfig {
                        events: &[
                            DataEvents::Mint::SIGNATURE,
                            DataEvents::Burn::SIGNATURE,
                            DataEvents::Swap::SIGNATURE,
                        ],
                        step_size: 50,
                        description: "Full sync",
                        requires_initial_sync: true, // Always fetch these
                    }
                }
            }
            pt if pt.is_balancer() => EventConfig {
                events: &[BalancerV2Event::Swap::SIGNATURE],
                step_size: 5000,
                description: "Swap Sync",
                requires_initial_sync: true,
            },
            _ => EventConfig {
                events: &[AerodromeSync::Sync::SIGNATURE, DataEvents::Sync::SIGNATURE],
                step_size: 250,
                description: "Reserve Sync",
                requires_initial_sync: true,
            },
        }
    }

    // Generate a range of blocks of step size distance
    pub fn get_block_range(step_size: u64, start_block: u64, end_block: u64) -> Vec<(u64, u64)> {
        if start_block == end_block {
            return vec![(start_block, end_block)];
        }

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
