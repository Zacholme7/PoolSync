use alloy_primitives::{address, Address, Log};
use alloy_sol_types::SolEvent;

use crate::onchain::BalancerV2Factory;
use crate::pools::PoolFetcher;
use crate::Chain;

pub struct BalancerV2Fetcher;

impl PoolFetcher for BalancerV2Fetcher {
    fn factory_address(&self, chain: Chain) -> Address {
        match chain {
            Chain::Ethereum => address!("897888115Ada5773E02aA29F775430BFB5F34c51"),
            Chain::Base => address!("4C32a8a8fDa4E24139B51b456B42290f51d6A1c4"),
        }
    }

    fn pair_created_signature(&self) -> &str {
        BalancerV2Factory::PoolCreated::SIGNATURE
    }

    fn log_to_address(&self, log: &Log) -> Address {
        let decoded_log = BalancerV2Factory::PoolCreated::decode_log(log).unwrap();
        decoded_log.data.pool
    }
}
