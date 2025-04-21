use crate::onchain::UniswapV2Factory;
use crate::pools::PoolFetcher;
use crate::Chain;
use alloy_primitives::{address, Address, Log};
use alloy_sol_types::SolEvent;

pub struct UniswapV2Fetcher;

impl PoolFetcher for UniswapV2Fetcher {
    fn factory_address(&self, chain: Chain) -> Address {
        match chain {
            Chain::Ethereum => address!("5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f"),
            Chain::Base => address!("8909Dc15e40173Ff4699343b6eB8132c65e18eC6"),
        }
    }

    fn pair_created_signature(&self) -> &str {
        UniswapV2Factory::PairCreated::SIGNATURE
    }

    fn log_to_address(&self, log: &Log) -> Address {
        let decoded_log = UniswapV2Factory::PairCreated::decode_log(log).unwrap();
        decoded_log.data.pair
    }
}
