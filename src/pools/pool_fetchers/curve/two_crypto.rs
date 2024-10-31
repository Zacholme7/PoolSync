use crate::pools::gen::TwoCryptoFactory;
use crate::pools::PoolFetcher;
use crate::pools::PoolType;
use crate::Chain;
use alloy::dyn_abi::DynSolType;
use alloy::primitives::Log;
use alloy::primitives::{address, Address};
use alloy::sol_types::SolEvent;

pub struct CurveTwoCryptoFetcher;

impl PoolFetcher for CurveTwoCryptoFetcher {
    fn pool_type(&self) -> PoolType {
        PoolType::CurveTwoCrypto
    }

    fn factory_address(&self, chain: Chain) -> Address {
        match chain {
            Chain::Ethereum => address!("98EE851a00abeE0d95D08cF4CA2BdCE32aeaAF7F"),
            Chain::Base => address!("c9Fe0C63Af9A39402e8a5514f9c43Af0322b665F"),
            Chain::BSC => address!("98EE851a00abeE0d95D08cF4CA2BdCE32aeaAF7F"),
        }
    }

    fn pair_created_signature(&self) -> &str {
        TwoCryptoFactory::TwocryptoPoolDeployed::SIGNATURE
    }

    fn log_to_address(&self, log: &Log) -> Address {
        let decoded_log = TwoCryptoFactory::TwocryptoPoolDeployed::decode_log(log, false).unwrap();
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
