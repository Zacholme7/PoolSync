use alloy::primitives::{Address, U128, U256};
use alloy_sol_types::SolEvent;
use serde::{Deserialize, Serialize};
use alloy::rpc::types::Log;
use crate::pools::PoolType;
use crate::rpc::{DataEvents, AerodromeSync};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UniswapV2Pool {
    pub address: Address,
    pub token0: Address,
    pub token1: Address,
    pub token0_name: String,
    pub token1_name: String,
    pub token0_decimals: u8,
    pub token1_decimals: u8,
    pub token0_reserves: U128,
    pub token1_reserves: U128,
    pub stable: Option<bool>,
    pub fee: Option<U256>,
}

pub fn process_sync_data(pool: &mut UniswapV2Pool, log: Log, pool_type: PoolType) {
    if pool_type == PoolType::Aerodrome {
        let sync_event = AerodromeSync::Sync::decode_log(log.as_ref(), true).unwrap();
        pool.token0_reserves = U128::from(sync_event.reserve0);
        pool.token1_reserves = U128::from(sync_event.reserve1);
        return;
    } else {
        let sync_event = DataEvents::Sync::decode_log(log.as_ref(), true).unwrap();
        pool.token0_reserves = U128::from(sync_event.reserve0);
        pool.token1_reserves = U128::from(sync_event.reserve1);
    }
}