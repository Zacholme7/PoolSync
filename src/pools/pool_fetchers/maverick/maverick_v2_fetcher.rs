use crate::pools::PoolFetcher;
use crate::onchain::PoolCreated;
use crate::pools::PoolType;
use crate::Chain;
use alloy_dyn_abi::DynSolType;
use alloy_primitives::{address, Address, Log};
use alloy_sol_types::SolEvent;

pub struct MaverickV2Fetcher;

impl PoolFetcher for MaverickV2Fetcher {
    fn pool_type(&self) -> PoolType {
        PoolType::MaverickV2
    }

    fn factory_address(&self, chain: Chain) -> Address {
        match chain {
            Chain::Ethereum => address!("0A7e848Aca42d879EF06507Fca0E7b33A0a63c1e"),
            Chain::Base => address!("0A7e848Aca42d879EF06507Fca0E7b33A0a63c1e"),
        }
    }

    fn pair_created_signature(&self) -> &str {
        PoolCreated::SIGNATURE
    }

    fn log_to_address(&self, log: &Log) -> Address {
        let decoded_log = PoolCreated::decode_log(log).unwrap();
        decoded_log.data.poolAddress
    }

    fn get_pool_repr(&self) -> DynSolType {
        DynSolType::Array(Box::new(DynSolType::Tuple(vec![
            DynSolType::Address,
            DynSolType::Address,
            DynSolType::Address,
            DynSolType::Uint(8),
            DynSolType::Uint(8),
        ])))
    }
}
