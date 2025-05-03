use crate::onchain::BaseSwapV2Factory;
use crate::{Chain, PoolFetcher};
use alloy_primitives::{address, Address, Log};
use alloy_sol_types::SolEvent;

pub struct BaseSwapV2Fetcher;

impl PoolFetcher for BaseSwapV2Fetcher {
    fn factory_address(&self, chain: Chain) -> Address {
        match chain {
            Chain::Base => address!("FDa619b6d20975be80A10332cD39b9a4b0FAa8BB"),
            _ => panic!("BaseSwap not supported on this chain"),
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
