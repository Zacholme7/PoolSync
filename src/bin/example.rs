use alloy::providers::{Provider, ProviderBuilder};
use anyhow::Result;
use pool_sync::{PoolSync, PoolType, Chain};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    let url = "https://rpc.merkle.io/1/sk_mbs_f124c596d96bd0fddcdaaa0ff626ade0".parse()?;
    let provider = Arc::new(ProviderBuilder::new()
        .network::<alloy::network::AnyNetwork>()
        .with_recommended_fillers()
        .on_http(url));

    let pool_sync = PoolSync::builder()
        .add_pool(PoolType::UniswapV3)
        .chain(Chain::Ethereum)
        .build()?;

    let pools = pool_sync.sync_pools(provider.clone()).await?;
    println!("pools: {:?}", pools.len());

    Ok(())
}
