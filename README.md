# PoolSync

PoolSync is a utility crate for efficiently synchronizing DeFi pools from various protocols on EVM-compatible blockchains. This crate streamlines the process of pool synchronization, eliminating the need for repetitive boilerplate code in DeFi projects.


## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
pool-sync = "3.0.0"
```

Configure your .env with a full node and a archive node. Archive must be an archive node. The full node can be either. It is designed this way due to the fact the fact that is it much more accessible to host full nodes due to the storage requirements. The typical workflow is to use a paid archive endpoint for the initial intensive sync and let the fullnode take it from there. After the initial sync, all information will be cached and the strain on the endpoints is reduced immensely. 

```env
FULL = "full node endpoint"
ARCHIVE = "archive node endpoint"
```

## Supported Protocols
### ETH
- UniswapV2/V3
- SushiswapV2/V2
- PancakeswapV2/V3
- MaverickV1/V2
### Base
- UniswapV2/V3
- SushiswapV2/V3
- PancakeswapV2/V3
- BaseswapV2/V3
- MaverickV1/V2
- Aerodrome/Slipstream
- AlienBase

## Example Usage
```rust
use pool_sync::{PoolSync, PoolType, Chain, PoolInfo};
use std::error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    // Configure and build the PoolSync instance
    let pool_sync = PoolSync::builder()
        .add_pool(PoolType::UniswapV2)
        .chain(Chain::Ethereum)
        .build()?;

    // Synchronize pools
    let (pools, last_synced_block) = pool_sync.sync_pools().await?;

    // Common Info
    for pool in &pools {
        println!("Pool Address {:?}, Token 0: {:?}, Token 1: {:?}", pool.address(), pool.token0_name(), pool.token1_name());
    }

    println!("Synced {} pools!", pools.len());
    Ok(())
}
```

## How to add a new protocol
### If the protocol already exists 
1) Add the factory address to the proper fetcher in `pools/pool_fetchers`
2) If the chain does not exist, modify the chain enum and mapping in `chain.rs` to reflect it.

### If the protocol does not exist 
1) Add the pool abi to `pools/abi`
2) Create a new directory in `pools/pool_fetchers` with the relevant pool files. Implement the `PoolFetcher` trait. This is very simple to implement and is used for event parsing. Use other implementations as an example.
3) Most pools will fit into the structures defined in `pools/pool_structures`. In the case that it is not, create a new file for your pool type, define the structure of the pool, and implement From<&DynSolValue]> for the pool.
4) Go through `pools/mod.rs` and add your new pool variant to all relevant sections. Very straighforward
5) Include your fetcher in `builder.rs`
6) Add the pool type to the proper chain in `chain.rs`


## Todo
- Much better instructions to add new pools (sorry, this repo is constantly evolving so I dont want to commit to anything yet)
- Abstract logic into macro for easy pool addition in `pools/mod.rs`
- Implement rate limiting to make it possible to sync on free public endpoints
- Add option to use DB directly for sync. 

## Acknowledgment
Took a ton of inspiration from [amm-rs](https://github.com/darkforestry/amms-rs). Make sure to check them out, super great work there! :)













