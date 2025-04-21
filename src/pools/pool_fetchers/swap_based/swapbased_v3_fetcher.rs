use crate::onchain::BaseSwapV3Factory;
use crate::pools::PoolFetcher;
use crate::Chain;
use alloy_primitives::{address, Address, Log};
use alloy_sol_types::SolEvent;

pub struct SwapBasedV3Fetcher;

impl PoolFetcher for SwapBasedV3Fetcher {
    fn factory_address(&self, chain: Chain) -> Address {
        match chain {
            Chain::Base => address!("b5620F90e803C7F957A9EF351B8DB3C746021BEa"),
            _ => panic!("SwapBased not supported on this chain"),
        }
    }

    fn pair_created_signature(&self) -> &str {
        BaseSwapV3Factory::PoolCreated::SIGNATURE
    }

    fn log_to_address(&self, log: &Log) -> Address {
        let decoded_log = BaseSwapV3Factory::PoolCreated::decode_log(log).unwrap();
        decoded_log.data.pool
    }
}
