use crate::onchain::TriCryptoFactory;
use crate::{Chain, PoolFetcher};
use alloy_primitives::{address, Address, Log};
use alloy_sol_types::SolEvent;

pub struct CurveTriCryptoFetcher;
impl PoolFetcher for CurveTriCryptoFetcher {
    fn factory_address(&self, chain: Chain) -> Address {
        match chain {
            Chain::Ethereum => address!("0c0e5f2fF0ff18a3be9b835635039256dC4B4963"),
            Chain::Base => address!("A5961898870943c68037F6848d2D866Ed2016bcB"),
        }
    }

    fn pair_created_signature(&self) -> &str {
        TriCryptoFactory::TricryptoPoolDeployed::SIGNATURE
    }

    fn log_to_address(&self, log: &Log) -> Address {
        let decoded_log = TriCryptoFactory::TricryptoPoolDeployed::decode_log(log).unwrap();
        decoded_log.data.pool
    }
}
