use crate::onchain::SlipstreamFactory;
use crate::{Chain, PoolFetcher};
use alloy_primitives::{address, Address, Log};
use alloy_sol_types::SolEvent;

pub struct SlipstreamFetcher;

impl PoolFetcher for SlipstreamFetcher {
    fn factory_address(&self, chain: Chain) -> Address {
        match chain {
            Chain::Base => address!("5e7BB104d84c7CB9B682AaC2F3d509f5F406809A"),
            _ => panic!("Aerodome not supported on this chain"),
        }
    }

    fn pair_created_signature(&self) -> &str {
        SlipstreamFactory::PoolCreated::SIGNATURE
    }

    fn log_to_address(&self, log: &Log) -> Address {
        let decoded_log = SlipstreamFactory::PoolCreated::decode_log(log).unwrap();
        decoded_log.data.pool
    }
}
