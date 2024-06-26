//! Uniswap V3 Pool Synchronization Implementation
//!
//! This module provides the Uniswap V3-specific implementations for pool synchronization,
//! including the pool structure, factory contract interface, and pool fetcher.

use crate::chain::Chain;
use crate::pools::{Pool, PoolFetcher, PoolType};
use alloy::primitives::address;
use alloy::primitives::{Address, Log};
use alloy::sol_types::{sol, SolEvent};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

// Uniswap V3 factory contract interface
sol! {
    #[derive(Debug)]
    #[sol(rpc)]
    contract UniswapV3Factory {
        event PoolCreated(
            address indexed token0,
            address indexed token1,
            uint24 indexed fee,
            int24 tickSpacing,
            address pool
        );
    }
}

/// Represents a Uniswap V3 pool
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UniswapV3Pool {
    /// The address of the pool contract
    pub address: Address,
    /// The address of the first token in the pair
    pub token0: Address,
    /// The address of the second token in the pair
    pub token1: Address,
    /// The fee tier of the pool
    pub fee: u32,
    /// The tick spacing of the pool
    pub tick_spacing: i32,
}

/// Uniswap V3 pool fetcher implementation
pub struct UniswapV3Fetcher;

#[async_trait]
impl PoolFetcher for UniswapV3Fetcher {
    /// Returns the pool type for Uniswap V3
    fn pool_type(&self) -> PoolType {
        PoolType::UniswapV3
    }

    /// Returns the factory address for Uniswap V3 on the given chain
    ///
    /// # Panics
    ///
    /// Panics if the protocol is not supported for the given chain
    fn factory_address(&self, chain: Chain) -> Address {
        match chain {
            Chain::Ethereum => address!("1F98431c8aD98523631AE4a59f267346ea31F984"),
            Chain::Base => address!("33128a8fC17869897dcE68Ed026d694621f6FDfD"),
        }
    }

    /// Returns the event signature for pool creation in Uniswap V3
    fn pair_created_signature(&self) -> &str {
        UniswapV3Factory::PoolCreated::SIGNATURE
    }

    /// Attempts to create a `Pool` instance from a log entry
    ///
    /// # Arguments
    ///
    /// * `log` - The log entry potentially containing pool creation data
    ///
    /// # Returns
    ///
    /// An `Option<Pool>` which is `Some(Pool)` if the log was successfully parsed,
    /// or `None` if the log did not represent a valid pool creation event or could not be decoded.
    async fn from_log(&self, log: &Log) -> Option<Pool> {
        let decoded_log = UniswapV3Factory::PoolCreated::decode_log(log, false).ok()?;
        Some(Pool::UniswapV3(UniswapV3Pool {
            address: decoded_log.data.pool,
            token0: decoded_log.data.token0,
            token1: decoded_log.data.token1,
            fee: decoded_log.data.fee,
            tick_spacing: decoded_log.data.tickSpacing,
        }))
    }
}
