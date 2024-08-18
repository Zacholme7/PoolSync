//! Core definitions for pool synchronization
//!
//! This module defines the core structures and traits used in the pool synchronization system.
//! It includes enumerations for supported pool types, a unified `Pool` enum, and a trait for
//! fetching and decoding pool creation events.

use crate::chain::Chain;
use crate::impl_pool_info;
use alloy::network::Network;
use alloy::primitives::U128;
use alloy::primitives::U256;
use alloy::primitives::{Address, Log};
use alloy::providers::Provider;
use alloy::transports::Transport;
use pool_structure::{SimulatedPool, TickInfo, UniswapV2Pool, UniswapV3Pool};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{fmt, sync::Arc};

pub use v2_builder::build_pools as build_v2_pools;
pub use v2_builder::process_sync_data;
pub use v3_builder::build_pools as build_v3_pools;
pub use v3_builder::process_tick_data;
pub use simulated_builder::build_pools as build_simulated_pools;
pub mod aerodrome;
pub mod alien_base;
pub mod base_swap;
mod gen;
pub mod maverick;
pub mod pancake_swap;
pub mod pool_structure;
mod simulated_builder;
pub mod sushiswap;
pub mod uniswap;
mod v2_builder;
mod v3_builder;

/// Enumerates the supported pool types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

    AlienBase,

    MaverickV1,
    MaverickV2,
}

impl PoolType {
    pub fn is_v2(&self) -> bool {
        self == &PoolType::UniswapV2
            || self == &PoolType::SushiSwapV2
            || self == &PoolType::PancakeSwapV2
            || self == &PoolType::Aerodrome
            || self == &PoolType::BaseSwapV2
    }

    pub fn is_v3(&self) -> bool {
        self == &PoolType::UniswapV3
            || self == &PoolType::SushiSwapV3
            || self == &PoolType::PancakeSwapV3
            || self == &PoolType::Slipstream
            || self == &PoolType::BaseSwapV3
            || self == &PoolType::AlienBase
    }

    pub fn is_simulated(&self) -> bool {
        self == &PoolType::MaverickV1
            || self == &PoolType::MaverickV2
    }
}

/// Represents a populated pool from any of the supported protocols
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Pool {
    UniswapV2(UniswapV2Pool),
    SushiSwapV2(UniswapV2Pool),
    PancakeSwapV2(UniswapV2Pool),
    Aerodrome(UniswapV2Pool),
    BaseSwapV2(UniswapV2Pool),

    UniswapV3(UniswapV3Pool),
    SushiSwapV3(UniswapV3Pool),
    PancakeSwapV3(UniswapV3Pool),
    Slipstream(UniswapV3Pool),
    BaseSwapV3(UniswapV3Pool),
    AlienBase(UniswapV3Pool),

    MaverickV1(SimulatedPool),
    MaverickV2(SimulatedPool),
}

impl Pool {
    pub fn new_v2(pool_type: PoolType, pool: UniswapV2Pool) -> Self {
        match pool_type {
            PoolType::UniswapV2 => Pool::UniswapV2(pool),
            PoolType::SushiSwapV2 => Pool::SushiSwapV2(pool),
            PoolType::PancakeSwapV2 => Pool::PancakeSwapV2(pool),
            PoolType::Aerodrome => Pool::Aerodrome(pool),
            PoolType::BaseSwapV2 => Pool::BaseSwapV2(pool),
            _ => panic!("Invalid pool type"),
        }
    }

    pub fn new_v3(pool_type: PoolType, pool: UniswapV3Pool) -> Self {
        match pool_type {
            PoolType::UniswapV3 => Pool::UniswapV3(pool),
            PoolType::SushiSwapV3 => Pool::SushiSwapV3(pool),
            PoolType::PancakeSwapV3 => Pool::PancakeSwapV3(pool),
            PoolType::Slipstream => Pool::Slipstream(pool),
            PoolType::BaseSwapV3 => Pool::BaseSwapV3(pool),
            PoolType::AlienBase => Pool::AlienBase(pool),
            _ => panic!("Invalid pool type"),
        }
    }

    pub fn new_simulated(pool_type: PoolType, pool: SimulatedPool) -> Self {
        match pool_type {
            PoolType::MaverickV1 => Pool::MaverickV1(pool),
            PoolType::MaverickV2 => Pool::MaverickV2(pool),
            _ => panic!("Invalid pool type"),
        }
    }

    pub fn is_v2(&self) -> bool {
        match self {
            Pool::UniswapV2(_) => true,
            Pool::SushiSwapV2(_) => true,
            Pool::PancakeSwapV2(_) => true,
            Pool::Aerodrome(_) => true,
            Pool::BaseSwapV2(_) => true,
            _ => false,
        }
    }

    pub fn is_v3(&self) -> bool {
        match self {
            Pool::UniswapV3(_) => true,
            Pool::SushiSwapV3(_) => true,
            Pool::PancakeSwapV3(_) => true,
            Pool::Slipstream(_) => true,
            Pool::BaseSwapV3(_) => true,
            Pool::AlienBase(_) => true,
            _ => false,
        }
    }

