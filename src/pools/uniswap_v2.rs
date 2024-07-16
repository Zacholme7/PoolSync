//! Uniswap V2 Pool Synchronization Implementation
//!
//! This module provides the Uniswap V2-specific implementations for pool synchronization,
//! including the pool structure, factory contract interface, and pool fetcher.

use std::sync::Arc;

use crate::chain::Chain;
use crate::pools::{Pool, PoolFetcher, PoolType};
use alloy::dyn_abi::DynSolValue;
use alloy::network::Network;
use alloy::primitives::{address, Address, Log};
use alloy::primitives::U128;
use alloy::providers::Provider;
use alloy::sol_types::{SolEvent};
use alloy::sol;
use alloy::transports::Transport;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

// Uniswap V2 factory contract interface
sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    contract UniswapV2Factory  {
        event PairCreated(address indexed token0, address indexed token1, address pair, uint256);
    }
);


/// Represents a Uniswap V2 Automated Market Maker (AMM) pool
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UniswapV2Pool {
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

/// Uniswap V2 pool fetcher implementation
pub struct UniswapV2Fetcher;


impl UniswapV2Fetcher {
    pub async fn build_pools_from_addrs<P, T, N>(
        &self,
        provider: Arc<P>,
        addresses: Vec<Address>
    ) -> Vec<Pool>
    where
        P: Provider<T, N> + Sync,
        T: Transport + Sync + Clone,
        N: Network
    {
        let pools: Vec<Pool> = Vec::new();
        println!("{:?}", addresses);
        pools
    }
}

#[async_trait]
impl PoolFetcher for UniswapV2Fetcher {
    /// Returns the pool type for Uniswap V2
    fn pool_type(&self) -> PoolType {
        PoolType::UniswapV2
    }

    /// Returns the factory address for Uniswap V2 on the given chain
    fn factory_address(&self, chain: Chain) -> Address {
        match chain {
            Chain::Ethereum => address!("5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f"),
            Chain::Base => address!("8909Dc15e40173Ff4699343b6eB8132c65e18eC6"),
        }
    }

    /// Returns the event signature for pair creation in Uniswap V2
    fn pair_created_signature(&self) -> &str {
        UniswapV2Factory::PairCreated::SIGNATURE
    }

    /// Attempts to create a `Pool` instance from a log entry
    fn log_to_address(&self, log: &Log) -> Address {
        let decoded_log = UniswapV2Factory::PairCreated::decode_log(log, false).unwrap();
        decoded_log.data.pair
    }

    fn construct_pool_from_data(&self, data: &[DynSolValue]) -> Pool {
        todo!()
    }
}
