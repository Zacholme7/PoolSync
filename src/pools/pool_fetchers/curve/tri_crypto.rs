use alloy::dyn_abi::DynSolType;
use alloy::primitives::Log;
use alloy::primitives::{address, Address};
use alloy::sol_types::SolEvent;

use crate::pools::gen::TriCryptoFactory;
use crate::pools::PoolFetcher;
use crate::pools::PoolType;
use crate::Chain;

pub struct CurveTriCryptoFetcher;

impl PoolFetcher for CurveTriCryptoFetcher {
    fn pool_type(&self) -> PoolType {
        PoolType::CurveTriCrypto
    }

    fn factory_address(&self, chain: Chain) -> Address {
        match chain {
            Chain::Ethereum => address!("0c0e5f2fF0ff18a3be9b835635039256dC4B4963"),
            Chain::Base => address!("A5961898870943c68037F6848d2D866Ed2016bcB"),
            Chain::BSC => address!("c55837710bc500F1E3c7bb9dd1d51F7c5647E657"),
        }
    }

    fn pair_created_signature(&self) -> &str {
        TriCryptoFactory::TricryptoPoolDeployed::SIGNATURE
    }

    fn log_to_address(&self, log: &Log) -> Address {
        let decoded_log = TriCryptoFactory::TricryptoPoolDeployed::decode_log(log, false).unwrap();
        decoded_log.data.pool
    }

    fn get_pool_repr(&self) -> DynSolType {
        DynSolType::Array(Box::new(DynSolType::Tuple(vec![
            DynSolType::Address,
            DynSolType::Address,
            DynSolType::Address,
            DynSolType::Address,
            DynSolType::Uint(8),
            DynSolType::Uint(8),
            DynSolType::Uint(8),
        ])))
    }
}
