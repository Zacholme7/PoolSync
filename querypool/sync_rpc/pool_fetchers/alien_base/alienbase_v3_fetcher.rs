use crate::onchain::AlienBaseV3Factory;
use crate::{Chain, PoolFetcher};
use alloy_primitives::{address, Address, Log};
use alloy_sol_types::SolEvent;

pub struct AlienBaseV3Fetcher;
impl PoolFetcher for AlienBaseV3Fetcher {
    fn factory_address(&self, chain: Chain) -> Address {
        match chain {
            Chain::Base => address!("0Fd83557b2be93617c9C1C1B6fd549401C74558C"),
            _ => panic!("Alienbase not supported on this chain"),
        }
    }

    fn pair_created_signature(&self) -> &str {
        AlienBaseV3Factory::PoolCreated::SIGNATURE
    }

    fn log_to_address(&self, log: &Log) -> Address {
        let decoded_log = AlienBaseV3Factory::PoolCreated::decode_log(log).unwrap();
        decoded_log.data.pool
    }
}
