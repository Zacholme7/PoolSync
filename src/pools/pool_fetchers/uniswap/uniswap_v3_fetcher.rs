
use crate::pools::gen::UniswapV3Factory;
use alloy::primitives::{address, Address};
use alloy_sol_types::SolEvent;
use crate::pools::PoolFetcher;
use alloy::primitives::Log;
use crate::pools::PoolType;
use crate::Chain;
use alloy::dyn_abi::DynSolType;
pub struct UniswapV3Fetcher;

impl PoolFetcher for UniswapV3Fetcher {
    fn pool_type(&self) -> PoolType {
        PoolType::UniswapV3
    }

    fn factory_address(&self, chain: Chain) -> Address {
        match chain {
            Chain::Ethereum => address!("1F98431c8aD98523631AE4a59f267346ea31F984"),
            Chain::Base => address!("33128a8fC17869897dcE68Ed026d694621f6FDfD"),
        }
    }

    fn pair_created_signature(&self) -> &str {
        UniswapV3Factory::PoolCreated::SIGNATURE
    }

    fn log_to_address(&self, log: &Log) -> Address {
        let decoded_log = UniswapV3Factory::PoolCreated::decode_log(log, false).unwrap();
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