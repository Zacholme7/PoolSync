use crate::pools::gen::PancakeSwapV3Factory;
use alloy::primitives::{address, Address};
use alloy_sol_types::SolEvent;
use crate::pools::PoolFetcher;
use alloy::primitives::Log;
use crate::pools::PoolType;
use crate::Chain;

pub struct PancakeSwapV3Fetcher;

impl PoolFetcher for PancakeSwapV3Fetcher {
    fn pool_type(&self) -> PoolType {
        PoolType::PancakeSwapV3
    }

    fn factory_address(&self, chain: Chain) -> Address {
        match chain {
            Chain::Ethereum => address!("0BFbCF9fa4f9C56B0F40a671Ad40E0805A091865"),
            Chain::Base => address!("0BFbCF9fa4f9C56B0F40a671Ad40E0805A091865"),
        }
    }
    
    fn pair_created_signature(&self) -> &str {
        PancakeSwapV3Factory::PoolCreated::SIGNATURE
    }

    fn log_to_address(&self, log: &Log) -> Address {
        let decoded_log = PancakeSwapV3Factory::PoolCreated::decode_log(log, false).unwrap();
        decoded_log.data.pool
    }
}