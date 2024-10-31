use crate::pools::gen::UniswapV2Factory;
use crate::pools::PoolFetcher;
use crate::pools::PoolType;
use crate::Chain;
use alloy::dyn_abi::DynSolType;
use alloy::primitives::Log;
use alloy::primitives::{address, Address};
use alloy::sol_types::SolEvent;

pub struct UniswapV2Fetcher;

impl PoolFetcher for UniswapV2Fetcher {
    fn pool_type(&self) -> PoolType {
        PoolType::UniswapV2
    }

    fn factory_address(&self, chain: Chain) -> Address {
        match chain {
            Chain::Ethereum => address!("5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f"),
            Chain::Base => address!("8909Dc15e40173Ff4699343b6eB8132c65e18eC6"),
            Chain::BSC => address!("BCfCcbde45cE874adCB698cC183deBcF17952812"),
        }
    }

    fn pair_created_signature(&self) -> &str {
        UniswapV2Factory::PairCreated::SIGNATURE
    }

    fn log_to_address(&self, log: &Log) -> Address {
        let decoded_log = UniswapV2Factory::PairCreated::decode_log(log, false).unwrap();
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
