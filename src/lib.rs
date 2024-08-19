//! PoolSync: A library for synchronizing and managing various types of liquidity pools across different blockchains
//!
//! This library provides functionality to interact with and synchronize data from
//! various decentralized exchange protocols across multiple blockchain networks.
//! It supports different pool types like Uniswap V2, Uniswap V3, and SushiSwap,
//! and can work with multiple blockchain networks such as Ethereum and Base.

// Public re-exports
pub use chain::Chain;
pub use pool_sync::PoolSync;
pub use pools::{Pool, PoolInfo, PoolType};
pub use rpc::Rpc;
pub use filter::fetch_top_volume_tokens;

// Internal modules
mod builder;
mod cache;
mod chain;
mod errors;
mod filter;
mod pool_sync;
mod pools;
mod rpc;
mod util;