    pub fn is_simulated(&self) -> bool {
        match self {
            Pool::MaverickV1(_) => true,
            Pool::MaverickV2(_) => true,
            _ => false,
        }
    }

    pub fn get_v2(&self) -> Option<&UniswapV2Pool> {
        match self {
            Pool::UniswapV2(pool) => Some(pool),
            Pool::SushiSwapV2(pool) => Some(pool),
            Pool::PancakeSwapV2(pool) => Some(pool),
            Pool::Aerodrome(pool) => Some(pool),
            Pool::BaseSwapV2(pool) => Some(pool),
            _ => None,
        }
    }

    pub fn get_v3(&self) -> Option<&UniswapV3Pool> {
        match self {
            Pool::UniswapV3(pool) => Some(pool),
            Pool::SushiSwapV3(pool) => Some(pool),
            Pool::PancakeSwapV3(pool) => Some(pool),
            Pool::Slipstream(pool) => Some(pool),
            Pool::BaseSwapV3(pool) => Some(pool),
            Pool::AlienBase(pool) => Some(pool),
            _ => None,
        }
    }

    pub fn get_simulated(&self) -> Option<&SimulatedPool> {
        match self {
            Pool::MaverickV1(pool) => Some(pool),
            Pool::MaverickV2(pool) => Some(pool),
            _ => None,
        }
    }

    pub fn get_v2_mut(&mut self) -> Option<&mut UniswapV2Pool> {
        match self {
            Pool::UniswapV2(pool) => Some(pool),
            Pool::SushiSwapV2(pool) => Some(pool),
            Pool::PancakeSwapV2(pool) => Some(pool),
            Pool::Aerodrome(pool) => Some(pool),
            Pool::BaseSwapV2(pool) => Some(pool),
            _ => None,
        }
    }

    pub fn get_v3_mut(&mut self) -> Option<&mut UniswapV3Pool> {
        match self {
            Pool::UniswapV3(pool) => Some(pool),
            Pool::SushiSwapV3(pool) => Some(pool),
            Pool::PancakeSwapV3(pool) => Some(pool),
            Pool::Slipstream(pool) => Some(pool),
            Pool::BaseSwapV3(pool) => Some(pool),
            Pool::AlienBase(pool) => Some(pool),
            _ => None,
        }
    }

    pub fn get_simulated_mut(&mut self) -> Option<&mut SimulatedPool> {
        match self {
            Pool::MaverickV1(pool) => Some(pool),
            Pool::MaverickV2(pool) => Some(pool),
            _ => None,
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
    Aerodrome,
    Slipstream,
    BaseSwapV2,
    BaseSwapV3,
    AlienBase,
    MaverickV1,
    MaverickV2
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
        provider: Arc<P>,
        addresses: Vec<Address>,
    ) -> Vec<Pool>
    where
        P: Provider<T, N> + Sync + 'static,
        T: Transport + Sync + Clone,
        N: Network,
    {
        match self {
            PoolType::UniswapV2 => {
                v2_builder::build_pools(provider, addresses, PoolType::UniswapV2).await
            }
            PoolType::SushiSwapV2 => {
                v2_builder::build_pools(provider, addresses, PoolType::SushiSwapV2).await
            }
            PoolType::PancakeSwapV2 => {
                v2_builder::build_pools(provider, addresses, PoolType::PancakeSwapV2).await
            }
            PoolType::Aerodrome => {
                v2_builder::build_pools(provider, addresses, PoolType::Aerodrome).await
            }
            PoolType::BaseSwapV2 => {
                v2_builder::build_pools(provider, addresses, PoolType::BaseSwapV2).await
            }
            PoolType::UniswapV3 => {
                v3_builder::build_pools(provider, addresses, PoolType::UniswapV3).await
            }
            PoolType::SushiSwapV3 => {
                v3_builder::build_pools(provider, addresses, PoolType::SushiSwapV3).await
            }
            PoolType::PancakeSwapV3 => {
                v3_builder::build_pools(provider, addresses, PoolType::PancakeSwapV3).await
            }
            PoolType::Slipstream => {
                v3_builder::build_pools(provider, addresses, PoolType::Slipstream).await
            }
            PoolType::BaseSwapV3 => {
                v3_builder::build_pools(provider, addresses, PoolType::BaseSwapV3).await
            }
            PoolType::AlienBase => {
                v3_builder::build_pools(provider, addresses, PoolType::AlienBase).await
            }
            PoolType::MaverickV1 => {
                simulated_builder::build_pools(provider, addresses, PoolType::MaverickV1).await
            }
            PoolType::MaverickV2 => {
                simulated_builder::build_pools(provider, addresses, PoolType::MaverickV2).await
            }
            _ => panic!("Invalid pool type"),
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
    fn stable(&self) -> bool;
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
                    Pool::UniswapV3(pool) | Pool::SushiSwapV3(pool) | Pool::PancakeSwapV3(pool) | Pool::Slipstream(pool) => pool.fee,
                    _ => 0
                }
            }

            fn stable(&self) -> bool {
                match self {
                    Pool::Aerodrome(pool) => pool.stable.unwrap(),
                    _=> false
                }
            }
        }
    };
}
