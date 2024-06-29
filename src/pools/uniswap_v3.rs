use alloy::primitives::address;
use alloy::primitives::{Address, Log};
use alloy::sol_types::{sol, SolEvent};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::pools::{Pool, PoolFetcher, PoolType};
use crate::chain::Chain;

sol! {
    #[derive(Debug)]
    #[sol(rpc)]
    contract UniswapV3Factory {
        event PoolCreated(
            address indexed token0,
            address indexed token1,
            uint24 indexed fee,
            int24 tickSpacing,
            address pool
        );
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UniswapV3Pool {
    pub address: Address,
    pub token0: Address,
    pub token1: Address,
    pub fee: u32,
    pub tick_spacing: i32,
}

pub struct UniswapV3Fetcher;

#[async_trait]
impl PoolFetcher for UniswapV3Fetcher {
    fn pool_type(&self) -> PoolType {
        PoolType::UniswapV3
    }

    fn factory_address(&self, chain: Chain) -> Address {
        match chain {
                Chain::Ethereum => address!("1F98431c8aD98523631AE4a59f267346ea31F984"),
                Chain::Base => address!("33128a8fC17869897dcE68Ed026d694621f6FDfD"),
                _ => panic!("Protocol not suppored for this chain")
        }
    }

    fn pair_created_signature(&self) -> &str {
        UniswapV3Factory::PoolCreated::SIGNATURE
    }

    async fn from_log(&self, log: &Log) -> Option<Pool> {
        let decoded_log = UniswapV3Factory::PoolCreated::decode_log(log, false).ok()?;
        Some(Pool::UniswapV3(UniswapV3Pool {
            address: decoded_log.data.pool,
            token0: decoded_log.data.token0,
            token1: decoded_log.data.token1,
            fee: decoded_log.data.fee,
            tick_spacing: decoded_log.data.tickSpacing,
        }))
    }
}
