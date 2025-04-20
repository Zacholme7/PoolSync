use crate::onchain::AlienBaseV3Factory;
use crate::pools::PoolFetcher;
use crate::pools::PoolType;
use crate::Chain;
use alloy_dyn_abi::DynSolType;
use alloy_primitives::Log;
use alloy_primitives::{address, Address};
use alloy_sol_types::SolEvent;

pub struct AlienBaseV3Fetcher;

impl PoolFetcher for AlienBaseV3Fetcher {
    fn pool_type(&self) -> PoolType {
        PoolType::AlienBaseV3
    }

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

    fn get_pool_repr(&self) -> DynSolType {
        DynSolType::Array(Box::new(DynSolType::Tuple(vec![
            DynSolType::Address,
            DynSolType::Address,
            DynSolType::Uint(8),
            DynSolType::Address,
            DynSolType::Uint(8),
            DynSolType::Uint(128),
            DynSolType::Uint(160),
            DynSolType::Int(24),
            DynSolType::Int(24),
            DynSolType::Uint(24),
            DynSolType::Int(128),
        ])))
    }
}
