use alloy::primitives::{address, Address};
use alloy_sol_types::SolEvent;
use crate::pools::gen::SlipstreamFactory;
use crate::pools::PoolFetcher;
use alloy::dyn_abi::DynSolType;
use alloy::primitives::Log;
use crate::pools::PoolType;
use crate::Chain;

pub struct SlipstreamFetcher;


impl PoolFetcher for SlipstreamFetcher {
    fn pool_type(&self) -> PoolType {
        PoolType::Slipstream
    }

    fn factory_address(&self, chain: Chain) -> Address {
        match chain {
            Chain::Base => address!("5e7BB104d84c7CB9B682AaC2F3d509f5F406809A"),
            _ => panic!("Aerodome not supported on this chain")
        }
    }

    fn pair_created_signature(&self) -> &str {
        SlipstreamFactory::PoolCreated::SIGNATURE
    }

    fn log_to_address(&self, log: &Log) -> Address {
        let decoded_log = SlipstreamFactory::PoolCreated::decode_log(log, false).unwrap();
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