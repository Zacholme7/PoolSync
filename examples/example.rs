//! Pool Synchronization Program
//!
//! This program synchronizes pools from a specified blockchain using the PoolSync library.
//! It demonstrates how to set up a provider, configure pool synchronization, and execute the sync process.
use anyhow::Result;
use pool_sync::PoolSync;
use pool_sync::{Chain, PoolType};
use std::path::Path;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let filter = EnvFilter::builder()
        .parse("debug,alloy_transport_http=off,alloy_rpc_client=off,alloy_transport_ws=off,hyper_util=off,reqwest=off")
        .expect("filter should be valid");

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter)
        .init();

    // Define database path
    let db_path = Path::new("./pools.db");

    // Configure the pool sync with a database
    let pool_sync = PoolSync::builder()
        .chain(Chain::Base)
        .add_pool(PoolType::UniswapV2)
        .with_database(db_path)
        .build()?;

    // First, attempt to load any previously saved pools
    let loaded_pools = pool_sync.load_pools().await?;
    println!("Loaded {} pools from database", loaded_pools.len());

    // Get the last processed block
    if let Some(block) = pool_sync.get_last_processed_block().await? {
        println!("Resuming sync from block {}", block);
    } else {
        println!("Starting new sync");
    }

    // Sync pools
    let (pools, last_block) = pool_sync.sync_pools().await?;
    println!("Synced {} pools up to block {}", pools.len(), last_block);

    Ok(())
}
