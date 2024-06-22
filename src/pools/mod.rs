use alloy::primitives::Address;
use serde::{Serialize, Deserialize};
use alloy::providers::RootProvider;
use alloy::transports::http::{Client, Http};
use std::sync::Arc;
use uniswap_v2::UniswapV2Pool;

mod uniswap_v2;
//use uniswap_v2::UniswapV2Pool;

/// A ERC20 token
//#[derive(Debug, Clone, Serialize, Deserialize)]
struct Token {
    address: Address,
    name: String,
    decimals: u8,
}

// The different type of pools/protocols supported
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum PoolType {
    UniswapV2,
    // add other pools here
}

impl PoolType {
    // Dispatch a call to the correct protocol variant
    pub async fn get_all_pools(&self, provider: Arc<RootProvider<Http<Client>>>) -> Vec<Pool> {
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
    pub fn address(&self) -> Address {
        match self {
            Pool::UniswapV2(uniswap_v2_pool) => uniswap_v2_pool.address,
            _ => panic!()
        }
    }
}




