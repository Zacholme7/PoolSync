use crate::onchain::BaseSwapV3Factory;
use crate::{Chain, PoolFetcher};
use alloy_primitives::{address, Address, Log};
use alloy_sol_types::SolEvent;

pub struct BaseSwapV3Fetcher;

impl PoolFetcher for BaseSwapV3Fetcher {
    fn factory_address(&self, chain: Chain) -> Address {
        match chain {
            Chain::Base => address!("38015D05f4fEC8AFe15D7cc0386a126574e8077B"),
            _ => panic!("Aerodome not supported on this chain"),
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
