use crate::onchain::DackieSwapV3Factory;
use crate::{Chain, PoolFetcher};
use alloy_primitives::{address, Address, Log};
use alloy_sol_types::SolEvent;

pub struct DackieSwapV3Fetcher;
impl PoolFetcher for DackieSwapV3Fetcher {
    fn factory_address(&self, chain: Chain) -> Address {
        match chain {
            Chain::Base => address!("3D237AC6D2f425D2E890Cc99198818cc1FA48870"),
            _ => panic!("DackieSwap not supported on this chain"),
        }
    }

    fn pair_created_signature(&self) -> &str {
        DackieSwapV3Factory::PoolCreated::SIGNATURE
    }

    fn log_to_address(&self, log: &Log) -> Address {
        let decoded_log = DackieSwapV3Factory::PoolCreated::decode_log(log).unwrap();
        decoded_log.data.pool
    }
}
