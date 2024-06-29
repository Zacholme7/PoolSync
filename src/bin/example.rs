use alloy::providers::{Provider, ProviderBuilder};
use anyhow::Result;
use pool_sync::{PoolSync, PoolType, Chain};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    let url = "https://base-mainnet.g.alchemy.com/v2/4Qnctl85Dx4oOtzPKc4Fz6z-0fH40TbW".parse()?;
    let provider = Arc::new(ProviderBuilder::new()
        .network::<alloy::network::AnyNetwork>()
        .with_recommended_fillers()
        .on_http(url));

    let pool_sync = PoolSync::builder()
        .add_pool(PoolType::UniswapV2)
        .add_pool(PoolType::UniswapV3)
        .chain(Chain::Base)
        .build()?;

    let pools = pool_sync.sync_pools(provider.clone()).await?;
    println!("pools: {:?}", pools.len());

    Ok(())
}
