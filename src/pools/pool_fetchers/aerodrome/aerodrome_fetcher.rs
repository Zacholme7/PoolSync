use alloy_dyn_abi::DynSolType;
use alloy_primitives::{address, Address, Log};
use alloy_sol_types::SolEvent;

use crate::onchain::AerodromeV2Factory;
use crate::pools::{PoolFetcher, PoolType};
use crate::Chain;

pub struct AerodromeFetcher;

impl PoolFetcher for AerodromeFetcher {
    fn pool_type(&self) -> PoolType {
        PoolType::Aerodrome
    }

    fn factory_address(&self, chain: Chain) -> Address {
        match chain {
            Chain::Base => address!("420DD381b31aEf6683db6B902084cB0FFECe40Da"),
            _ => panic!("Aerodome not supported on this chain"),
        }
    }

    fn pair_created_signature(&self) -> &str {
        AerodromeV2Factory::PoolCreated::SIGNATURE
    }

    fn log_to_address(&self, log: &Log) -> Address {
        let decoded_log = AerodromeV2Factory::PoolCreated::decode_log(log).unwrap();
        decoded_log.data.pool
    }

    fn get_pool_repr(&self) -> DynSolType {
        DynSolType::Array(Box::new(DynSolType::Tuple(vec![
            DynSolType::Address,
            DynSolType::Address,
            DynSolType::Address,
            DynSolType::Uint(8),
            DynSolType::Uint(8),
            DynSolType::Uint(256),
            DynSolType::Uint(256),
        ])))
    }
}
