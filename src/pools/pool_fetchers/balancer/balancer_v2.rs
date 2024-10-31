use alloy::dyn_abi::DynSolType;
use alloy::primitives::Log;
use alloy::primitives::{address, Address};
use alloy::sol_types::SolEvent;

use crate::pools::gen::BalancerV2Factory;
use crate::pools::PoolFetcher;
use crate::pools::PoolType;
use crate::Chain;

pub struct BalancerV2Fetcher;

impl PoolFetcher for BalancerV2Fetcher {
    fn pool_type(&self) -> PoolType {
        PoolType::BalancerV2
    }

    fn factory_address(&self, chain: Chain) -> Address {
        match chain {
            Chain::Ethereum => address!("897888115Ada5773E02aA29F775430BFB5F34c51"),
            Chain::Base => address!("4C32a8a8fDa4E24139B51b456B42290f51d6A1c4"),
            Chain::BSC => todo!(),
        }
    }

    fn pair_created_signature(&self) -> &str {
        BalancerV2Factory::PoolCreated::SIGNATURE
    }

    fn log_to_address(&self, log: &Log) -> Address {
        let decoded_log = BalancerV2Factory::PoolCreated::decode_log(log, false).unwrap();
        decoded_log.data.pool
    }

    fn get_pool_repr(&self) -> DynSolType {
        DynSolType::Array(Box::new(DynSolType::Tuple(vec![
            DynSolType::Address,
            DynSolType::FixedBytes(32),
            DynSolType::Address,
            DynSolType::Address,
            DynSolType::Uint(8),
            DynSolType::Uint(8),
            DynSolType::Array(Box::new(DynSolType::Address)), // tokens (including token0, token1, and additional_tokens)
            DynSolType::Array(Box::new(DynSolType::Uint(8))), // decimals (including token0_decimals, token1_decimals, and additional_token_decimals)
            DynSolType::Array(Box::new(DynSolType::Uint(256))), // balances
            DynSolType::Array(Box::new(DynSolType::Uint(256))), // weights
            DynSolType::Uint(256),                            // swap_fee
        ])))
    }
}
