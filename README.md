# PoolSync

PoolSync is a utility crate for efficiently synchronizing DeFi pools from various protocols on EVM-compatible blockchains. This crate streamlines the process of pool synchronization, eliminating the need for repetitive boilerplate code in DeFi projects.


## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
pool-sync = "2.0.4"
```

Configure your .env with a full node and a archive node. Archive must be an archive node. The full node can be either. 

```env
FULL = "full node endpoint"
ARCHIVE = "archive node endpoint"
```

## Example Usage
```rust
use pool_sync::{PoolSync, PoolType, Chain, PoolInfo};

#[tokio::main]
async fn main() -> Result<()> {
    // Configure and build the PoolSync instance
    let pool_sync = PoolSync::builder()
        .add_pool(PoolType::UniswapV2)
        .chain(Chain::Ethereum)
        .rate_limit(20)
        .build()?;

    // Synchronize pools
    let pools = pool_sync.sync_pools().await?;

    // Common Info
    for pool in &pools {
        println!("Pool Address {:?}, Token 0: {:?}, Token 1: {:?}", pool.address(), pool.token0(), pool.token1());
    }

    println!("Synced {} pools!", pools.len());
    Ok(())
}
```
