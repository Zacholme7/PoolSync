//! Pool Synchronization Program
//!
//! This program synchronizes pools from a specified blockchain using the PoolSync library.
//! It demonstrates how to set up a provider, configure pool synchronization, and execute the sync process.
use anyhow::Result;
use pool_sync::{Chain, Pool, PoolSync, PoolType};

#[tokio::main]
async fn main() -> Result<()> {
    // Configure and build the PoolSync instance
    let pool_sync = PoolSync::builder()
        .add_pools(&[
            PoolType::BalancerV2
        ])
        .chain(Chain::Ethereum) // Specify the chain
        .build()?;

    // Initiate the sync process
    let (pools , last_synced_block)= pool_sync.sync_pools().await?;
    println!("Number of synchronized pools: {:#?}", pools.len());

    Ok(())
}
