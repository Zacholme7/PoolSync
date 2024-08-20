use alloy::network::Network;
use alloy::primitives::Address;
use alloy::providers::Provider;
use alloy::rpc::types::Filter;
use alloy::rpc::types::Log;
use alloy::sol;
use alloy::transports::Transport;
use alloy_sol_types::SolEvent;
use anyhow::Result;
use futures::stream;
use futures::stream::StreamExt;
use indicatif::ProgressBar;
use rand::Rng;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;

use crate::pools::pool_structures::v3_structure::process_tick_data;
use crate::pools::pool_structures::v2_structure::process_sync_data;
use crate::pools::pool_structures::balancer_v2_structure::process_balance_data;
use crate::pools::pool_builder;
use crate::pools::PoolFetcher;
use crate::util::create_progress_bar;
use crate::Chain;
use crate::Pool;
use crate::PoolInfo;
use crate::PoolType;


/// The number of blocks to query in one call to get_logs
const STEP_SIZE: u64 = 10_000;
const MAX_RETRIES: u32 = 5;
const INITIAL_BACKOFF: u64 = 1000; // 1 second

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    contract AerodromeSync {
        event Sync(uint256 reserve0, uint256 reserve1);
    }
);

sol! (
    #[derive(Debug)]
    contract BalancerV2Event {
        event PoolBalanceChanged(
            bytes32 indexed poolId,
            address indexed liquidityProvider,
            address[] tokens,
            int256[] deltas,
            uint256[] protocolFeeAmounts
        );
    }
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    contract PancakeSwap {
        event Swap(
            address indexed sender,
            address indexed recipient,
            int256 amount0,
            int256 amount1,
            uint160 sqrtPriceX96,
            uint128 liquidity,
            int24 tick,
            uint128 protocolFeesToken0,
            uint128 protocolFeesToken1
        );
    }
);


sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    contract DataEvents {
        event Sync(uint112 reserve0, uint112 reserve1);
        event Swap(address indexed sender, address indexed recipient, int256 amount0, int256 amount1, uint160 sqrtPriceX96, uint128 liquidity, int24 tick);
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

        let rate_limiter = Arc::new(Semaphore::new(100 as usize));

        // Map all the addresses into chunks the contract can handle
        let addr_chunks: Vec<Vec<Address>> =
            pool_addrs.chunks(BATCH_SIZE).map(|chunk| chunk.to_vec()).collect();

        let results = stream::iter(addr_chunks)
            .map(|chunk| {
                let provider = provider.clone();
                let progress_bar = progress_bar.clone();
                let pool = pool.clone();
                let rate_limiter = rate_limiter.clone();
                let fetcher = fetcher.clone();

                async move {
                    let mut retry_count = 0;
                    let mut backoff = INITIAL_BACKOFF;
                    let _permit = rate_limiter.acquire().await.unwrap();
                    let data = fetcher.get_pool_repr();

                    loop {
                        match pool_builder::build_pools(provider.clone(), chunk.clone(), pool, data.clone()).await
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
                        Rpc::fetch_balance_logs(start_block, end_block, provider.clone(), pool_type)
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


    pub async fn fetch_balance_logs<P, T, N>(
        start_block: u64,
        end_block: u64,
        provider: Arc<P>,
        pool_type: PoolType,
    ) -> Result<Vec<Log>>
    where
        P: Provider<T, N> + 'static,
        T: Transport + Clone + 'static,
        N: Network,
    {
        let mint_burn_range = Rpc::get_block_range(5000, start_block, end_block);
        let info = format!("{} balance sync", pool_type);
        let progress_bar = create_progress_bar(mint_burn_range.len() as u64, info);
        let logs = stream::iter(mint_burn_range)
            .map(|(from_block, to_block)| {
                let provider = provider.clone();
                let pb = progress_bar.clone();
                async move {
                    let filter = Filter::new()
                        .event_signature(vec![
                            BalancerV2Event::PoolBalanceChanged::SIGNATURE_HASH
                        ])
                        .from_block(from_block)
                        .to_block(to_block);
                    let logs = provider.get_logs(&filter).await.unwrap();
                    pb.inc(1);
                    drop(provider);
                    logs
                }
            })
            .buffer_unordered(100) // Allow some buffering for smoother operation
            .collect::<Vec<Vec<Log>>>()
            .await;
        let new_logs: Vec<Log> = logs.into_iter().flatten().collect();
        Ok(new_logs)
    }

    pub async fn fetch_tick_logs<P, T, N>(
        start_block: u64,
        end_block: u64,
        provider: Arc<P>,
        pool_type: PoolType,
    ) -> Result<Vec<Log>>
    where
        P: Provider<T, N> + 'static,
        T: Transport + Clone + 'static,
        N: Network,
    {
        let mint_burn_range = Rpc::get_block_range(5000, start_block, end_block);
        let info = format!("{} tick sync", pool_type);
        let progress_bar = create_progress_bar(mint_burn_range.len() as u64, info);
        let logs = stream::iter(mint_burn_range)
            .map(|(from_block, to_block)| {
                let provider = provider.clone();
                let pb = progress_bar.clone();
                async move {
                    let filter = Filter::new()
                        .event_signature(vec![
                            DataEvents::Burn::SIGNATURE_HASH,
                            DataEvents::Mint::SIGNATURE_HASH,
                        ])
                        .from_block(from_block)
                        .to_block(to_block);
                    let logs = provider.get_logs(&filter).await.unwrap();
                    pb.inc(1);
                    drop(provider);
                    logs
                }
            })
            .buffer_unordered(100) // Allow some buffering for smoother operation
            .collect::<Vec<Vec<Log>>>()
            .await;
        let new_logs: Vec<Log> = logs.into_iter().flatten().collect();
        Ok(new_logs)
    }

    pub async fn fetch_swap_logs<P, T, N>(
        start_block: u64,
        end_block: u64,
        provider: Arc<P>,
        pool_type: PoolType
    ) -> Result<Vec<Log>>
    where
        P: Provider<T, N> + 'static,
        T: Transport + Clone + 'static,
        N: Network,
    {
        let swap_range = Rpc::get_block_range(500, start_block, end_block);
        let info = format!("{} swap sync", pool_type);
        let progress_bar = create_progress_bar(swap_range.len() as u64, info);
        let logs = stream::iter(swap_range)
            .map(|(from_block, to_block)| {
                let provider = provider.clone();
                let pb = progress_bar.clone();
                async move {
                    let filter = Filter::new()
                        .event_signature(vec![
                            PancakeSwap::Swap::SIGNATURE_HASH,
                            DataEvents::Swap::SIGNATURE_HASH
                        ])
                        .from_block(from_block)
                        .to_block(to_block);
                    let logs = provider.get_logs(&filter).await.unwrap();
                    pb.inc(1);
                    drop(provider);
                    logs
                }
            })
            .buffer_unordered(100) // Allow some buffering for smoother operation
            .collect::<Vec<Vec<Log>>>()
            .await;
        let new_logs: Vec<Log> = logs.into_iter().flatten().collect();
        Ok(new_logs)
    }

    pub async fn fetch_sync_logs<P, T, N>(
        start_block: u64,
        end_block: u64,
        provider: Arc<P>,
        pool_type: PoolType
    ) -> Result<Vec<Log>>
    where
        P: Provider<T, N> + 'static,
        T: Transport + Clone + 'static,
        N: Network,
    {
        let sync_range = Rpc::get_block_range(200, start_block, end_block);
        let info = format!("{} sync sync", pool_type);
        let progress_bar = create_progress_bar(sync_range.len() as u64, info);
        let logs = stream::iter(sync_range)
            .map(|(from_block, to_block)| {
                let provider = provider.clone();
                let progress_bar = progress_bar.clone();
                async move {
                    let filter = Filter::new()
                        .event_signature(vec![
                            AerodromeSync::Sync::SIGNATURE_HASH,
                            DataEvents::Sync::SIGNATURE_HASH
                        ])
                        .from_block(from_block)
                        .to_block(to_block);
                    let logs = provider.get_logs(&filter).await.unwrap();
                    drop(provider);
                    progress_bar.inc(1);
                    logs
                }
            })
            .buffer_unordered(1000) // Allow some buffering for smoother operation
            .collect::<Vec<Vec<Log>>>()
            .await;
        let new_logs: Vec<Log> = logs.into_iter().flatten().collect();
        Ok(new_logs)
    }

    pub fn get_block_range(step_size: u64, start_block: u64, end_block: u64) -> Vec<(u64, u64)> {
        let block_difference = end_block.saturating_sub(start_block);
        let (total_steps, step_size) = if block_difference < step_size {
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
