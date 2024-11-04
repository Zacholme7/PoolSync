//! Pool Synchronization Program
//!
//! This program synchronizes pools from a specified blockchain using the PoolSync library.
//! It demonstrates how to set up a provider, configure pool synchronization, and execute the sync process.
use anyhow::Result;
use pool_sync::{Chain, PoolSync, PoolType};
use env_logger;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    // Configure and build the PoolSync instance
    let pool_sync = PoolSync::builder()
        .add_pool(PoolType::Slipstream)
        .chain(Chain::Base)
        .rate_limit(1000)
        .build()?;

    // Synchronize pools
    let (pools, last_synced_block) = pool_sync.sync_pools().await?;
    println!(
        "Synced {} pools up to block {}!",
        pools.len(),
        last_synced_block
    );

    Ok(())
}
