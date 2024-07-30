use crate::pools::gen::SushiSwapV3Factory;
use alloy::primitives::{address, Address};
use alloy_sol_types::SolEvent;
use crate::pools::PoolFetcher;
use alloy::primitives::Log;
use crate::pools::PoolType;
use crate::Chain;

pub struct SushiSwapV3Fetcher;

impl PoolFetcher for SushiSwapV3Fetcher {
    fn pool_type(&self) -> PoolType {
        PoolType::SushiSwapV3
    }

    fn factory_address(&self, chain: Chain) -> Address {
        match chain {
            Chain::Ethereum => address!("bACEB8eC6b9355Dfc0269C18bac9d6E2Bdc29C4F"),
            Chain::Base => address!("c35DADB65012eC5796536bD9864eD8773aBc74C4"),
        }
    }
    
    fn pair_created_signature(&self) -> &str {
        SushiSwapV3Factory::PoolCreated::SIGNATURE
    }

    fn log_to_address(&self, log: &Log) -> Address {
        let decoded_log = SushiSwapV3Factory::PoolCreated::decode_log(log, false).unwrap();
        decoded_log.data.pool
    }
}