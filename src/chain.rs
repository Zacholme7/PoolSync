//! Chain Support and Pool Type Management
//!
//! This module defines the supported blockchain networks (Chains) and manages
//! the mapping of supported pool types for each chain.

use crate::PoolType;
use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};
use std::fmt;

/// Enum representing supported blockchain networks
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Chain {
    /// Ethereum mainnet
    Ethereum,
    /// Base chain
    Base,
    // Additional chains can be added here
}

/// Static mapping of supported pool types for each chain
///
/// This mapping is important because not all protocols are deployed on all chains,
/// and the contract addresses for the same protocol may differ across chains.
static CHAIN_POOLS: Lazy<HashMap<Chain, HashSet<PoolType>>> = Lazy::new(|| {
    let mut m = HashMap::new();

    // Protocols supported by Ethereum
    m.insert(
        Chain::Ethereum,
        [
            PoolType::UniswapV2,
            PoolType::UniswapV3,
            PoolType::SushiSwap,
        ]
        .iter()
        .cloned()
        .collect(),
    );

    // Protocols supported by Base
    m.insert(
        Chain::Base,
        [PoolType::UniswapV2, PoolType::UniswapV3]
            .iter()
            .cloned()
            .collect(),
    );

    // Additional chains can be configured here

    m
});

impl Chain {
    /// Determines if a given pool type is supported on this chain
    pub fn supported(&self, pool_type: &PoolType) -> bool {
        CHAIN_POOLS
            .get(self)
            .map(|pools| pools.contains(pool_type))
            .unwrap_or(false)
    }
}

// Display implementation for Chain, used for file naming and debugging purposes
impl fmt::Display for Chain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
