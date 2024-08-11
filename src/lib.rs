//! PoolSync: A library for synchronizing and managing various types of liquidity pools across different blockchains
//!
//! This library provides functionality to interact with and synchronize data from
//! various decentralized exchange protocols across multiple blockchain networks.
//! It supports different pool types like Uniswap V2, Uniswap V3, and SushiSwap,
//! and can work with multiple blockchain networks such as Ethereum and Base.

// Public re-exports
pub use chain::Chain;
pub use pool_sync::PoolSync;
pub use pools::{build_v2_pools, build_v3_pools};
pub use pools::{Pool, PoolInfo, PoolType};
pub use rpc::Rpc;

// Internal modules
mod builder;
mod cache;
mod chain;
mod errors;
pub mod filter;
mod pool_sync;
pub mod pools;
mod rpc;
mod util;
