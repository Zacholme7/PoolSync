//! Core definitions for pool synchronization
//!
//! This module defines the core structures and traits used in the pool synchronization system.
//! It includes enumerations for supported pool types, a unified `Pool` enum, and a trait for
//! fetching and decoding pool creation events.

use crate::chain::Chain;
use crate::impl_pool_info;
use alloy::primitives::{Address, Log};
use alloy::providers::Provider;
use alloy::transports::Transport;
use alloy::network::Network;
use pool_structure::{TickInfo, UniswapV2Pool, UniswapV3Pool};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{fmt, sync::Arc};
use alloy::primitives::U128;
use alloy::primitives::U256;


pub mod pool_structure;
pub use v2_builder::build_pools as build_v2_pools;
pub use v3_builder::build_pools as build_v3_pools;
pub use v3_builder::process_tick_data;
mod gen;
mod v2_builder;
mod v3_builder;
pub mod uniswap;
pub mod sushiswap;
pub mod pancake_swap;
pub mod aerodome;


/// Enumerates the supported pool types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PoolType {
    UniswapV2,
    SushiSwapV2,
    PancakeSwapV2,

    UniswapV3,
    SushiSwapV3,
    PancakeSwapV3,

    Aerodome,
}

/// Represents a populated pool from any of the supported protocols
#[derive(Debug, Clone,  Serialize, Deserialize)]
pub enum Pool {
    UniswapV2(UniswapV2Pool),
    SushiSwapV2(UniswapV2Pool),
    PancakeSwapV2(UniswapV2Pool),

    UniswapV3(UniswapV3Pool),
    SushiSwapV3(UniswapV3Pool),
    PancakeSwapV3(UniswapV3Pool),

    Aerodome(UniswapV2Pool),
}

impl Pool {
    pub fn new_v2(pool_type: PoolType, pool: UniswapV2Pool) -> Self {
        match pool_type {
            PoolType::UniswapV2 => Pool::UniswapV2(pool),
            PoolType::SushiSwapV2 => Pool::SushiSwapV2(pool),
            PoolType::PancakeSwapV2 => Pool::PancakeSwapV2(pool),
            PoolType::Aerodome => Pool::Aerodome(pool),
            _ => panic!("Invalid pool type")
        }
    }

    pub fn new_v3(pool_type: PoolType, pool: UniswapV3Pool) -> Self {
        match pool_type {
            PoolType::UniswapV3 => Pool::UniswapV3(pool),
            PoolType::SushiSwapV3 => Pool::SushiSwapV3(pool),
            PoolType::PancakeSwapV3 => Pool::PancakeSwapV3(pool),
            _ => panic!("Invalid pool type")
        }
    }
}


impl fmt::Display for PoolType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}


// Implement the PoolInfo trait for all pool variants that are supported
impl_pool_info!(
    Pool, 
    UniswapV2, 
    SushiSwapV2,
    PancakeSwapV2,
    UniswapV3, 
    SushiSwapV3, 
    PancakeSwapV3,
    Aerodome
);

/* 
*/


/// Defines common functionality for fetching and decoding pool creation events
///
/// This trait provides a unified interface for different pool types to implement
/// their specific logic for identifying and parsing pool creation events.
pub trait PoolFetcher: Send + Sync {
    /// Returns the type of pool this fetcher is responsible for
    fn pool_type(&self) -> PoolType;

    /// Returns the factory address for the given chain
    fn factory_address(&self, chain: Chain) -> Address;

    /// Returns the event signature for pool creation
    fn pair_created_signature(&self) -> &str;

    /// Attempts to create a `Pool` instance from a log entry
    fn log_to_address(&self, log: &Log) -> Address;
}

impl PoolType {
    pub async fn build_pools_from_addrs<P, T, N>(
        &self,
        start_end: (u64, u64),
        provider: Arc<P>,
        addresses: Vec<Address>,
    ) -> Vec<Pool>
    where
        P: Provider<T, N> + Sync + 'static,
        T: Transport + Sync + Clone,
        N: Network 
    {
        match self {
            PoolType::UniswapV2 => v2_builder::build_pools(provider, addresses, PoolType::UniswapV2).await,
            PoolType::SushiSwapV2 => v2_builder::build_pools(provider, addresses, PoolType::SushiSwapV2).await,
            PoolType::PancakeSwapV2 => v2_builder::build_pools(provider, addresses, PoolType::PancakeSwapV2).await,
            PoolType::UniswapV3 => v3_builder::build_pools(start_end, provider, addresses, PoolType::UniswapV3).await,
            PoolType::SushiSwapV3 => v3_builder::build_pools(start_end, provider, addresses, PoolType::SushiSwapV3).await,
            PoolType::PancakeSwapV3 => v3_builder::build_pools(start_end, provider, addresses, PoolType::PancakeSwapV3).await,
            PoolType::Aerodome => v2_builder::build_pools(provider, addresses, PoolType::Aerodome).await,
        }
    }

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
    fn fee(&self) -> u32;
}

pub trait V2PoolInfo {
    fn token0_reserves(&self) -> U128;
    fn token1_reserves(&self) -> U128;
}

pub trait V3PoolInfo {
    fn fee(&self) -> u32;
    fn tick_spacing(&self) -> i32;
    fn tick_bitmap(&self) -> HashMap<i16, U256>;
    fn ticks(&self) -> HashMap<i32, TickInfo>;
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

            fn fee(&self) -> u32 {
                match self {
                    Pool::UniswapV3(pool) | Pool::SushiSwapV3(pool) | Pool::PancakeSwapV3(pool) => pool.fee,
                    _ => 0
                }
            }
        }
    };
}

/* 
#[macro_export]
macro_rules! impl_v2_pool_info {
    ($enum_name:ident, $($variant:ident),+) => {
        impl V2PoolInfo for $enum_name {
            fn token0_reserves(&self) -> U128 {
                match self {
                    $(
                        $enum_name::$variant(pool) => pool.token0_reserves,
                    )+
                    _ => panic!("Not a V2 pool"),
                }
            }

            fn token1_reserves(&self) -> U128 {
                match self {
                    $(
                        $enum_name::$variant(pool) => pool.token1_reserves,
                    )+
                    _ => panic!("Not a V2 pool"),
                }
            }
        }
    };
}

#[macro_export]
macro_rules! impl_v3_pool_info {
    ($enum_name:ident, $($variant:ident),+) => {
        impl V3PoolInfo for $enum_name {
            fn fee(&self) -> u32 {
                match self {
                    $(
                        $enum_name::$variant(pool) => pool.fee,
                    )+
                    _ => panic!("Not a V3 pool"),
                }
            }

            fn tick_spacing(&self) -> i32 {
                match self {
                    $(
                        $enum_name::$variant(pool) => pool.tick_spacing,
                    )+
                    _ => panic!("Not a V3 pool"),
                }
            }

            fn tick_bitmap(&self) -> &HashMap<i16, U256> {
                match self {
                    $(
                        $enum_name::$variant(pool) => pool.tick_bitmap,
                    )+
                    _ => panic!("Not a V3 pool"),
                }
            }

            fn ticks(&self) -> &HashMap<i32, TickInfo> {
                match self {
                    $(
                        $enum_name::$variant(pool) => pool.ticks,
                    )+
                    _ => panic!("Not a V3 pool"),
                }
            }
        }
    };
}
*/