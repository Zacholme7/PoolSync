use super::PoolBuilder;
use crate::onchain::TwoCurveDataSync;
use crate::{Pool, PoolSyncError, PoolType};
use alloy_dyn_abi::{DynSolType, DynSolValue};
use alloy_primitives::{Address, Bytes};
use alloy_provider::RootProvider;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CurveTwoCryptoPool {
    pub address: Address,
    pub token0: Address,
    pub token1: Address,
    pub token0_name: String,
    pub token1_name: String,
    pub token0_decimals: u8,
    pub token1_decimals: u8,
}

impl PoolBuilder for CurveTwoCryptoPool {
    // Fetch the raw pool data for the address set at end_block
    async fn get_raw_pool_data(
        end_block: u64,
        provider: Arc<RootProvider>,
        addresses: &[Address],
    ) -> Result<Bytes, PoolSyncError> {
        TwoCurveDataSync::deploy_builder(provider, Address::default(), addresses.to_vec())
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
        ])))
    }

    // Consume self and construct a top level Pool
    fn into_typed_pool(self, pool_type: PoolType) -> Pool {
        match pool_type {
            PoolType::CurveTwoCrypto => Pool::CurveTwoCrypto(self),
            _ => panic!("Pool type not supported for Curve Two structure"),
        }
    }
}

impl From<Vec<DynSolValue>> for CurveTwoCryptoPool {
    fn from(data: Vec<DynSolValue>) -> Self {
        Self {
            address: data[0].as_address().unwrap(),
            token0: data[1].as_address().unwrap(),
            token1: data[2].as_address().unwrap(),
            token0_decimals: data[3].as_uint().unwrap().0.to::<u8>(),
            token1_decimals: data[4].as_uint().unwrap().0.to::<u8>(),
            ..Default::default()
        }
    }
}
