

pub struct SushiSwapV2Fetcher;


impl PoolFetcher for SushiSwapV2Fetcher {
    fn pool_type(&self) -> PoolType {
        PoolType::SushiSwap
    }

    fn factory_address(&self, chain: Chain) -> Address {
        match chain {
            Chain::Ethereum => address!("0xC0AEe478e3658e2610c5F7A4A2E1777cE9e4f2Ac"),
            Chain::Base => address!("0x71524B4f93c58fcbF659783284E38825f0622859"),
        }
    }
    
    fn pair_created_signature(&self) -> &str {
        SushiSwapFactory::PairCreated::SIGNATURE
    }

    fn log_to_address(&self, log: &Log) -> Address {
        let decoded_log = SushiSwapFactory::PairCreated::decode_log(log, false).unwrap();
        decoded_log.data.pair
    }
}