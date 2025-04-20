use crate::onchain::TwoCryptoFactory;
use crate::pools::PoolFetcher;
use crate::pools::PoolType;
use crate::Chain;
use alloy_dyn_abi::DynSolType;
use alloy_primitives::Log;
use alloy_primitives::{address, Address};
use alloy_sol_types::SolEvent;

pub struct CurveTwoCryptoFetcher;

impl PoolFetcher for CurveTwoCryptoFetcher {
    fn pool_type(&self) -> PoolType {
        PoolType::CurveTwoCrypto
    }

    fn factory_address(&self, chain: Chain) -> Address {
        match chain {
            Chain::Ethereum => address!("98EE851a00abeE0d95D08cF4CA2BdCE32aeaAF7F"),
            Chain::Base => address!("c9Fe0C63Af9A39402e8a5514f9c43Af0322b665F"),
        }
    }

    fn pair_created_signature(&self) -> &str {
        TwoCryptoFactory::TwocryptoPoolDeployed::SIGNATURE
    }

    fn log_to_address(&self, log: &Log) -> Address {
        let decoded_log = TwoCryptoFactory::TwocryptoPoolDeployed::decode_log(log).unwrap();
        decoded_log.data.pool
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
