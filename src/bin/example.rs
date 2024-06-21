use anyhow::Result;
use alloy::providers::ProviderBuilder;
use pool_sync::{PoolSync, PoolType};

#[tokio::main]
async fn main() -> Result<()> {
    let url = "https://eth.merkle.io".parse()?;
    let provider =  ProviderBuilder::new().on_http(url);

    // build a PoolSync and then sync pools
    let pool_sync = PoolSync::builder()
        .add_pool(PoolType::UniswapV2)
        .build();
    pool_sync.sync_pools(&provider).await;

    Ok(())
}
