use alloy::providers::ProviderBuilder;
use anyhow::Result;
use pool_sync::{PoolSync, PoolType, Chain};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    let url = std::env::var("ETH_RPC")?.parse()?;
    let provider = Arc::new(ProviderBuilder::new()
        .network::<alloy::network::AnyNetwork>()
        .with_recommended_fillers()
        .on_http(url));

    let pool_sync = PoolSync::builder()
        .add_pool(PoolType::UniswapV2)
        .chain(Chain::Ethereum)
        .build()?;

    let pools = pool_sync.sync_pools(provider.clone()).await?;
    println!("pools: {:?}", pools.len());

    Ok(())
}
