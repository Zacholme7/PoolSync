//! Custom fetching implementations for each pool types.
//! Each pool must implement PoolFetcher which defines
//! all of the logic needed for fetching and parsing a pool
//! variant

pub use aerodrome::{AerodromeFetcher, SlipstreamFetcher};
pub use alien_base::{AlienBaseV2Fetcher, AlienBaseV3Fetcher};
pub use balancer::BalancerV2Fetcher;
pub use base_swap::{BaseSwapV2Fetcher, BaseSwapV3Fetcher};
pub use curve::{CurveTriCryptoFetcher, CurveTwoCryptoFetcher};
pub use dackie_swap::{DackieSwapV2Fetcher, DackieSwapV3Fetcher};
pub use maverick::{MaverickV1Fetcher, MaverickV2Fetcher};
pub use pancake_swap::{PancakeSwapV2Fetcher, PancakeSwapV3Fetcher};
pub use sushiswap::{SushiSwapV2Fetcher, SushiSwapV3Fetcher};
pub use swap_based::{SwapBasedV2Fetcher, SwapBasedV3Fetcher};
pub use uniswap::{UniswapV2Fetcher, UniswapV3Fetcher};

mod aerodrome;
mod alien_base;
mod balancer;
mod base_swap;
mod curve;
mod dackie_swap;
mod maverick;
mod pancake_swap;
mod sushiswap;
mod swap_based;
mod uniswap;

use crate::Chain;
use alloy_primitives::{Address, Log};

/// Defines common functionality for fetching and decoding pool creation events
///
/// This trait provides a unified interface for different pool types to implement
/// their specific logic for identifying and parsing pool creation events.
pub trait PoolFetcher: Send + Sync {
    /// Returns the factory address for the given chain
    fn factory_address(&self, chain: Chain) -> Address;

    /// Returns the event signature for pool creation
    fn pair_created_signature(&self) -> &str;

    /// Attempts to create a `Pool` instance from a log entry
    fn log_to_address(&self, log: &Log) -> Address;
}
