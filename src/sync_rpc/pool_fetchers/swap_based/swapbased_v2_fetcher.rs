use crate::onchain::BaseSwapV2Factory;
use crate::{Chain, PoolFetcher};
use alloy_primitives::{address, Address, Log};
use alloy_sol_types::SolEvent;

pub struct SwapBasedV2Fetcher;
impl PoolFetcher for SwapBasedV2Fetcher {
    fn factory_address(&self, chain: Chain) -> Address {
        match chain {
            Chain::Base => address!("04C9f118d21e8B767D2e50C946f0cC9F6C367300"),
            _ => panic!("SwapBased not supported on this chain"),
        }
    }

    fn pair_created_signature(&self) -> &str {
        BaseSwapV2Factory::PairCreated::SIGNATURE
    }

    fn log_to_address(&self, log: &Log) -> Address {
        let decoded_log = BaseSwapV2Factory::PairCreated::decode_log(log).unwrap();
        decoded_log.data.pair
    }
}
