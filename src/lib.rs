//! PoolSync: A library for synchronizing and managing various types of liquidity pools across different blockchains
//!
//! This library provides functionality to interact with and synchronize data from
//! various decentralized exchange protocols across multiple blockchain networks.
//! It supports different pool types like Uniswap V2, Uniswap V3, and SushiSwap,
//! and can work with multiple blockchain networks such as Ethereum and Base.

// Public re-exports
use crate::pools::PoolFetcher;
pub use chain::Chain;
use errors::PoolSyncError;
pub use pool_sync::PoolSync;
pub use pools::pool_structures::{
    balancer_v2_structure::BalancerV2Pool,
    maverick_structure::MaverickPool,
    tri_crypto_curve_structure::CurveTriCryptoPool,
    two_crypto_curve_structure::CurveTwoCryptoPool,
    v2_structure::UniswapV2Pool,
    v3_structure::{TickInfo, UniswapV3Pool},
};
pub use pools::{Pool, PoolInfo};
use std::sync::Arc;

// Internal modules
mod builder;
mod chain;
mod errors;
mod onchain;
mod pool_database;
mod pool_sync;
mod pools;
mod sync_database;
mod sync_rpc;

use alloy_primitives::Address;
use async_trait::async_trait;

// Sync all configured pools in a 3 pass approach
// 1) Fetch all of the pool addresses
// 2) Sync basic pool information (token names, decimals, etc)
// 3) Populate pool liquidity
#[async_trait]
pub(crate) trait Syncer {
    // Fetch all of the pool addresses for the sync configuration
    async fn fetch_addresses(
        &self,
        start_block: u64,
        end_block: u64,
        pool_fetcher: Arc<dyn PoolFetcher>,
    ) -> Result<Vec<Address>, PoolSyncError>;

    // Given a set of addresses, construct pool and populate it with basic information. This includes
    // token names, decimals, addresses, etc. This does not include liquidity information
    async fn populate_pool_info(
        &self,
        addresses: Vec<Address>,
        pool_type: &PoolType,
        block_num: u64,
    ) -> Result<Vec<Pool>, PoolSyncError>;

    // For a set of pools, populate it with all liquidity information
    async fn populate_liquidity(
        &self,
        pools: &mut Vec<Pool>,
        pool_type: &PoolType,
        start_block: u64,
        end_block: u64,
    ) -> Result<Vec<Pool>, PoolSyncError>;

    // Get the latest block number
    async fn block_number(&self) -> Result<u64, PoolSyncError>;
}

// Enumerate every specific pool variant that the syncer supports
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PoolType {
    UniswapV2,
    SushiSwapV2,
    PancakeSwapV2,
    UniswapV3,
    SushiSwapV3,
    PancakeSwapV3,
    Aerodrome,
    Slipstream,
    BaseSwapV2,
    BaseSwapV3,
    AlienBaseV2,
    AlienBaseV3,
    MaverickV1,
    MaverickV2,
    CurveTwoCrypto,
    CurveTriCrypto,
    BalancerV2,
    SwapBasedV2,
    SwapBasedV3,
    DackieSwapV2,
    DackieSwapV3,
}
