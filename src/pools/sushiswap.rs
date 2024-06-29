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
    contract SushiSwapFactory {
        event PairCreated(address indexed token0, address indexed token1, address pair, uint256);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SushiSwapPool {
    pub address: Address,
    pub token0: Address,
    pub token1: Address,
}

pub struct SushiSwapFetcher;

#[async_trait]
impl  PoolFetcher for SushiSwapFetcher {
    fn pool_type(&self) -> PoolType {
        PoolType::SushiSwap
    }

    fn factory_address(&self, chain: Chain) -> Address {
        match chain {
                Chain::Ethereum => address!("C0AEe478e3658e2610c5F7A4A2E1777cE9e4f2Ac"),
                _ => panic!("Protocol is not supported for the chain")
        }
    }

    fn pair_created_signature(&self) -> &str {
        SushiSwapFactory::PairCreated::SIGNATURE
    }

    async fn from_log(&self, log: &Log) -> Option<Pool> {
        let decoded_log = SushiSwapFactory::PairCreated::decode_log(log, false).ok()?;
        Some(Pool::SushiSwap(SushiSwapPool {
            address: decoded_log.data.pair,
            token0: decoded_log.data.token0,
            token1: decoded_log.data.token1,
        }))
    }
}
