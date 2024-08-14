use alloy::primitives::{address, Address};
use alloy_sol_types::SolEvent;
use crate::pools::gen::AerodromeV2Factory;
use crate::pools::PoolFetcher;
use alloy::primitives::Log;
use crate::pools::PoolType;
use crate::Chain;

pub struct AerodromeFetcher;


impl PoolFetcher for AerodromeFetcher {
    fn pool_type(&self) -> PoolType {
        PoolType::Aerodrome
    }

    fn factory_address(&self, chain: Chain) -> Address {
        match chain {
            Chain::Base => address!("420DD381b31aEf6683db6B902084cB0FFECe40Da"),
            _ => panic!("Aerodome not supported on this chain")
        }
    }

    fn pair_created_signature(&self) -> &str {
        AerodromeV2Factory::PoolCreated::SIGNATURE
    }

    fn log_to_address(&self, log: &Log) -> Address {
        let decoded_log = AerodromeV2Factory::PoolCreated::decode_log(log, false).unwrap();
        decoded_log.data.pool
    }
}