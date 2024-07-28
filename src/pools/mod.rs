//! Core definitions for pool synchronization
//!
//! This module defines the core structures and traits used in the pool synchronization system.
//! It includes enumerations for supported pool types, a unified `Pool` enum, and a trait for
//! fetching and decoding pool creation events.

use crate::chain::Chain;
use crate::impl_pool_info;
use alloy::{dyn_abi::DynSolValue, network::Network, primitives::{Address, Log}, providers::Provider, transports::Transport};
use async_trait::async_trait;
use pool_structure::{UniswapV2Pool, UniswapV3Pool};
use serde::{Deserialize, Serialize};
use std::{fmt, sync::Arc};
use alloy::primitives::U128;
//use sushiswap_v2::SushiSwapPool;
//use uniswap_v2::UniswapV2Pool;
//use uniswap_v3::UniswapV3Pool;
//use aerodome::{AerodomeFetcher, AerodomePool};

// Re-exports
//pub use sushiswap_v2::SushiSwapFetcher;
//pub use uniswap_v2::UniswapV2Fetcher;
//pub use uniswap_v3::UniswapV3Fetcher;

// Pool modules
//mod sushiswap_v2;
//pub mod uniswap_v2;
//mod uniswap_v3;
//mod aerodome;
mod pool_structure;
mod gen;
mod v2_builder;
mod uniswap_v2_fetcher;

/// Enumerates the supported pool types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PoolType {
    /// Uniswap V2 pool type
    UniswapV2,
    /// Uniswap V3 pool type
    UniswapV3,
    /// SushiSwap pool type
    SushiSwap,
    /// Aerodome
    Aerodome,
}

/// Represents a populated pool from any of the supported protocols
#[derive(Debug, Clone,  Serialize, Deserialize)]
pub enum Pool {
    /// A Uniswap V2 pool
    UniswapV2(UniswapV2Pool),
    /// A Uniswap V3 pool
    UniswapV3(UniswapV3Pool),
    /// A SushiSwap pool
    SushiSwap(UniswapV2Pool),
    /// A Aerodome pool
    Aerodome(UniswapV2Pool),
}


impl PoolType {
    pub async fn build_pools_from_addrs<P, T, N>(
        &self,
        provider: Arc<P>,
        addresses: Vec<Address>
    ) -> Vec<Pool>
    where
        P: Provider<T, N> + Sync + 'static,
        T: Transport + Sync + Clone,
        N: Network
    {
        match  self {
            PoolType::UniswapV2 => UniswapV2Fetcher.build_pools_from_addrs(provider, addresses).await,
            _ => unimplemented!()
            //PoolType::UniswapV3 => UniswapV3Fetcher.build_pools_from_addrs(provider, addresses).await,
            //PoolType::SushiSwap => SushiSwapFetcher.build_pools_from_addrs(provider, addresses).await,
            //PoolType::Aerodome => AerodomeFetcher.build_pools_from_addrs(provider, addresses).await
        }
    }
}

impl fmt::Display for PoolType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}


// Implement the PoolInfo trait for all pool variants that are supported
impl_pool_info!(Pool, UniswapV2, UniswapV3, SushiSwap, Aerodome);


/// Defines common functionality for fetching and decoding pool creation events
///
/// This trait provides a unified interface for different pool types to implement
/// their specific logic for identifying and parsing pool creation events.
#[async_trait]
pub trait PoolFetcher: Send + Sync {
    /// Returns the type of pool this fetcher is responsible for
    fn pool_type(&self) -> PoolType;

    /// Returns the factory address for the given chain
    fn factory_address(&self, chain: Chain) -> Address;

    /// Returns the event signature for pool creation
    fn pair_created_signature(&self) -> &str;

    /// Attempts to create a `Pool` instance from a log entry
    fn log_to_address(&self, log: &Log) -> Address;

    async fn build_pools_from_addrs<P, T, N>(
        &self,
        provider: Arc<P>,
        addresses: Vec<Address>,
    ) -> Vec<Pool>
    where
        P: Provider<T, N> + Sync + 'static,
        T: Transport + Sync + Clone,
        N: Network;
}

/// Defines common methods that are used to access information about the pools
pub trait PoolInfo {
    fn address(&self) -> Address;
    fn token0_address(&self) -> Address;
    fn token1_address(&self) -> Address;
    fn token0_name(&self) -> String;
    fn token1_name(&self) -> String;
    fn token0_decimals(&self) -> u8;
    fn token1_decimals(&self) -> u8;
    fn pool_type(&self) -> PoolType;
    fn reserves(&self) -> (U128, U128);
}


/// Macro for generating getter methods for all of the suppored pools
#[macro_export]
macro_rules! impl_pool_info {
    ($enum_name:ident, $($variant:ident),+) => {
        impl PoolInfo for $enum_name {
            fn address(&self) -> Address {
                match self {
                    $(
                        $enum_name::$variant(pool) => pool.address,

                    )+
                }
            }

            fn token0_address(&self) -> Address {
                match self {
                    $(
                        $enum_name::$variant(pool) => pool.token0,
                    )+
                }
            }

            fn token1_address(&self) -> Address {
                match self {
                    $(
                        $enum_name::$variant(pool) => pool.token1,
                    )+
                }
            }

            fn token0_name(&self) -> String {
                match self {
                    $(
                        $enum_name::$variant(pool) => pool.token0_name.clone(),
                    )+
                }
            }
            fn token1_name(&self) -> String {
                match self {
                    $(
                        $enum_name::$variant(pool) => pool.token1_name.clone(),
                    )+
                }
            }

            fn token0_decimals(&self) -> u8 {
                match self {
                    $(
                        $enum_name::$variant(pool) => pool.token0_decimals,
                    )+
                }
            }
            fn token1_decimals(&self) -> u8 {
                match self {
                    $(
                        $enum_name::$variant(pool) => pool.token1_decimals,
                    )+
                }
            }

            fn pool_type(&self) -> PoolType {
                match self {
                    $(
                        $enum_name::$variant(_) => PoolType::$variant,
                    )+
                }
            }

            fn reserves(&self) -> (U128, U128) {
                match self {
                    $enum_name::UniswapV3(pool) => (pool.liquidity.into(), pool.liquidity.into()),
                    $enum_name::UniswapV2(pool) => (pool.token0_reserves, pool.token1_reserves),
                    $enum_name::SushiSwap(pool) => (pool.token0_reserves, pool.token1_reserves),
                    $enum_name::Aerodome(pool) => (pool.token0_reserves, pool.token1_reserves),
                }
            }
        }
    };
}
