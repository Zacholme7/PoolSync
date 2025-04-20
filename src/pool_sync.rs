//! PoolSync Core Implementation
//!
//! This module contains the core functionality for synchronizing pools across different
//! blockchain networks and protocols. It includes the main `PoolSync` struct and its
//! associated methods for configuring and executing the synchronization process.

use crate::errors::PoolSyncError;
use std::collections::HashMap;
use std::sync::Arc;

use crate::builder::PoolSyncBuilder;
use crate::pools::*;
use crate::{PoolType, Syncer};
use tracing::info;

/// The main struct for pool synchronization
pub struct PoolSync {
    /// Map of pool types to their fetcher implementations
    fetchers: HashMap<PoolType, Arc<dyn PoolFetcher>>,
    /// Underlying syncer
    syncer: Box<dyn Syncer>,
    // Hold the database
    // todo!()
}

impl PoolSync {
    // Construct a new instance of PoolSync
    pub(crate) fn new(
        fetchers: HashMap<PoolType, Arc<dyn PoolFetcher>>,
        syncer: Box<dyn Syncer>,
    ) -> Self {
        Self { fetchers, syncer }
    }

    /// Construct a new builder to configure sync parameters
    pub fn builder() -> PoolSyncBuilder {
        PoolSyncBuilder::default()
    }

    /// Sync all of the pools from the chain
    pub async fn sync_pools(&self) -> Result<(Vec<Pool>, u64), PoolSyncError> {
        let mut last_processed_block = 0; // get this from database

        let mut synced_pools = Vec::new();
        loop {
            // Check if we have synced to tip
            let current_block = self.syncer.block_number().await?;
            if last_processed_block == current_block {
                break;
            }

            info!(
                "Syncing from block {} to {}",
                last_processed_block + 1,
                current_block
            );

            // Sync each pool type for the block range
            for (pool_type, fetcher) in self.fetchers.iter() {
                // Fetch all pool addresses
                let new_addresses = self
                    .syncer
                    .fetch_addresses(last_processed_block + 1, current_block, fetcher.clone())
                    .await?;

                info!("Fetched {} addresses", new_addresses.len());

                // Populate pool information and then liquidity information
                let mut pools = self
                    .syncer
                    .populate_pool_info(new_addresses, pool_type, current_block)
                    .await?;
                self.syncer.populate_liquidity(&mut pools, pool_type).await?;
                synced_pools.extend(pools);
            }

            last_processed_block = current_block;
        }
        info!("Synced {} pools", synced_pools.len());
        Ok((synced_pools, last_processed_block))
    }
}
