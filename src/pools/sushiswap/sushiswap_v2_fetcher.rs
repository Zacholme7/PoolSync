use crate::pools::gen::SushiSwapV2Factory;
use alloy::primitives::{address, Address};
use alloy_sol_types::SolEvent;
use crate::pools::PoolFetcher;
use alloy::primitives::Log;
use crate::pools::PoolType;
use crate::Chain;
pub struct SushiSwapV2Fetcher;

impl PoolFetcher for SushiSwapV2Fetcher {
    fn pool_type(&self) -> PoolType {
        PoolType::SushiSwap
    }

    fn factory_address(&self, chain: Chain) -> Address {
        match chain {
            Chain::Ethereum => address!("C0AEe478e3658e2610c5F7A4A2E1777cE9e4f2Ac"),
            Chain::Base => address!("71524B4f93c58fcbF659783284E38825f0622859"),
        }
    }
    
    fn pair_created_signature(&self) -> &str {
        SushiSwapV2Factory::PairCreated::SIGNATURE
    }

    fn log_to_address(&self, log: &Log) -> Address {
        let decoded_log = SushiSwapV2Factory::PairCreated::decode_log(log, false).unwrap();
        decoded_log.data.pair
    }
}