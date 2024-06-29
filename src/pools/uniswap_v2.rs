use alloy::primitives::{address, Address, Log};
use alloy::sol_types::{sol, SolEvent};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::pools::{Pool, PoolFetcher, PoolType};
use crate::chain::Chain;

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    contract UniswapV2Factory  {
        event PairCreated(address indexed token0, address indexed token1, address pair, uint256);
    }
);

/// A UniswapV2 AMM/pool
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UniswapV2Pool {
    pub address: Address,
    token0: Address,
    token1: Address,
}

/// Fetcher implementation for uniswapv2
pub struct UniswapV2Fetcher;
#[async_trait]
impl PoolFetcher for UniswapV2Fetcher {
    /// Return the type of the pool
    fn pool_type(&self) -> PoolType {
        PoolType::UniswapV2
    }
    /// Return the factory address
    fn factory_address(&self, chain: Chain) -> Address {
        match chain {
                Chain::Ethereum => address!("5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f"),
                Chain::Base => address!("8909Dc15e40173Ff4699343b6eB8132c65e18eC6"),
                _ => panic!("Protocol not supported for this chain")
        }
    }
    /// Return the pair created signature
    fn pair_created_signature(&self) -> &str {
        UniswapV2Factory::PairCreated::SIGNATURE
    }
    /// Given a pair created log, decode it and construct a pool
    async fn from_log(&self, log: &Log) -> Option<Pool> {
        let decoded_log = UniswapV2Factory::PairCreated::decode_log(log, false).unwrap();
        Some(Pool::UniswapV2(UniswapV2Pool {
            address: decoded_log.data.pair,
            token0: decoded_log.data.token0,
            token1: decoded_log.data.token1,
        }))
    }
}
