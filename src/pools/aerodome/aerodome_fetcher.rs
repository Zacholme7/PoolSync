

pub struct AerodomeFetcher;


impl PoolFetcher for AerodomeFetcher {
    fn pool_type(&self) -> PoolType {
        PoolType::Aerodome
    }

    fn factory_address(&self, chain: Chain) -> Address {
        match chain {
            Chain::Base => address!("	420DD381b31aEf6683db6B902084cB0FFECe40Da"),
            _ => panic!("Aerodome not supported on this chain")
        }
    }

    fn pair_created_signature(&self) -> &str {
        AerodomeFactory::PoolCreated::SIGNATURE
    }

    fn log_to_address(&self, log: &Log) -> Address {
        let decoded_log = AerodomeFactory::PoolCreated::decode_log(log, false).unwrap();
        decoded_log.data.pool
    }
}