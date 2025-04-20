use alloy_dyn_abi::DynSolType;
use alloy_primitives::{address, Address, Log};
use alloy_sol_types::SolEvent;

use crate::onchain::AlienBaseV2Factory;
use crate::pools::{PoolFetcher, PoolType};
use crate::Chain;

pub struct AlienBaseV2Fetcher;

impl PoolFetcher for AlienBaseV2Fetcher {
    fn pool_type(&self) -> PoolType {
        PoolType::AlienBaseV2
    }

    fn factory_address(&self, chain: Chain) -> Address {
        match chain {
            Chain::Base => address!("3E84D913803b02A4a7f027165E8cA42C14C0FdE7"),
            _ => panic!("AlienBase not supported on this chain"),
        }
    }

    fn pair_created_signature(&self) -> &str {
        AlienBaseV2Factory::PairCreated::SIGNATURE
    }

    fn log_to_address(&self, log: &Log) -> Address {
        let decoded_log = AlienBaseV2Factory::PairCreated::decode_log(log).unwrap();
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
