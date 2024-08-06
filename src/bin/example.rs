//! Pool Synchronization Program
//!
//! This program synchronizes pools from a specified blockchain using the PoolSync library.
//! It demonstrates how to set up a provider, configure pool synchronization, and execute the sync process.
use anyhow::Result;
use alloy::primitives::Address;
use alloy::primitives::U256;
use pool_sync::{snapshot::{v3_pool_snapshot}, Chain, Pool, PoolInfo, PoolSync, PoolType};
use pool_sync::filter::filter_pools_by_liquidity;
use std::sync::Arc;
use alloy::providers::ProviderBuilder;

#[tokio::main]
async fn main() -> Result<()> {
    // Configure and build the PoolSync instance
    let pool_sync = PoolSync::builder()
        .add_pools(&[
            PoolType::UniswapV2, 
            PoolType::UniswapV3,
            PoolType::SushiSwapV2,
            PoolType::SushiSwapV3,
        ])
        .chain(Chain::Base) // Specify the chain
        .rate_limit(1000)
        .build()?;

    // Initiate the sync process
    let pools = pool_sync.sync_pools().await?;

    let addresses: Vec<Address> = pools.iter().map(|pool| pool.address()).collect();
    println!("Number of synchronized pools: {}", addresses.len());

    let provider = Arc::new(ProviderBuilder::new().on_http(std::env::var("FULL").unwrap().parse().unwrap()));

    println!("Pool len before filtering: {}", pools.len());
    let res = filter_pools_by_liquidity(provider, pools, U256::from(5e17)).await;
    println!("Pool len after filtering: {}", res.len());

    //let addresses: Vec<Address> = addresses.clone().into_iter().rev().take(10).collect();
    //let output = v3_pool_snapshot(&addresses, provider).await.unwrap();
    



    Ok(())
}
