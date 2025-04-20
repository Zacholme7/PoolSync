use crate::onchain::MaverickV1Factory;
use crate::pools::PoolFetcher;
use crate::pools::PoolType;
use crate::Chain;
use alloy_dyn_abi::DynSolType;
use alloy_primitives::Log;
use alloy_primitives::{address, Address};
use alloy_sol_types::SolEvent;

pub struct MaverickV1Fetcher;

impl PoolFetcher for MaverickV1Fetcher {
    fn pool_type(&self) -> PoolType {
        PoolType::MaverickV1
    }

    fn factory_address(&self, chain: Chain) -> Address {
        match chain {
            Chain::Ethereum => address!("Eb6625D65a0553c9dBc64449e56abFe519bd9c9B"),
            Chain::Base => address!("B2855783a346735e4AAe0c1eb894DEf861Fa9b45"),
        }
    }

    fn pair_created_signature(&self) -> &str {
        MaverickV1Factory::PoolCreated::SIGNATURE
    }

    fn log_to_address(&self, log: &Log) -> Address {
        let decoded_log = MaverickV1Factory::PoolCreated::decode_log(log).unwrap();
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
