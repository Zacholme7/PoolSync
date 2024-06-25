use alloy::providers::Provider;
use alloy::network::Ethereum;
use alloy::transports::Transport;
use std::marker::PhantomData;
use std::sync::Arc;

use crate::pools::PoolType;
use crate::pools::Pool;

/// Builder for PoolSync
/// Allows you to configure the protocols you want to sync
#[derive(Default)]
pub struct PoolSyncBuilder<P, T> {
    pools: Vec<PoolType>,
    _phantom: PhantomData<(P, T)>
}

impl<P, T> PoolSyncBuilder<P, T> {
    /// Constructor
    pub fn new() -> Self {
        Self {
            pools: Vec::new(),
            _phantom: PhantomData
        }
    }

    /// Add a new protocol to the list to sync
    pub fn add_pool(mut self, pool_type: PoolType) -> Self {
        self.pools.push(pool_type);
        self
    }
    pub fn build(self) -> PoolSync<P, T> {
        PoolSync {
            pools: self.pools,
            _phantom: PhantomData
        }
    }
}

/// Core structure holding the pools we want to sync
pub struct PoolSync<P, T> {
    pools: Vec<PoolType>,
    _phantom: PhantomData<(P, T)>
}

impl<P, T> PoolSync<P, T> 
where 
    P: Provider<T, Ethereum> + 'static,
    T: Transport + Clone + 'static
{
    /// Constructs a builder
    pub fn builder() -> PoolSyncBuilder<P, T> {
        PoolSyncBuilder::new()
    }

    /// Syncs all pools
    pub async fn sync_pools(&self, provider: Arc<P>) -> Vec<Pool>{
        let mut all_pools: Vec<Pool> = Vec::new();
        for pool_type in &self.pools {
            let mut pools = pool_type.get_all_pools(provider.clone()).await;
            all_pools.append(&mut pools);
        }
        all_pools
    }
}
