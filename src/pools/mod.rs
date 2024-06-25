use alloy::primitives::Address;
use alloy::primitives::Log;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sushiswap::SushiSwapPool;
use uniswap_v2::UniswapV2Pool;
use uniswap_v3::UniswapV3Pool;

pub mod sushiswap;
pub mod uniswap_v2;
pub mod uniswap_v3;

/// Enumerate the pools supported
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PoolType {
    UniswapV2,
    UniswapV3,
    SushiSwap,
}

/// Populated pools
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
    fn factory_address(&self) -> Address;
    fn pair_created_signature(&self) -> &str;
    async fn from_log(&self, log: &Log) -> Option<Pool>;
}
