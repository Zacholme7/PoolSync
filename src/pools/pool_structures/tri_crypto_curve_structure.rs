use super::PoolBuilder;
use crate::onchain::TriCurveDataSync;
use crate::{Pool, PoolSyncError, PoolType};
use alloy_dyn_abi::{DynSolType, DynSolValue};
use alloy_primitives::{Address, Bytes};
use alloy_provider::RootProvider;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CurveTriCryptoPool {
    pub address: Address,
    pub token0: Address,
    pub token1: Address,
    pub token2: Address,
    pub token0_name: String,
    pub token1_name: String,
    pub token2_name: String,
    pub token0_decimals: u8,
    pub token1_decimals: u8,
    pub token2_decimals: u8,
}

impl PoolBuilder for CurveTriCryptoPool {
    // Fetch the raw pool data for the address set at end_block
    async fn get_raw_pool_data(
        end_block: u64,
        provider: Arc<RootProvider>,
        addresses: &[Address],
    ) -> Result<Bytes, PoolSyncError> {
        TriCurveDataSync::deploy_builder(provider, Address::default(), addresses.to_vec())
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
            DynSolType::Address,
            DynSolType::Uint(8),
            DynSolType::Uint(8),
            DynSolType::Uint(8),
        ])))
    }

    // Consume self and construct a top level Pool
    fn into_typed_pool(self, pool_type: PoolType) -> Pool {
        match pool_type {
            PoolType::CurveTriCrypto => Pool::CurveTriCrypto(self),
            _ => panic!("Pool type not supported for Curve Two structure"),
        }
    }
}

impl CurveTriCryptoPool {
    pub fn get_tokens(&self) -> Vec<Address> {
        let tokens = vec![self.token0, self.token1, self.token2];
        tokens
    }

    pub fn get_token_index(&self, token: &Address) -> Option<usize> {
        if *token == self.token0 {
            Some(0)
        } else if *token == self.token1 {
            Some(1)
        } else {
            Some(2)
        }
    }
}

impl From<Vec<DynSolValue>> for CurveTriCryptoPool {
    fn from(data: Vec<DynSolValue>) -> Self {
        let pool_address = data[0].as_address().unwrap();
        let token0 = data[1].as_address().unwrap();
        let token1 = data[2].as_address().unwrap();
        let token2 = data[3].as_address().unwrap();
        let token0_decimals = data[4].as_uint().unwrap().0.to::<u8>();
        let token1_decimals = data[5].as_uint().unwrap().0.to::<u8>();
        let token2_decimals = data[6].as_uint().unwrap().0.to::<u8>();

        Self {
            address: pool_address,
            token0,
            token1,
            token2,
            token0_decimals,
            token1_decimals,
            token2_decimals,
            ..Default::default()
        }
    }
}
