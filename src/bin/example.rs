//! Pool Synchronization Program
//!
//! This program synchronizes pools from a specified blockchain using the PoolSync library.
//! It demonstrates how to set up a provider, configure pool synchronization, and execute the sync process.
use anyhow::Result;
use pool_sync::{Chain, PoolSync, PoolType};

#[tokio::main]
async fn main() -> Result<()> {
    // Configure and build the PoolSync instance
    let pool_sync = PoolSync::builder()
        .add_pools(&[
            PoolType::UniswapV2,
            PoolType::UniswapV3,
        ])
        .chain(Chain::Base) // Specify the chain
        .rate_limit(1000)
        .build()?;

    // Initiate the sync process
    let pools = pool_sync.sync_pools().await?;
    println!("Number of synchronized pools: {:#?}", pools.len());

    Ok(())
}
