# PoolSync
Utility crate for sycing defi pools from various protocols on the evm. Every project has the same boring pool sync boilerplate so this crate is meant to streamline the process and provide an easy and efficient way to sync all the pool variants you require. 

# Example Usage
```
use pool_sync::{PoolSync, PoolType};

#[tokio::main]
async fn main() -> Result<()> {
    let url = "https://eth.merkle.io".parse()?;
    let provider =  ProviderBuilder::new().on_http(url);

    let pool_sync = PoolSync::builder()
        .add_pool(PoolType::UniswapV2)
        .build();
    let pools = pool_sync.sync_pools(&provider).await;
    info!("Synced {} pools!", pools.len();
    Ok(())
}
```
