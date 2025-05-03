use crate::errors::PoolSyncError;
use crate::onchain::{AerodromeSync, DataEvents};
use crate::{Chain, Pool, PoolInfo, PoolType, Syncer};
use alloy_network::Ethereum;
use alloy_primitives::Address;
use alloy_provider::{Provider, ProviderBuilder, RootProvider};
use alloy_rpc_types::{Filter, Log};
use alloy_sol_types::SolEvent;
use async_trait::async_trait;
use futures::{stream, StreamExt};
use pool_builder::build_pools;
use pool_fetchers::PoolFetcher;
use std::collections::{BTreeMap, HashMap};
use std::future::Future;
use std::sync::Arc;
use tracing::{debug, error};

pub mod pool_builder;
pub mod pool_fetchers;

// Batch steps size for address syncing
const ADDRESS_BATCH_SIZE: u64 = 10_000;
const INFO_BATCH_SIZE: u64 = 50;
const RETRY_LIMIT: usize = 10;

// Sync pools via rpc
pub(crate) struct RpcSyncer {
    provider: Arc<RootProvider>,
    chain: Chain,
}

    async fn populate_liquidity(
        &self,
        pools: &mut HashMap<Address, Pool>,
        pool_type: &PoolType,
        start_block: u64,
        end_block: u64,
        is_initial_sync: bool,
    ) -> Result<Vec<Address>, PoolSyncError> {
        let filter = if pool_type.is_v2() {
            Filter::new().events([AerodromeSync::Sync::SIGNATURE, DataEvents::Sync::SIGNATURE])
        } else if pool_type.is_v3() {
            Filter::new().events([
                DataEvents::Mint::SIGNATURE,
                DataEvents::Burn::SIGNATURE,
                DataEvents::Swap::SIGNATURE,
            ])
        } else {
            todo!()
        };

        // Chunk up block range into fetching futures and join them all
        let tasks = self.build_fetch_tasks(start_block, end_block, filter);

        // Buffer the futures to not overwhelm the provider
        let logs: Vec<_> = stream::iter(tasks).buffer_unordered(100).collect().await;
        let logs: Vec<Log> = logs
            .into_iter()
            .filter_map(|result| match result {
                Ok(logs) => Some(logs),
                Err(e) => {
                    error!("Fetching failed: {}", e);
                    None
                }
            })
            .flatten()
            .collect();

        let mut ordered_logs: BTreeMap<u64, Vec<Log>> = BTreeMap::new();
        for log in logs {
            if let Some(block_number) = log.block_number {
                ordered_logs.entry(block_number).or_default().push(log);
            }
        }

        // Process all of the logs
        let mut touched_pools = Vec::new();
        for (_, log_group) in ordered_logs {
            for log in log_group {
                let address = log.address();
                touched_pools.push(address);
                if let Some(pool) = pools.get_mut(&address) {
                    if pool_type.is_v3() {
                        let pool = pool.get_v3_mut().unwrap();
                        pool.process_tick_data(log, pool_type, is_initial_sync);
                    } else if pool_type.is_balancer() {
                        //process_balance_data(pool.get_balancer_mut().unwrap(), log);
                    } else {
                        let pool = pool.get_v2_mut().unwrap();
                        pool.process_sync_data(log, pool_type);
                    }
                }
            }
        }
        Ok(touched_pools)
    }

    async fn block_number(&self) -> Result<u64, PoolSyncError> {
        self.provider
            .get_block_number()
            .await
            .map_err(|_| PoolSyncError::ProviderError("failed to get block".to_string()))
    }
}

impl RpcSyncer {
    // Construct a new Rpc Syncer to sync pools via RPC
    pub fn new(chain: Chain) -> Result<Self, PoolSyncError> {
        let endpoint = std::env::var("ARCHIVE").map_err(|_e| PoolSyncError::EndpointNotSet)?;

        let provider = Arc::new(
            ProviderBuilder::<_, _, Ethereum>::default().on_http(
                endpoint
                    .parse()
                    .map_err(|_e| PoolSyncError::ParseEndpointError)?,
            ),
        );
        Ok(Self { provider, chain })
    }

    // Fetch logs from start_block..end_block for the provided filter
    fn fetch_logs(
        &self,
        start_block: u64,
        end_block: u64,
        filter: Filter,
    ) -> impl Future<Output = Result<Vec<Log>, PoolSyncError>> {
        let filter = filter.from_block(start_block).to_block(end_block);
        let client = self.provider.clone();
        async move {
            let mut fetch_cnt = 0;
            loop {
                // Fetch the logs w/ a backoff retry
                match client.get_logs(&filter).await {
                    Ok(logs) => {
                        debug!("Fetched logs from block {} to {}", start_block, end_block);
                        return Ok(logs);
                    }
                    Err(_) => {
                        fetch_cnt += 1;
                        if fetch_cnt == RETRY_LIMIT {
                            return Err(PoolSyncError::ProviderError(
                                "Reached rety limit".to_string(),
                            ));
                        }

                        // Jitter for some retry sleep duration
                        let jitter = fastrand::u64(0..=1000);
                        tokio::time::sleep(std::time::Duration::from_millis(jitter)).await
                    }
                }
            }
        }
    }

}
