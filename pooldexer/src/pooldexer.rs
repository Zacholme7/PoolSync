use crate::pool_type::PoolType;
use alloy_primitives::{Address, FixedBytes, Log as PrimitiveLog};
use alloy_provider::{Provider, ProviderBuilder, RootProvider};
use alloy_rpc_types::{Filter, Log as RpcLog};
use anyhow::{Context, Result};
use futures::future::try_join_all;
use futures::stream;
use futures::stream::StreamExt;
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::sync::Arc;
use tracing::info;

pub struct PooldexerConfig {
    el_url: String,
    chain: Chain,
    pools: Vec<PoolType>,
}

#[derive(Debug, Copy, Clone)]
pub enum Chain {
    Ethereum,
    Base,
}

pub struct Pooldexer {
    provider: Arc<RootProvider>,
    last_processed_block: u64,
    pools: Vec<PoolType>,
    chain: Chain,
}

const BATCH_SIZE: u64 = 5000;
const GROUP_SIZE: u64 = 50;

impl Pooldexer {
    pub fn new(config: PooldexerConfig) -> Result<Self> {
        let provider = Arc::new(ProviderBuilder::default().connect_http(config.el_url.parse()?));

        Ok(Self {
            provider,
            last_processed_block: 0,
            pools: config.pools,
            chain: config.chain,
        })
    }

    /// Long running task to index all past and future pools from the chain
    pub async fn index_pools(&mut self) -> Result<()> {
        loop {
            match self.try_sync().await {
                Ok(_) => unreachable!("Indexing should never finish successfully"),
                Err(_) => todo!(),
            }
        }
    }

    async fn try_sync(&mut self) -> Result<()> {
        // Sync to the tip for all pools in a round robin fashion
        info!("Starting a historical sync");
        self.historical_sync().await?;

        // Transition into a live sync. This is essentially just a historical sync with a block
        // history length of 1
        info!("Transitioning into live sync");
        self.live_sync().await?;

        // we should never reach here
        todo!();
    }

    // On a fresh start of after a period of blocks has passed while not running, we need to
    // historical sync the missed parts of the chain and catch back up to the tip
    async fn historical_sync(&self) -> Result<()> {
        // Define a block detal to represent a total value of how far we are from the tip of the
        // chain. Historical syncing large block histories takes time and compute. While we may
        // sync one pool up to the tip, by the time we sync others we will have diverged from the
        // tip on the initial sync. By implemented this staged pass upto a recent tip, we can
        // converge on the tip over time.
        //
        // Note: we do this in a round robin fashion so we do not blow up a node with rpc requests
        let block_delta = 0;

        while block_delta != 0 {
            // Sync each pool up to local tip
            let current_end = self.provider.get_block_number().await?;
            for pool in &self.pools {
                // Get the local start for each
                let start_block = 0; // self.db.get_last_processed_block(pool);

                // Step 1: Fetch new pools addresses
                let addresses = self
                    .fetch_pool_addresses(start_block, current_end, pool)
                    .await?;

                // Step 2: Populate pool information (Decimals, Token names, snapshot liquidity)
                self.populate_pool_info(start_block, current_end, &addresses, pool)
                    .await?;

                // Step 3: Populate liquidity information for pools where we have to recreate the
                // changes liquidity from scratch (V3...)
                self.populate_pool_liquidity(start_block, current_end, &addresses, pool)
                    .await?;
            }
        }

        // We have synced up to the tip
        Ok(())
    }

    async fn fetch_pool_addresses(
        &self,
        start_block: u64,
        end_block: u64,
        pool_type: &PoolType,
    ) -> Result<Vec<Address>> {
        // Setup a filter targeting the factor address and the event signature that corresponds to
        // newly created pools
        let filter = Filter::new()
            .address(pool_type.factory_address(self.chain))
            .event_signature(pool_type.pair_created_signature());

        // Fetch all of the logs and extract out the pool address
        let logs = Arc::new(RwLock::new(Vec::new()));
        let logs_ref = logs.clone();
        self.fetch_logs_from_range(
            start_block,
            end_block,
            filter,
            |fetch_logs: Vec<RpcLog>| async move {
                logs_ref.write().extend(fetch_logs.clone());
                Ok(())
            },
        )
        .await;

        let addresses = logs
            .read()
            .iter()
            .map(|log| pool_type.log_to_address(log))
            .collect();

        // save addresses to the database
        // seld.db.save_addresses(addresses, pool_type);

        Ok(addresses)
    }

    async fn populate_pool_info(
        &self,
        start_block: u64,
        end_block: u64,
        addresses: &[Address],
        pool_type: &PoolType,
    ) -> Result<()> {
        // Build a set of futures for fetching pool data. We are not building the actual pools
        // here, just persisting the data into the db
        let futures: Vec<_> = addresses
            .chunks(10 as usize)
            .map(|addr_chunk| async move {
                //build_pools(addr_chunk, pool_type, self.provider.clone(), block_num).await
                todo!()
            })
            .collect();

        // Sequentially process the futures w/ controlled concurrency
        //stream::iter(futures).buffer_unordered(100).collect::<_>().await;

        // At this point, we have saved all new pool addresses and their info, and snapshot
        // liquidity into the database
        Ok(())
    }

    async fn populate_pool_liquidity(
        &self,
        start_block: u64,
        end_block: u64,
        addresses: &[Address],
        pool_type: &PoolType,
    ) -> Result<()> {
        //let filter = pool_type.liqiudity_filter();
        let filter = Filter::new();

        self.fetch_logs_from_range(start_block, end_block, filter, Self::process_tick_data)
            .await;

        Ok(())
    }

