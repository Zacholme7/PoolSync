use alloy::primitives::{address, Address};
use alloy_sol_types::SolEvent;
use crate::pools::gen::DackieSwapV2Factory;
use crate::pools::PoolFetcher;
use alloy::primitives::Log;
use alloy::dyn_abi::DynSolType;
use crate::pools::PoolType;
use crate::Chain;

pub struct DackieSwapV2Fetcher;

impl PoolFetcher for DackieSwapV2Fetcher {
    fn pool_type(&self) -> PoolType {
        PoolType::DackieSwapV2
    }

    fn factory_address(&self, chain: Chain) -> Address {
        match chain {
            Chain::Base => address!("591f122D1df761E616c13d265006fcbf4c6d6551"),
            _ => panic!("DackieSwap not supported on this chain")
        }
    }

    fn pair_created_signature(&self) -> &str {
        DackieSwapV2Factory::PairCreated::SIGNATURE
    }

    fn log_to_address(&self, log: &Log) -> Address {
        let decoded_log = DackieSwapV2Factory::PairCreated::decode_log(log, false).unwrap();
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