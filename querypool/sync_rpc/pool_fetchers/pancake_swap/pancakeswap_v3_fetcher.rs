use crate::onchain::PancakeSwapV3Factory;
use crate::{Chain, PoolFetcher};
use alloy_primitives::{address, Address, Log};
use alloy_sol_types::SolEvent;

pub struct PancakeSwapV3Fetcher;
impl PoolFetcher for PancakeSwapV3Fetcher {
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
        let decoded_log = PancakeSwapV3Factory::PoolCreated::decode_log(log).unwrap();
        decoded_log.data.pool
    }
}
