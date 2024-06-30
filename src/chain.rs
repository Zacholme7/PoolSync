use crate::PoolType;
use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};
use std::fmt;

/// The chains that are supported 
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Chain {
        Ethereum,
        Base,
        // ..
}

/// Mapping from the chain to the protocols that are supported on the chain
/// 
/// This is important because not all protocols are deployed on all chains and the 
/// contract addresses for the same protocol are different across chains
static CHAIN_POOLS: Lazy<HashMap<Chain, HashSet<PoolType>>> = Lazy::new(|| {
        let mut m = std::collections::HashMap::new();

        // Protocols supported by Ethereum
        m.insert(Chain::Ethereum, [
                PoolType::UniswapV2, 
                PoolType::UniswapV3, 
                PoolType::SushiSwap
        ].iter().cloned().collect());

        // Protocols supported by Base
        m.insert(Chain::Base, [
                PoolType::UniswapV2, 
                PoolType::UniswapV3
        ].iter().cloned().collect());

        // ...

        m
});

impl Chain {
        /// Given a protocol, determine if it is suppored
        pub fn supported(&self, pool_type: &PoolType) -> bool {
            CHAIN_POOLS.get(self)
                .map(|pools| pools.contains(&pool_type))
                .unwrap_or(false)
        }
}

// Display for chain, used for file naming and debugging purposes
impl fmt::Display for Chain {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{:?}", self)
        }
}






