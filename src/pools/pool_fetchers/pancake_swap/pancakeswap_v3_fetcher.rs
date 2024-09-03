use alloy::primitives::{address, Address};
use alloy::sol_types::SolEvent;
use alloy::primitives::Log;
use alloy::dyn_abi::DynSolType;
use crate::pools::gen::PancakeSwapV3Factory;
use crate::pools::PoolFetcher;
use crate::pools::PoolType;
use crate::Chain;

pub struct PancakeSwapV3Fetcher;

impl PoolFetcher for PancakeSwapV3Fetcher {
    fn pool_type(&self) -> PoolType {
        PoolType::PancakeSwapV3
    }

    fn factory_address(&self, chain: Chain) -> Address {
        match chain {
            Chain::Ethereum => address!("0BFbCF9fa4f9C56B0F40a671Ad40E0805A091865"),
            Chain::Base => address!("0BFbCF9fa4f9C56B0F40a671Ad40E0805A091865"),
        }
    }
    
    fn pair_created_signature(&self) -> &str {
        PancakeSwapV3Factory::PoolCreated::SIGNATURE
    }

    fn log_to_address(&self, log: &Log) -> Address {
        let decoded_log = PancakeSwapV3Factory::PoolCreated::decode_log(log, false).unwrap();
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