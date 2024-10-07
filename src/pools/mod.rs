//! Core definitions for pool synchronization
//!
//! This module defines the core structures and traits used in the pool synchronization system.
//! It includes enumerations for supported pool types, a unified `Pool` enum, and a trait for
//! fetching and decoding pool creation events.

use alloy::dyn_abi::DynSolType;
use alloy::dyn_abi::DynSolValue;
use alloy::primitives::U128;
use alloy::primitives::U256;
use alloy::primitives::{Address, Log};
use pool_structures::balancer_v2_structure::BalancerV2Pool;
use pool_structures::two_crypto_curve_structure::CurveTwoCryptoPool;
use pool_structures::tri_crypto_curve_structure::CurveTriCryptoPool;
use pool_structures::maverick_structure::MaverickPool;
use pool_structures::v2_structure::UniswapV2Pool;
use pool_structures::v3_structure::TickInfo;
use pool_structures::v3_structure::UniswapV3Pool;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

use crate::chain::Chain;
use crate::impl_pool_info;

pub mod pool_fetchers;
pub mod pool_structures;
pub mod pool_builder;
mod gen;

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
    DackieSwapV3
}

impl PoolType {
    pub fn is_v2(&self) -> bool {
        matches!(self, 
            PoolType::UniswapV2 | PoolType::SushiSwapV2 | PoolType::PancakeSwapV2 |
            PoolType::Aerodrome | PoolType::BaseSwapV2 | PoolType::SwapBasedV2 |
            PoolType::DackieSwapV2 | PoolType::AlienBaseV2
        )
    }


    pub fn is_v3(&self) -> bool {
        matches!(self, 
            PoolType::UniswapV3 | PoolType::SushiSwapV3 | PoolType::PancakeSwapV3 |
            PoolType::Slipstream | PoolType::BaseSwapV3 | PoolType::AlienBaseV3 |
            PoolType::SwapBasedV3 | PoolType::DackieSwapV3
        )
    }

    pub fn is_maverick(&self) -> bool {
        matches!(self, PoolType::MaverickV1 | PoolType::MaverickV2)
    }

    pub fn is_curve_two(&self) -> bool {
        matches!(self, PoolType::CurveTwoCrypto)
    }

    pub fn is_curve_tri(&self) -> bool {
        matches!(self, PoolType::CurveTriCrypto)
    }

    pub fn is_balancer(&self) -> bool {
        matches!(self, PoolType::BalancerV2)
    }

    pub fn build_pool(&self, pool_data: &[DynSolValue]) -> Pool {
        if self.is_v2() {
            let pool = UniswapV2Pool::from(pool_data);
            Pool::new_v2(*self, pool)
        } else if self.is_v3() {
            let pool = UniswapV3Pool::from(pool_data);
            Pool::new_v3(*self, pool)
        } else if self.is_maverick() {
            let pool = MaverickPool::from(pool_data);
            Pool::new_maverick(*self, pool)
        } else if self.is_balancer() {
            let pool = BalancerV2Pool::from(pool_data);
            Pool::new_balancer(*self, pool)
        } else if self.is_curve_two() {
            let pool = CurveTwoCryptoPool::from(pool_data);
            Pool::new_curve_two(*self, pool)
        }else if self.is_curve_tri() {
            let pool = CurveTriCryptoPool::from(pool_data);
            Pool::new_curve_tri(*self, pool)
        } else {
            panic!("Invalid pool type");
        }
    }

}

/// Represents a populated pool from any of the supported protocols
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Pool {
    UniswapV2(UniswapV2Pool),
    SushiSwapV2(UniswapV2Pool),
    PancakeSwapV2(UniswapV2Pool),
    BaseSwapV2(UniswapV2Pool),
    AlienBaseV2(UniswapV2Pool),
    SwapBasedV2(UniswapV2Pool),
    DackieSwapV2(UniswapV2Pool),

    Aerodrome(UniswapV2Pool),
    Slipstream(UniswapV3Pool),

    UniswapV3(UniswapV3Pool),
    SushiSwapV3(UniswapV3Pool),
    PancakeSwapV3(UniswapV3Pool),
    BaseSwapV3(UniswapV3Pool),
    AlienBaseV3(UniswapV3Pool),
    SwapBasedV3(UniswapV3Pool),
    DackieSwapV3(UniswapV3Pool),

    MaverickV1(MaverickPool),
    MaverickV2(MaverickPool),

    CurveTwoCrypto(CurveTwoCryptoPool),
    CurveTriCrypto(CurveTriCryptoPool),

    BalancerV2(BalancerV2Pool),
}

