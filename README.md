# PoolSync

PoolSync is a utility crate for efficiently synchronizing DeFi pools from various protocols on EVM-compatible blockchains. This crate streamlines the process of pool synchronization, eliminating the need for repetitive boilerplate code in DeFi projects.


## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
pool-sync = "0.1.2"
```

## Example Usage
```rust
use pool_sync::{PoolSync, PoolType, Chain, PoolInfo};

#[tokio::main]
async fn main() -> Result<()> {
    // Set up the provider
    let url = "https://eth.merkle.io".parse()?;
    let provider = Arc::new(ProviderBuilder::new().on_http(url));

    // Configure and build the PoolSync instance
    let pool_sync = PoolSync::builder()
        .add_pool(PoolType::UniswapV2)
        .chain(Chain::Ethereum)
        .rate_limit(20)
        .build()?;

    // Synchronize pools
    let pools = pool_sync.sync_pools(provider).await?;

    // Common Info
    for pool in &pools {
        println!("Pool Address {:?}, Token 0: {:?}, Token 1: {:?}", pool.address(), pool.token0(), pool.token1());
    }

    println!("Synced {} pools!", pools.len());
    Ok(())
}
```