    async fn process_tick_data(logs: Vec<RpcLog>) -> Result<()> {
        todo!()
    }

    async fn live_sync(&self) -> Result<()> {
        // We do not care what the block is, just that we got a new one, historical sync will take
        // care of figuring out the details

        panic!("This should never end");
    }

    async fn fetch_logs_from_range(
        &self,
        start_block: u64,
        end_block: u64,
        event_filter: Filter,
        processor: impl AsyncFnOnce(Vec<RpcLog>) -> Result<()> + Clone,
    ) {
        // Here, we have a start..end block that we need to sync the logs from. This range gets
        // broken up into individual ranges of BATCH_SIZE where the logs are fetches from. The
        // individual ranges are further broken up into a set of batches that are sequentually
        // processes. This makes it so that we do not blow up the node with requests and can
        // optionally run a processing funtion against the logs

        // Chunk the start and end block range into a set of ranges of size BATCH_SIZE
        // and construct a future to fetch the logs in each range
        let mut tasks: Vec<_> = (start_block..=end_block)
            .step_by(BATCH_SIZE as usize)
            .map(|start| {
                let (start, end) = (start, std::cmp::min(start + BATCH_SIZE - 1, end_block));
                self.fetch_logs(start, end, event_filter.clone())
            })
            .collect();

        // Further chunk the block ranges into groups where each group covers 500k blocks, so
        // there are 50 tasks per group. BATCH_SIZE * 50 = 500k
        let mut task_groups = Vec::new();
        while !tasks.is_empty() {
            // Drain takes elements from the original vector, moving them to a new vector
            // take up to chunk_size elements (or whatever is left if less than chunk_size)
            let chunk: Vec<_> = tasks
                .drain(..tasks.len().min(GROUP_SIZE as usize))
                .collect();
            task_groups.push(chunk);
        }

        for group in task_groups.into_iter() {
            // Await all of the futures.
            let event_logs: Vec<Vec<RpcLog>> = try_join_all(group).await.unwrap();
            let event_logs: Vec<RpcLog> = event_logs.into_iter().flatten().collect();

            // The futures may join out of order block wise. The individual events within the
            // block retain their tx ordering. Due to this, we can reassemble
            // back into blocks and be confident the order is correct
            let mut ordered_event_logs: BTreeMap<u64, Vec<RpcLog>> = BTreeMap::new();
            for log in event_logs {
                let block_num = log
                    .block_number
                    .ok_or("Log is missing block number")
                    .unwrap();
                ordered_event_logs.entry(block_num).or_default().push(log);
            }
            let ordered_event_logs: Vec<RpcLog> =
                ordered_event_logs.into_values().flatten().collect();

            // Perform extra log processing. This is a workaround for being able to support
            let processor = processor.clone();
            processor(ordered_event_logs).await.unwrap();
        }
    }

    fn fetch_logs(
        &self,
        start_block: u64,
        end_block: u64,
        event_filter: Filter,
    ) -> impl Future<Output = anyhow::Result<Vec<RpcLog>>> {
        let filter = event_filter.from_block(start_block).to_block(end_block);
        let provider = self.provider.clone();
        async move {
            // Provider has build in retry mechanism, will retry up to TODO times
            provider
                .get_logs(&filter)
                .await
                .context("Failed to fetch logs")
        }
    }
}

/*
    // Once caught up with the chain, start live sync which will stream in live blocks from the
    // network. The events will be processed and duties will be created in response to network
    // actions
    #[instrument(skip(self, contract_address))]
    async fn live_sync(&mut self, contract_address: Address) -> Result<(), ExecutionError> {
        info!("Network up to sync..");
        info!("Current state");
        info!(?contract_address, "Starting live sync");

        metrics::set_gauge(&metrics::EXECUTION_SYNC_STATUS, 1);

        loop {
            // Try to subscribe to a block stream
            let stream = match self.ws_client.subscribe_blocks().await {
                Ok(sub) => {
                    info!("Successfully subscribed to block stream");
                    Some(sub.into_stream())
                }
                Err(e) => {
                    return Err(ExecutionError::WsError(format!(
                        "Failed to subscribe to block stream: {e}"
                    )));
                }
            };

            // If we have a connection, continuously stream in blocks
            if let Some(mut stream) = stream {
                while let Some(block_header) = stream.next().await {
                    // Block we are interested in is the current block number - follow distance
                    let relevant_block = block_header.number - FOLLOW_DISTANCE;
                    debug!(
                        block_number = block_header.number,
                        relevant_block, "Processing new block"
                    );

                    metrics::set_gauge(
                        &metrics::EXECUTION_CURRENT_BLOCK,
                        block_header.number as i64,
                    );

                    let logs = self
                        .fetch_logs(
                            relevant_block,
                            relevant_block,
                            contract_address,
                            SSV_EVENTS.clone(),
                        )
                        .await?;

                    info!(
                        log_count = logs.len(),
                        "Processing events from block {}", relevant_block
                    );

                    // process the logs and update the last block we have recorded
                    self.event_processor.process_logs(logs, true);
                    self.event_processor
                        .db
                        .processed_block(relevant_block)
                        .expect("Failed to update last processed block number");
                }
            }

            // If we get here, the stream ended (likely due to disconnect)
            error!("WebSocket stream ended, reconnecting...");
            metrics::set_gauge(&metrics::EXECUTION_SYNC_STATUS, 0);
        }
    }
*/
