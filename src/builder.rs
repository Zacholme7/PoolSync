//! PoolSync Builder Implementation
//!
//! This module provides a builder pattern for constructing a PoolSync instance,
//! allowing for flexible configuration of pool types and chains to be synced.

use crate::pools::pool_fetchers::{
    AerodromeFetcher, AlienBaseV2Fetcher, AlienBaseV3Fetcher, BalancerV2Fetcher, BaseSwapV2Fetcher,
    BaseSwapV3Fetcher, CurveTriCryptoFetcher, CurveTwoCryptoFetcher, DackieSwapV2Fetcher,
    DackieSwapV3Fetcher, MaverickV1Fetcher, MaverickV2Fetcher, PancakeSwapV2Fetcher,
    PancakeSwapV3Fetcher, SlipstreamFetcher, SushiSwapV2Fetcher, SushiSwapV3Fetcher,
    SwapBasedV2Fetcher, SwapBasedV3Fetcher, UniswapV2Fetcher, UniswapV3Fetcher,
};

use crate::errors::*;
use crate::pools::*;
use crate::{Chain, PoolSync, PoolType};
use std::collections::HashMap;
use std::sync::Arc;

/// Builder for constructing a PoolSync instance
#[derive(Default)]
pub struct PoolSyncBuilder {
    /// Mapping from the pool type to the implementation of its fetcher
    fetchers: HashMap<PoolType, Arc<dyn PoolFetcher>>,
    /// The chain to be synced on
    chain: Option<Chain>,
    /// Rate limit on the rpc endpoint
    rate_limit: Option<usize>,
}

impl PoolSyncBuilder {
    /// Adds a new pool type to be synced
    /// The builder instance for method chaining
    pub fn add_pool(mut self, pool_type: PoolType) -> Self {
        match pool_type {
            PoolType::UniswapV2 => {
                self.fetchers
                    .insert(PoolType::UniswapV2, Arc::new(UniswapV2Fetcher));
            }
            PoolType::UniswapV3 => {
                self.fetchers
                    .insert(PoolType::UniswapV3, Arc::new(UniswapV3Fetcher));
            }
            PoolType::SushiSwapV2 => {
                self.fetchers
                    .insert(PoolType::SushiSwapV2, Arc::new(SushiSwapV2Fetcher));
            }
            PoolType::SushiSwapV3 => {
                self.fetchers
                    .insert(PoolType::SushiSwapV3, Arc::new(SushiSwapV3Fetcher));
            }
            PoolType::PancakeSwapV2 => {
                self.fetchers
                    .insert(PoolType::PancakeSwapV2, Arc::new(PancakeSwapV2Fetcher));
            }
            PoolType::PancakeSwapV3 => {
                self.fetchers
                    .insert(PoolType::PancakeSwapV3, Arc::new(PancakeSwapV3Fetcher));
            }
            PoolType::Aerodrome => {
                self.fetchers
                    .insert(PoolType::Aerodrome, Arc::new(AerodromeFetcher));
            }
            PoolType::Slipstream => {
                self.fetchers
                    .insert(PoolType::Slipstream, Arc::new(SlipstreamFetcher));
            }
            PoolType::BaseSwapV2 => {
                self.fetchers
                    .insert(PoolType::BaseSwapV2, Arc::new(BaseSwapV2Fetcher));
            }
            PoolType::BaseSwapV3 => {
                self.fetchers
                    .insert(PoolType::BaseSwapV3, Arc::new(BaseSwapV3Fetcher));
            }
            PoolType::AlienBaseV2 => {
                self.fetchers
                    .insert(PoolType::AlienBaseV2, Arc::new(AlienBaseV2Fetcher));
            }
            PoolType::AlienBaseV3 => {
                self.fetchers
                    .insert(PoolType::AlienBaseV3, Arc::new(AlienBaseV3Fetcher));
            }
            PoolType::MaverickV1 => {
                self.fetchers
                    .insert(PoolType::MaverickV1, Arc::new(MaverickV1Fetcher));
            }
            PoolType::MaverickV2 => {
                self.fetchers
                    .insert(PoolType::MaverickV2, Arc::new(MaverickV2Fetcher));
            }
            PoolType::CurveTwoCrypto => {
                self.fetchers
                    .insert(PoolType::CurveTwoCrypto, Arc::new(CurveTwoCryptoFetcher));
            }
            PoolType::CurveTriCrypto => {
                self.fetchers
                    .insert(PoolType::CurveTriCrypto, Arc::new(CurveTriCryptoFetcher));
            }
            PoolType::BalancerV2 => {
                self.fetchers
                    .insert(PoolType::BalancerV2, Arc::new(BalancerV2Fetcher));
            }
            PoolType::SwapBasedV2 => {
                self.fetchers
                    .insert(PoolType::SwapBasedV2, Arc::new(SwapBasedV2Fetcher));
            }
            PoolType::SwapBasedV3 => {
                self.fetchers
                    .insert(PoolType::SwapBasedV3, Arc::new(SwapBasedV3Fetcher));
            }
            PoolType::DackieSwapV2 => {
                self.fetchers
                    .insert(PoolType::DackieSwapV2, Arc::new(DackieSwapV2Fetcher));
            }
            PoolType::DackieSwapV3 => {
                self.fetchers
                    .insert(PoolType::DackieSwapV3, Arc::new(DackieSwapV3Fetcher));
            }
        }
        self
    }

    /// Add multiple pools to be synced
    pub fn add_pools(mut self, pools: &[PoolType]) -> Self {
        for pool in pools.iter() {
            self = self.add_pool(*pool);
        }
        self
    }

    /// Sets the chain to sync on
    /// The builder instance for method chaining
    pub fn chain(mut self, chain: Chain) -> Self {
        self.chain = Some(chain);
        self
    }

    /// Set the rate limit of the rpc
    /// The builder instance for method chaining
    pub fn rate_limit(mut self, rate_limit: usize) -> Self {
        self.rate_limit = Some(rate_limit);
        self
    }

    /// Consumes the builder and produces a constructed PoolSync
    pub fn build(self) -> Result<PoolSync, PoolSyncError> {
        // Ensure the chain is set
        let chain = self.chain.ok_or(PoolSyncError::ChainNotSet)?;

        // Ensure all the pools are supported
        for pool_type in self.fetchers.keys() {
            if !chain.supported(pool_type) {
                return Err(PoolSyncError::UnsupportedPoolType);
            }
        }

        // set rate limit to user defined if specified, otherwise set high value
        // that will not be hit to simulate unlimited requests
        let rate_limit = self.rate_limit.unwrap_or(10000) as u64;

        // Construct PoolSync
        Ok(PoolSync {
            fetchers: self.fetchers,
            rate_limit,
            chain,
        })
    }
}
