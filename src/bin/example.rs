//! Pool Synchronization Program
//!
//! This program synchronizes pools from a specified blockchain using the PoolSync library.
//! It demonstrates how to set up a provider, configure pool synchronization, and execute the sync process.

use alloy::providers::ProviderBuilder;
use anyhow::Result;
use alloy::primitives::Address;
use pool_sync::{Chain, Pool, PoolInfo, PoolSync, PoolType};
use pool_sync::snapshot::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Configure and build the PoolSync instance
    let pool_sync = PoolSync::builder()
        .add_pools(&[PoolType::UniswapV3])
        .chain(Chain::Ethereum) // Specify the chain
        .rate_limit(1000)
        .build()?;

    // Initiate the sync process
    let pools = pool_sync.sync_pools().await?;

    let addresses: Vec<Address> = pools.into_iter().map(|pool| pool.address()).collect();
    println!("Number of synchronized pools: {}", addresses.len());


    println!("Getting snaphsot");

    let provider = Arc::new(ProviderBuilder::new().on_http(std::env::var("FULL").unwrap().parse().unwrap()));
    let start = std::time::Instant::now();
    let snapshot = v3_pool_snapshot(addresses, provider).await?;
    println!("Time taken: {:?}", start.elapsed());

    Ok(())
}
