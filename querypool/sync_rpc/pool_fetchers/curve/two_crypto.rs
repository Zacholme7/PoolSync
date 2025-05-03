use crate::onchain::TwoCryptoFactory;
use crate::{Chain, PoolFetcher};
use alloy_primitives::{address, Address, Log};
use alloy_sol_types::SolEvent;

pub struct CurveTwoCryptoFetcher;
impl PoolFetcher for CurveTwoCryptoFetcher {
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
}
