use alloy::primitives::{Address, Log};
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use sushiswap::SushiSwapPool;
use uniswap_v2::UniswapV2Pool;
use uniswap_v3::UniswapV3Pool;
use std::fmt;
use crate::chain::Chain;

// reexports
pub use sushiswap::SushiSwapFetcher;
pub use uniswap_v2::UniswapV2Fetcher;
pub use uniswap_v3::UniswapV3Fetcher;

// pool modules
mod sushiswap;
mod uniswap_v2;
mod uniswap_v3;

/// Enumerate the pools supported
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PoolType {
    UniswapV2,
    UniswapV3,
    SushiSwap,
}

impl fmt::Display for PoolType {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{:?}", self)
        }
}

/// Populated protocol variants 
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Pool {
    UniswapV2(UniswapV2Pool),
    UniswapV3(UniswapV3Pool),
    SushiSwap(SushiSwapPool),
}

/// Defines common functionality for fetching the log information for a pool creation event and
/// then decoding the log into the pool
#[async_trait]
pub trait PoolFetcher: Send + Sync {
    fn pool_type(&self) -> PoolType;
    fn factory_address(&self, chain: Chain) -> Address;
    fn pair_created_signature(&self) -> &str;
    async fn from_log(&self, log: &Log) -> Option<Pool>;
}
