use alloy::primitives::Address;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use alloy::providers::RootProvider;
use alloy::transports::http::{Client, Http};

mod uniswap_v2;
use uniswap_v2::UniswapV2Pool;

/// A ERC20 token
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Token {
    address: Address,
    name: String,
    decimals: u8,
}

// The different type of pools/protocols supported
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum PoolType {
    UniswapV2(UniswapV2Pool),
    // add other pools here
}

impl PoolType {
    pub fn sync(&self, provider: &RootProvider<Http<Client>>) {
        match self {
            PoolType::UniswapV2 => UniswapV2Factory::get_all_pools(&provider),
            _ => {}
        }
    }
}

// Common trait among all pools
#[async_trait]
pub trait Pool: Send + Sync {
    async fn get_all_pools(&self, provider: RootProvider<Http<Client>>) ;
}
