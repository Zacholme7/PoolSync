

pub struct SushiSwapV3Fetcher;


impl PoolFetcher for SushiSwapV3Fetcher {
    fn pool_type(&self) -> PoolType {
        PoolType::SushiSwap
    }

    fn factory_address(&self, chain: Chain) -> Address {
        match chain {
            Chain::Ethereum => address!("bACEB8eC6b9355Dfc0269C18bac9d6E2Bdc29C4F"),
            Chain::Base => address!("c35DADB65012eC5796536bD9864eD8773aBc74C4"),
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