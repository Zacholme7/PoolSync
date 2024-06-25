use alloy::providers::Provider;
use alloy::network::Ethereum;
use alloy::transports::Transport;

use alloy::primitives::Address;
use std::sync::Arc;
use uniswap_v2::UniswapV2Pool;
use serde::{Serialize, Deserialize};

mod uniswap_v2;

// The different type of pools/protocols supported
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum PoolType {
    UniswapV2,
    // add other pools here
}

impl PoolType {
    // Dispatch a call to the correct protocol variant
    pub async fn get_all_pools<P, T>(&self, provider: Arc<P>) -> Vec<Pool> 
    where 
        P: Provider<T, Ethereum> + 'static,
        T: Transport + Clone + 'static
    {
        match self {
            PoolType::UniswapV2 => UniswapV2Pool::get_all_pools(provider).await,
            _ => panic!("Not supported")
        }
    }
}

/// Common enum to link all protocols
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Pool {
    UniswapV2(UniswapV2Pool),
    // other pools
}

impl Pool {
    // common functionality that all pools support
    pub fn address(&self) -> Address {
        match self {
            Pool::UniswapV2(uniswap_v2_pool) => uniswap_v2_pool.address,
            _ => panic!()
        }
    }
}