impl Pool {
    pub fn new_v2(pool_type: PoolType, pool: UniswapV2Pool) -> Self {
        match pool_type {
            PoolType::UniswapV2 => Pool::UniswapV2(pool),
            PoolType::SushiSwapV2 => Pool::SushiSwapV2(pool),
            PoolType::PancakeSwapV2 => Pool::PancakeSwapV2(pool),
            PoolType::Aerodrome => Pool::Aerodrome(pool),
            PoolType::BaseSwapV2 => Pool::BaseSwapV2(pool),
            PoolType::SwapBasedV2 => Pool::SwapBasedV2(pool),
            PoolType::DackieSwapV2 => Pool::DackieSwapV2(pool),
            PoolType::AlienBaseV2 => Pool::AlienBaseV2(pool),
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
            PoolType::SwapBasedV3 => Pool::SwapBasedV3(pool),
            PoolType::DackieSwapV3 => Pool::DackieSwapV3(pool),
            PoolType::AlienBaseV3 => Pool::AlienBaseV3(pool),
            _ => panic!("Invalid pool type"),
        }
    }

    pub fn new_maverick(pool_type: PoolType, pool: MaverickPool) -> Self {
        match pool_type {
            PoolType::MaverickV1 => Pool::MaverickV1(pool),
            PoolType::MaverickV2 => Pool::MaverickV2(pool),
            _ => panic!("Invalid pool type"),
        }
    }

    pub fn new_curve_two(pool_type: PoolType, pool: CurveTwoCryptoPool) -> Self {
        match pool_type {
            PoolType::CurveTwoCrypto => Pool::CurveTwoCrypto(pool),
            _ => panic!("Invalid pool type"),
        }
    }

    pub fn new_curve_tri(pool_type: PoolType, pool: CurveTriCryptoPool) -> Self {
        match pool_type {
            PoolType::CurveTriCrypto => Pool::CurveTriCrypto(pool),
            _ => panic!("Invalid pool type"),
        }
    }

    pub fn new_balancer(pool_type: PoolType, pool: BalancerV2Pool) -> Self {
        match pool_type {
            PoolType::BalancerV2 => Pool::BalancerV2(pool),
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
            Pool::AlienBaseV2(_) => true,
            Pool::SwapBasedV2(_) => true,
            Pool::DackieSwapV2(_) => true,
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
            Pool::AlienBaseV3(_) => true,
            Pool::SwapBasedV3(_) => true,
            Pool::DackieSwapV3(_) => true,
            _ => false,
        }
    }

    pub fn is_maverick(&self) -> bool {
        match self {
            Pool::MaverickV1(_) => true,
            Pool::MaverickV2(_) => true,
            _ => false,
        }
    }

    pub fn is_curve_two(&self) -> bool {
        match self {
            Pool::CurveTwoCrypto(_) => true,
            _ => false,
        }
    }

    pub fn is_curve_tri(&self) -> bool {
        match self {
            Pool::CurveTriCrypto(_) => true,
            _ => false,
        }
    }

    pub fn is_balancer(&self) -> bool {
        match self {
            Pool::BalancerV2(_) => true,
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
            Pool::SwapBasedV2(pool) => Some(pool),
            Pool::DackieSwapV2(pool) => Some(pool),
            Pool::AlienBaseV2(pool) => Some(pool),
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
            Pool::AlienBaseV3(pool) => Some(pool),
            Pool::SwapBasedV3(pool) => Some(pool),
            Pool::DackieSwapV3(pool) => Some(pool),
            _ => None,
        }
    }

