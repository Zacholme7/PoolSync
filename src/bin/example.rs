use alloy::providers::ProviderBuilder;
use pool_sync::{PoolSync, PoolType};
use std::sync::Arc;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let url = "https://rpc.merkle.io/1/sk_mbs_f3cc7544d55b8976b06f881c6910921c".parse()?;
    let provider =  Arc::new(ProviderBuilder::new().on_http(url));

    let pool_sync = PoolSync::builder()
        .add_pool(PoolType::UniswapV2)
        .build();
    let pools = pool_sync.sync_pools(provider.clone()).await;
    println!("Length: {}", pools.len());
    Ok(())
}