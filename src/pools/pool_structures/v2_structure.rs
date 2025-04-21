use super::PoolBuilder;
use crate::onchain::{AerodromeSync, DataEvents, V2DataSync};
use crate::pools::PoolType;
use crate::Pool;
use crate::PoolSyncError;
use alloy_dyn_abi::{DynSolType, DynSolValue};
use alloy_primitives::Bytes;
use alloy_primitives::{Address, U256};
use alloy_provider::RootProvider;
use alloy_rpc_types::Log;
use alloy_sol_types::SolEvent;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UniswapV2Pool {
    pub address: Address,
    pub token0: Address,
    pub token1: Address,
    pub token0_name: String,
    pub token1_name: String,
    pub token0_decimals: u8,
    pub token1_decimals: u8,
    pub token0_reserves: U256,
    pub token1_reserves: U256,
    pub stable: Option<bool>,
    pub fee: Option<U256>,
}

impl PoolBuilder for UniswapV2Pool {
    // Fetch the raw pool data for the address set at end_block
    async fn get_raw_pool_data(
        end_block: u64,
        provider: Arc<RootProvider>,
        addresses: &[Address],
    ) -> Result<Bytes, PoolSyncError> {
        V2DataSync::deploy_builder(provider, addresses.to_vec())
            .call_raw()
            .block(end_block.into())
            .await
            .map_err(|_| PoolSyncError::FailedDeployment)
    }

    fn get_pool_repr() -> DynSolType {
        DynSolType::Array(Box::new(DynSolType::Tuple(vec![
            DynSolType::Address,
            DynSolType::Address,
            DynSolType::Address,
            DynSolType::Uint(8),
            DynSolType::Uint(8),
            DynSolType::Uint(112),
            DynSolType::Uint(112),
            DynSolType::String,
            DynSolType::String,
        ])))
    }

    // Consume self and construct a top level Pool
    fn into_typed_pool(self, pool_type: PoolType) -> Pool {
        match pool_type {
            PoolType::UniswapV2 => Pool::UniswapV2(self),
            PoolType::SushiSwapV2 => Pool::SushiSwapV2(self),
            PoolType::PancakeSwapV2 => Pool::PancakeSwapV2(self),
            PoolType::BaseSwapV2 => Pool::BaseSwapV2(self),
            PoolType::AlienBaseV2 => Pool::AlienBaseV2(self),
            PoolType::SwapBasedV2 => Pool::SwapBasedV2(self),
            PoolType::DackieSwapV2 => Pool::DackieSwapV2(self),
            _ => panic!("Pool type not supported for V2 structure"),
        }
    }
}

impl From<Vec<DynSolValue>> for UniswapV2Pool {
    fn from(data: Vec<DynSolValue>) -> Self {
        Self {
            address: data[0].as_address().unwrap(),
            token0: data[1].as_address().unwrap(),
            token1: data[2].as_address().unwrap(),
            token0_decimals: data[3].as_uint().unwrap().0.to::<u8>(),
            token1_decimals: data[4].as_uint().unwrap().0.to::<u8>(),
            token0_reserves: data[5].as_uint().unwrap().0,
            token1_reserves: data[6].as_uint().unwrap().0,
            token0_name: data[7].as_str().unwrap().to_string(),
            token1_name: data[8].as_str().unwrap().to_string(),
            stable: None,
            fee: None,
        }
    }
}

// Helper to process liquidity data
pub fn process_sync_data(pool: &mut UniswapV2Pool, log: Log, pool_type: PoolType) {
    let (reserve0, reserve1) = if pool_type == PoolType::Aerodrome {
        let sync_event = AerodromeSync::Sync::decode_log(log.as_ref()).unwrap();
        (sync_event.reserve0, sync_event.reserve1)
    } else {
        let sync_event = DataEvents::Sync::decode_log(log.as_ref()).unwrap();
        (
            U256::from(sync_event.reserve0),
            U256::from(sync_event.reserve1),
        )
    };
    pool.token0_reserves = reserve0;
    pool.token1_reserves = reserve1;
}
