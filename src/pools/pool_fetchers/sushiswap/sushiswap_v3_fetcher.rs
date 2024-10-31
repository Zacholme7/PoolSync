use crate::pools::gen::SushiSwapV3Factory;
use crate::pools::PoolFetcher;
use crate::pools::PoolType;
use crate::Chain;
use alloy::dyn_abi::DynSolType;
use alloy::primitives::Log;
use alloy::primitives::{address, Address};
use alloy::sol_types::SolEvent;

pub struct SushiSwapV3Fetcher;

impl PoolFetcher for SushiSwapV3Fetcher {
    fn pool_type(&self) -> PoolType {
        PoolType::SushiSwapV3
    }

    fn factory_address(&self, chain: Chain) -> Address {
        match chain {
            Chain::Ethereum => address!("bACEB8eC6b9355Dfc0269C18bac9d6E2Bdc29C4F"),
            Chain::Base => address!("c35DADB65012eC5796536bD9864eD8773aBc74C4"),
            Chain::BSC => address!("126555dd55a39328F69400d6aE4F782Bd4C34ABb"),
        }
    }

    fn pair_created_signature(&self) -> &str {
        SushiSwapV3Factory::PoolCreated::SIGNATURE
    }

    fn log_to_address(&self, log: &Log) -> Address {
        let decoded_log = SushiSwapV3Factory::PoolCreated::decode_log(log, false).unwrap();
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
