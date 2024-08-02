//! Pool Synchronization Program
//!
//! This program synchronizes pools from a specified blockchain using the PoolSync library.
//! It demonstrates how to set up a provider, configure pool synchronization, and execute the sync process.

use anyhow::Result;
use alloy::primitives::Address;
use pool_sync::{Chain, Pool, PoolInfo, PoolSync, PoolType};


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

    // Configure and build the PoolSync instance
    let pool_sync = PoolSync::builder()
        .add_pools(&[PoolType::UniswapV2])
        .chain(Chain::Ethereum) // Specify the chain
        .rate_limit(1000)
        .build()?;

    // Initiate the sync process
    let pools = pool_sync.sync_pools().await?;

    let addresses: Vec<Address> = pools.into_iter().map(|pool| pool.address()).collect();
    println!("Number of synchronized pools: {}", addresses.len());

    Ok(())
}
