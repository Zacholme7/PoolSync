use anyhow::Result;
use alloy::providers::RootProvider;
use std::sync::Arc;
use alloy::transports::http::{Client, Http};
use crate::pools::PoolType;

/// Builder for PoolSync
/// Allows you to configure the protocols you want to sync
#[derive(Default)]
pub struct PoolSyncBuilder {
    pools: Vec<PoolType>
}

impl PoolSyncBuilder {
    /// Constructor
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a new protocol to the list to sync
    pub fn add_pool(mut self, pool_type: PoolType) -> Self {
        self.pools.push(pool_type);
        self
    }
    pub fn build(self) -> PoolSync {
        PoolSync {
            pools: self.pools,
        }
    }
}

pub struct PoolSync {
    pools: Vec<PoolType>, 
}

impl PoolSync {
    /// Constructs a builder
    pub fn builder() -> PoolSyncBuilder {
        PoolSyncBuilder::new()
    }

    /// Syncs all pools
    pub async fn sync_pools(&self, provider: Arc<RootProvider<Http<Client>>>) {
        for pool_type in &self.pools {
            pool_type.get_all_pools(provider.clone()).await
        }
        // for each pool, call the sync function which will dispatch the correct function
        /* 
        let all_pools = Vec::new();
        for pool_type in &self.pools {
            let pools = pool_type.sync(&proivder);
            // pools.pushd5jko
            //pool_type.sync(&provider)
        }
        */
    }


}
