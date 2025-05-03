use crate::onchain::MaverickV1Factory;
use crate::{Chain, PoolFetcher};
use alloy_primitives::{address, Address, Log};
use alloy_sol_types::SolEvent;

pub struct MaverickV1Fetcher;
impl PoolFetcher for MaverickV1Fetcher {
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
}
