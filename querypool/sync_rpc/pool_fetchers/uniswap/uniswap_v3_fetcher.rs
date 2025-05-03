use crate::onchain::UniswapV3Factory;
use crate::{Chain, PoolFetcher};
use alloy_primitives::{address, Address, Log};
use alloy_sol_types::SolEvent;

pub struct UniswapV3Fetcher;
impl PoolFetcher for UniswapV3Fetcher {
    fn factory_address(&self, chain: Chain) -> Address {
        match chain {
            Chain::Ethereum => address!("1F98431c8aD98523631AE4a59f267346ea31F984"),
            Chain::Base => address!("33128a8fC17869897dcE68Ed026d694621f6FDfD"),
        }
    }

    fn pair_created_signature(&self) -> &str {
        UniswapV3Factory::PoolCreated::SIGNATURE
    }

    fn log_to_address(&self, log: &Log) -> Address {
        let decoded_log = UniswapV3Factory::PoolCreated::decode_log(log).unwrap();
        decoded_log.data.pool
    }
}
