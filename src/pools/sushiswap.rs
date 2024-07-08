//! SushiSwap Pool Synchronization Implementation
//!
//! This module provides the SushiSwap-specific implementations for pool synchronization,
//! including the pool structure, factory contract interface, and pool fetcher.

use crate::chain::Chain;
use crate::pools::{Pool, PoolFetcher, PoolType};
use alloy::primitives::address;
use alloy::primitives::{Address, Log, U128};
use alloy::sol_types::{sol, SolEvent};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

// SushiSwap factory contract interface
sol! {
    #[derive(Debug)]
    #[sol(rpc)]
    contract SushiSwapFactory {
        event PairCreated(address indexed token0, address indexed token1, address pair, uint256);
    }
}

/// Represents a SushiSwap pool
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SushiSwapPool {
    /// The address of the pool contract
    pub address: Address,
    /// The address of the first token in the pair
    pub token0: Address,
    /// The address of the second token in the pair
    pub token1: Address,
    /// The name of the first token in the pair
    pub token0_name: String,
    /// The name of the second token in the pair
    pub token1_name: String,
    /// The amount of decimals in the first token
    pub token0_decimals: u8,
    /// The amount of decimals in the second token
    pub token1_decimals: u8,
    /// The reserves for the first token
    pub token0_reserves: U128,
    /// the reserves for the second pair
    pub token1_reserves: U128
}

/// SushiSwap pool fetcher implementation
pub struct SushiSwapFetcher;

#[async_trait]
impl PoolFetcher for SushiSwapFetcher {
    /// Returns the pool type for SushiSwap
    fn pool_type(&self) -> PoolType {
        PoolType::SushiSwap
    }

    /// Returns the factory address for SushiSwap on the given chain
    ///
    /// # Panics
    ///
    /// Panics if the protocol is not supported for the given chain
    fn factory_address(&self, chain: Chain) -> Address {
        match chain {
            Chain::Ethereum => address!("C0AEe478e3658e2610c5F7A4A2E1777cE9e4f2Ac"),
            _ => panic!("Protocol is not supported for the chain"),
        }
    }

    /// Returns the event signature for pair creation in SushiSwap
    fn pair_created_signature(&self) -> &str {
        SushiSwapFactory::PairCreated::SIGNATURE
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
    /// or `None` if the log did not represent a valid pool creation event.
    async fn from_log(&self, log: &Log) -> Option<Pool> {
        let decoded_log = SushiSwapFactory::PairCreated::decode_log(log, false).ok()?;
        Some(Pool::SushiSwap(SushiSwapPool {
            address: decoded_log.data.pair,
            token0: decoded_log.data.token0,
            token1: decoded_log.data.token1,
        }))
    }
}
