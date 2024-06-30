//! Uniswap V2 Pool Synchronization Implementation
//!
//! This module provides the Uniswap V2-specific implementations for pool synchronization,
//! including the pool structure, factory contract interface, and pool fetcher.

use crate::chain::Chain;
use crate::pools::{Pool, PoolFetcher, PoolType};
use alloy::primitives::{address, Address, Log};
use alloy::sol_types::{sol, SolEvent};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Uniswap V2 factory contract interface
sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    contract UniswapV2Factory  {
        event PairCreated(address indexed token0, address indexed token1, address pair, uint256);
    }
);

/// Represents a Uniswap V2 Automated Market Maker (AMM) pool
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UniswapV2Pool {
    /// The address of the pool contract
    pub address: Address,
    /// The address of the first token in the pair
    token0: Address,
    /// The address of the second token in the pair
    token1: Address,
}

/// Uniswap V2 pool fetcher implementation
pub struct UniswapV2Fetcher;

#[async_trait]
impl PoolFetcher for UniswapV2Fetcher {
    /// Returns the pool type for Uniswap V2
    fn pool_type(&self) -> PoolType {
        PoolType::UniswapV2
    }

    /// Returns the factory address for Uniswap V2 on the given chain
    ///
    /// # Panics
    ///
    /// Panics if the protocol is not supported for the given chain
    fn factory_address(&self, chain: Chain) -> Address {
        match chain {
            Chain::Ethereum => address!("5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f"),
            Chain::Base => address!("8909Dc15e40173Ff4699343b6eB8132c65e18eC6"),
            _ => panic!("Protocol not supported for this chain"),
        }
    }

    /// Returns the event signature for pair creation in Uniswap V2
    fn pair_created_signature(&self) -> &str {
        UniswapV2Factory::PairCreated::SIGNATURE
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
    ///
    /// # Panics
    ///
    /// Panics if the log cannot be decoded. This should be handled more gracefully in production code.
    async fn from_log(&self, log: &Log) -> Option<Pool> {
        let decoded_log = UniswapV2Factory::PairCreated::decode_log(log, false).unwrap();
        Some(Pool::UniswapV2(UniswapV2Pool {
            address: decoded_log.data.pair,
            token0: decoded_log.data.token0,
            token1: decoded_log.data.token1,
        }))
    }
}
