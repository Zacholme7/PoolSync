//! PoolSync Core Implementation
//!
//! This module contains the core functionality for synchronizing pools across different
//! blockchain networks and protocols. It includes the main `PoolSync` struct and its
//! associated methods for configuring and executing the synchronization process.

use crate::errors::PoolSyncError;
use crate::pool_database::PoolDatabase;
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
    /// Database connection for persisting data
    database: Option<Arc<PoolDatabase>>,
}

impl PoolSync {
    // Construct a new instance of PoolSync
    pub(crate) fn new(
        fetchers: HashMap<PoolType, Arc<dyn PoolFetcher>>,
        syncer: Box<dyn Syncer>,
        database: Option<Arc<PoolDatabase>>,
    ) -> Self {
        Self {
            fetchers,
            syncer,
            database,
        }
    }

    /// Construct a new builder to configure sync parameters
    pub fn builder() -> PoolSyncBuilder {
        PoolSyncBuilder::default()
    }

    /// Sync all of the pools from the chain
    pub async fn sync_pools(&self) -> Result<(Vec<Pool>, u64), PoolSyncError> {
        // Get last processed block from database or default to 0

        let mut last_processed_block = db.get_last_processed_block(self.syncer.get_chain()).unwrap_or_default();

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
                // Get known addresses from the database if available
                let mut known_addresses = if let Some(db) = &self.database {
                    db.get_pool_addresses(self.syncer.get_chain(), *pool_type)?
                } else {
                    Vec::new()
                };

                // Fetch new pool addresses
                let new_addresses = self
                    .syncer
                    .fetch_addresses(last_processed_block + 1, current_block, fetcher.clone())
                    .await?;

                info!(
                    "Processing {} addresses for {}",
                    new_addresses.len(),
                    pool_type
                );

                // Populate pool information and then liquidity information
                let mut pools = self
                    .syncer
                    .populate_pool_info(new_addresses, pool_type, current_block)
                    .await?;
                self.syncer
                    .populate_liquidity(&mut pools, pool_type)
                    .await?;

                // Save pools to database if available
                if let Some(db) = &self.database {
                    db.save_pools(&pools, self.syncer.get_chain())?;
                }

                synced_pools.extend(pools);
            }

            last_processed_block = current_block;

            // Update last processed block in database
            if let Some(db) = &self.database {
                db.update_last_processed_block(self.syncer.get_chain(), last_processed_block)?;
            }
        }

        info!("Synced {} pools", synced_pools.len());
        Ok((synced_pools, last_processed_block))
    }

    /// Load saved pools from the database
    pub async fn load_pools(&self) -> Result<Vec<Pool>, PoolSyncError> {
        if let Some(db) = &self.database {
            // Get pool types to load
            let pool_types: Vec<PoolType> = self.fetchers.keys().cloned().collect();

            // Load pools from database
            let pools = db.load_pools(self.syncer.get_chain(), Some(pool_types.as_slice()))?;

            info!("Loaded {} pools from database", pools.len());
            Ok(pools)
        } else {
            info!("No database configured, returning empty pool list");
            Ok(Vec::new())
        }
    }

    /// Get the last processed block
    pub async fn get_last_processed_block(&self) -> Result<Option<u64>, PoolSyncError> {
        if let Some(db) = &self.database {
            db.get_last_processed_block(self.syncer.get_chain())
        } else {
            Ok(None)
        }
    }
}