    pub fn get_maverick(&self) -> Option<&MaverickPool> {
        match self {
            Pool::MaverickV1(pool) => Some(pool),
            Pool::MaverickV2(pool) => Some(pool),
            _ => None,
        }
    }

    pub fn get_curve_two(&self) -> Option<&CurveTwoCryptoPool> {
        match self {
            Pool::CurveTwoCrypto(pool) => Some(pool),
            _ => None,
        }
    }

    pub fn get_curve_tri(&self) -> Option<&CurveTriCryptoPool> {
        match self {
            Pool::CurveTriCrypto(pool) => Some(pool),
            _ => None,
        }
    }

    pub fn get_balancer(&self) -> Option<&BalancerV2Pool> {
        match self {
            Pool::BalancerV2(pool) => Some(pool),
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
            Pool::AlienBaseV2(pool) => Some(pool),
            Pool::SwapBasedV2(pool) => Some(pool),
            Pool::DackieSwapV2(pool) => Some(pool),
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
            Pool::AlienBaseV3(pool) => Some(pool),
            Pool::SwapBasedV3(pool) => Some(pool),
            Pool::DackieSwapV3(pool) => Some(pool),
            _ => None,
        }
    }

    pub fn get_maverick_mut(&mut self) -> Option<&mut MaverickPool> {
        match self {
            Pool::MaverickV1(pool) => Some(pool),
            Pool::MaverickV2(pool) => Some(pool),
            _ => None,
        }
    }

    pub fn get_curve_two_mut(&mut self) -> Option<&mut CurveTwoCryptoPool> {
        match self {
            Pool::CurveTwoCrypto(pool) => Some(pool),
            _ => None,
        }
    }

    pub fn get_curve_tri_mut(&mut self) -> Option<&mut CurveTriCryptoPool> {
        match self {
            Pool::CurveTriCrypto(pool) => Some(pool),
            _ => None,
        }
    }

    pub fn get_balancer_mut(&mut self) -> Option<&mut BalancerV2Pool> {
        match self {
            Pool::BalancerV2(pool) => Some(pool),
            _ => None,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.address() != Address::ZERO && 
        self.token0_address() != Address::ZERO && 
        self.token1_address() != Address::ZERO
    }

    fn update_token0_name(pool: &mut Pool, token0: String) {
        if pool.is_v2() {
            let pool = pool.get_v2_mut().unwrap();
            pool.token0_name = token0;
        } else if pool.is_v3() {
            let pool = pool.get_v3_mut().unwrap();
            pool.token0_name = token0;
        } else if pool.is_curve_two() {
            let pool = pool.get_curve_two_mut().unwrap();
            pool.token0_name = token0;
        } else if pool.is_curve_tri() {
            let pool = pool.get_curve_tri_mut().unwrap();
            pool.token0_name = token0;
        } else if pool.is_balancer() {
            let pool = pool.get_balancer_mut().unwrap();
            pool.token0_name = token0;
        } else if pool.is_maverick() {
            let pool = pool.get_maverick_mut().unwrap();
            pool.token0_name = token0;
        }
    }

    pub fn update_token1_name(pool: &mut Pool, token1: String) {
        if pool.is_v2() {
            let pool = pool.get_v2_mut().unwrap();
            pool.token1_name = token1;
        } else if pool.is_v3() {
            let pool = pool.get_v3_mut().unwrap();
            pool.token1_name = token1;
        } else if pool.is_curve_two() {
            let pool = pool.get_curve_two_mut().unwrap();
            pool.token1_name = token1;
        } else if pool.is_curve_tri() {
            let pool = pool.get_curve_tri_mut().unwrap();
            pool.token1_name = token1;
        }else if pool.is_balancer() {
            let pool = pool.get_balancer_mut().unwrap();
            pool.token1_name = token1;
        } else if pool.is_maverick() {
            let pool = pool.get_maverick_mut().unwrap();
            pool.token1_name = token1;
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
    DackieSwapV3
);


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

    /// Get the DynSolType for the pool
    fn get_pool_repr(&self) -> DynSolType;
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
