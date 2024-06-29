use std::collections::HashMap;
use std::sync::Arc;
use crate::{Chain, PoolSync, PoolType};
use crate::pools::*;
use crate::errors::*;


/// Defines a builder for constructing PoolSync
#[derive(Default)]
pub struct PoolSyncBuilder {
    /// Mapping from the pool type to the implementation of its fetcher
    fetchers: HashMap<PoolType, Arc<dyn PoolFetcher>>,
    /// The chain to be synced on
    chain: Option<Chain>
}


impl PoolSyncBuilder {
    /// Add a new pool to the ones that we want to sync
    pub fn add_pool(mut self, pool_type: PoolType) -> Self {
        match pool_type {
            PoolType::UniswapV2 => {
                self.fetchers
                    .insert(PoolType::UniswapV2, Arc::new(UniswapV2Fetcher));
            }
            PoolType::UniswapV3 => {
                self.fetchers
                    .insert(PoolType::UniswapV3, Arc::new(UniswapV3Fetcher));
            }
            PoolType::SushiSwap => {
                self.fetchers
                    .insert(PoolType::SushiSwap, Arc::new(SushiSwapFetcher));
            }
        }
        self
    }

    /// Set the chain
    pub fn chain(mut self, chain: Chain) -> Self {
        self.chain = Some(chain);
        self
    }

    /// Consume the builder and produce a constructed PoolSync
    pub fn build(self) -> Result<PoolSync, PoolSyncError> {
        // make sure the chain is set
        let chain = self.chain.ok_or(PoolSyncError::ChainNotSet).unwrap();

        // make sure all the pools are suppored
        for pool_type in self.fetchers.keys().into_iter() {
            if !chain.supported(pool_type) {
                return Err(PoolSyncError::UnsupportedPoolType);
            }
        }

        // valid, construct PoolSync 
        Ok(PoolSync {
            fetchers: self.fetchers,
            chain: self.chain.unwrap()
        })
    }
}