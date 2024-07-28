pub struct PancakeSwapV3Fetcher;


impl PoolFetcher for PancakeSwapV3Fetcher {
    fn pool_type(&self) -> PoolType {
        PoolType::PancakeSwap
    }

    fn factory_address(&self, chain: Chain) -> Address {
        match chain {
            Chain::Ethereum => address!("0BFbCF9fa4f9C56B0F40a671Ad40E0805A091865"),
            Chain::Base => address!("0BFbCF9fa4f9C56B0F40a671Ad40E0805A091865"),
        }
    }
    
    fn pair_created_signature(&self) -> &str {
        UniswapV2Factory::PairCreated::SIGNATURE
    }

    fn log_to_address(&self, log: &Log) -> Address {
        let decoded_log = UniswapV2Factory::PairCreated::decode_log(log, false).unwrap();
        decoded_log.data.pair
    }
}