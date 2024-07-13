//! Pool Synchronization Program
//!
//! This program synchronizes pools from a specified blockchain using the PoolSync library.
//! It demonstrates how to set up a provider, configure pool synchronization, and execute the sync process.

use alloy::network::EthereumWallet;
use alloy::providers::{ProviderBuilder, WsConnect};
use alloy_node_bindings::anvil::Anvil;
use anyhow::Result;
use pool_sync::filter::filter_top_volume;
use pool_sync::{Chain, Pool, PoolInfo, PoolSync, PoolType};
use std::sync::Arc;

use alloy::signers::local::PrivateKeySigner;

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
    let url = std::env::var("ETH")?;
    let anvil = Anvil::new().fork(url).try_spawn()?;
    let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
    let wallet = EthereumWallet::from(signer);

    let http_provider = Arc::new(
        ProviderBuilder::new()
            .network::<alloy::network::AnyNetwork>()
            .with_recommended_fillers()
            .wallet(wallet)
            .on_http(anvil.endpoint_url()),
    );

    let ws_provider = Arc::new(
        ProviderBuilder::new()
            .network::<alloy::network::AnyNetwork>()
            .on_ws(WsConnect::new("wss://eth.merkle.io")).await?
    );

    // Configure and build the PoolSync instance
    let pool_sync = PoolSync::builder()
        .add_pool(PoolType::UniswapV2) // Add all the pools you would like to sync
        .chain(Chain::Ethereum) // Specify the chain
        .rate_limit(20) // Specify the rate limit
        .build()?;

    // Initiate the sync process
    let pools = pool_sync.sync_pools(http_provider.clone(), ws_provider.clone()).await?;

    println!("Number of synchronized pools: {}", pools.len());

    // print out common pool information
    for pool in &pools {
        println!(
            "Pool Address {:?}, Token 0: {:?}, Token 1: {:?}",
            pool.address(),
            pool.token0(),
            pool.token1()
        );
    }

    // extract all pools with top volume tokens
    //let pools_over_top_volume_tokens = filter_top_volume(pools, 10).await?;

    Ok(())
}
