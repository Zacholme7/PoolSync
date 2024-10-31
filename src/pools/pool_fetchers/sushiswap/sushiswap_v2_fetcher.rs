use crate::pools::gen::SushiSwapV2Factory;
use crate::pools::PoolFetcher;
use crate::pools::PoolType;
use crate::Chain;
use alloy::dyn_abi::DynSolType;
use alloy::primitives::Log;
use alloy::primitives::{address, Address};
use alloy::sol_types::SolEvent;
pub struct SushiSwapV2Fetcher;

impl PoolFetcher for SushiSwapV2Fetcher {
    fn pool_type(&self) -> PoolType {
        PoolType::SushiSwapV2
    }

    fn factory_address(&self, chain: Chain) -> Address {
        match chain {
            Chain::Ethereum => address!("C0AEe478e3658e2610c5F7A4A2E1777cE9e4f2Ac"),
            Chain::Base => address!("71524B4f93c58fcbF659783284E38825f0622859"),
            Chain::BSC => address!("cA143Ce32Fe78f1f7019d7d551a6402fC5350c73"),
        }
    }

    fn pair_created_signature(&self) -> &str {
        SushiSwapV2Factory::PairCreated::SIGNATURE
    }

    fn log_to_address(&self, log: &Log) -> Address {
        let decoded_log = SushiSwapV2Factory::PairCreated::decode_log(log, false).unwrap();
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
