use alloy::primitives::{address, Address};
use alloy_sol_types::SolEvent;
use crate::pools::gen::BaseSwapV2Factory;
use crate::pools::PoolFetcher;
use alloy::primitives::Log;
use crate::pools::PoolType;
use alloy::dyn_abi::DynSolType;
use crate::Chain;

pub struct BaseSwapV2Fetcher;

impl PoolFetcher for BaseSwapV2Fetcher {
    fn pool_type(&self) -> PoolType {
        PoolType::BaseSwapV2
    }

    fn factory_address(&self, chain: Chain) -> Address {
        match chain {
            Chain::Base => address!("FDa619b6d20975be80A10332cD39b9a4b0FAa8BB"),
            _ => panic!("BaseSwap not supported on this chain")
        }
    }

    fn pair_created_signature(&self) -> &str {
        BaseSwapV2Factory::PairCreated::SIGNATURE
    }

    fn log_to_address(&self, log: &Log) -> Address {
        let decoded_log = BaseSwapV2Factory::PairCreated::decode_log(log, false).unwrap();
        decoded_log.data.pair
    }


    fn get_pool_repr(&self) -> DynSolType {
        DynSolType::Array(Box::new(DynSolType::Tuple(vec![
            DynSolType::Address,
            DynSolType::Address,
            DynSolType::Address,
            DynSolType::Uint(8),
            DynSolType::Uint(8),
            DynSolType::Uint(112),
            DynSolType::Uint(112),
        ])))
    }
}