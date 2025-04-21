use crate::onchain::DackieSwapV2Factory;
use crate::pools::PoolFetcher;
use crate::Chain;
use alloy_primitives::{address, Address, Log};
use alloy_sol_types::SolEvent;

pub struct DackieSwapV2Fetcher;
impl PoolFetcher for DackieSwapV2Fetcher {
    fn factory_address(&self, chain: Chain) -> Address {
        match chain {
            Chain::Base => address!("591f122D1df761E616c13d265006fcbf4c6d6551"),
            _ => panic!("DackieSwap not supported on this chain"),
        }
    }

    fn pair_created_signature(&self) -> &str {
        DackieSwapV2Factory::PairCreated::SIGNATURE
    }

    fn log_to_address(&self, log: &Log) -> Address {
        let decoded_log = DackieSwapV2Factory::PairCreated::decode_log(log).unwrap();
        decoded_log.data.pair
    }
}
