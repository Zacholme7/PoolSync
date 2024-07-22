//! Pool Synchronization Program
//!
//! This program synchronizes pools from a specified blockchain using the PoolSync library.
//! It demonstrates how to set up a provider, configure pool synchronization, and execute the sync process.

use alloy::providers::ProviderBuilder;
use anyhow::Result;
use pool_sync::filter::filter_top_volume;
use pool_sync::{Chain, Pool, PoolInfo, PoolSync, PoolType};
use std::sync::Arc;

/// The main entry point for the pool synchronization program.
///
/// This function performs the following steps:
/// 1. Loads environment variables
/// 2. Constructs an Alloy provider for the specified chain
/// 3. Configures and builds a PoolSync instance
/// 4. Initiates the pool synchronization process
/// 5. Prints the number of synchronized pools
///
/// # Errors
///
/// This function will return an error if:
/// - The required environment variables are not set
/// - There's an issue constructing the provider or PoolSync instance
/// - The synchronization process fails

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from a .env file if present
    dotenv::dotenv().ok();
    let url = std::env::var("BASE")?;

    let http_provider = Arc::new(
        ProviderBuilder::new()
            .network::<alloy::network::AnyNetwork>()
            .on_http(url.parse().unwrap()),
    );

    // Configure and build the PoolSync instance
    let pool_sync = PoolSync::builder()
        //k.add_pool(PoolType::UniswapV2) // Add all the pools you would like to sync
        .add_pools(&[PoolType::UniswapV2])
        .chain(Chain::Base) // Specify the chain
        .rate_limit(100)
        .build()?;

    // Initiate the sync process
    let pools = pool_sync.sync_pools(http_provider.clone()).await?;
    println!("Number of synchronized pools: {}", pools.len());

    // extract all pools with top volume tokens
    let pools_over_top_volume_tokens = filter_top_volume(pools, 100, Chain::Base).await;
    println!("{:?}", pools_over_top_volume_tokens.len());
    Ok(())
}
