use super::PoolBuilder;
use crate::onchain::{BalancerV2DataSync, Vault};
use crate::{Pool, PoolSyncError, PoolType};
use alloy_dyn_abi::{DynSolType, DynSolValue};
use alloy_primitives::{Address, Bytes, FixedBytes, U256};
use alloy_provider::RootProvider;
use alloy_rpc_types::Log;
use alloy_sol_types::SolEvent;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BalancerV2Pool {
    pub address: Address,
    pub pool_id: FixedBytes<32>,
    pub token0: Address,
    pub token1: Address,
    pub token0_name: String,
    pub token1_name: String,
    pub token0_decimals: u8,
    pub token1_decimals: u8,
    pub additional_tokens: Vec<Address>,
    pub additional_token_names: Vec<String>,
    pub additional_token_decimals: Vec<u8>,
    pub balances: Vec<U256>,
    pub weights: Vec<U256>,
    pub swap_fee: U256,
}

impl PoolBuilder for BalancerV2Pool {
    // Fetch the raw pool data for the address set at end_block
    async fn get_raw_pool_data(
        end_block: u64,
        provider: Arc<RootProvider>,
        addresses: &[Address],
    ) -> Result<Bytes, PoolSyncError> {
        BalancerV2DataSync::deploy_builder(provider, addresses.to_vec())
            .call_raw()
            .block(end_block.into())
            .await
            .map_err(|_| PoolSyncError::FailedDeployment)
    }

    fn get_pool_repr() -> DynSolType {
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

    // Consume self and construct a top level Pool
    fn into_typed_pool(self, pool_type: PoolType) -> Pool {
        match pool_type {
            PoolType::BalancerV2 => Pool::BalancerV2(self),
            _ => panic!("Pool type not supported for BalancerV2 structure"),
        }
    }
}

impl From<Vec<DynSolValue>> for BalancerV2Pool {
    fn from(data: Vec<DynSolValue>) -> Self {
        let pool_address = data[0].as_address().unwrap();
        let pool_id = FixedBytes::from_slice(data[1].as_fixed_bytes().unwrap().0);
        let token0 = data[2].as_address().unwrap();
        let token1 = data[3].as_address().unwrap();
        let token0_decimals = data[4].as_uint().unwrap().0.to::<u8>();
        let token1_decimals = data[5].as_uint().unwrap().0.to::<u8>();
        let additional_tokens: Vec<Address> = data[6]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_address().unwrap())
            .collect();
        let additional_token_decimals: Vec<u8> = data[7]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_uint().unwrap().0.to::<u8>())
            .collect();
        let balances: Vec<U256> = data[8]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_uint().unwrap().0)
            .collect();
        let weights: Vec<U256> = data[9]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_uint().unwrap().0)
            .collect();
        let swap_fee = data[10].as_uint().unwrap().0;

        Self {
            address: pool_address,
            pool_id,
            token0,
            token1,
            token0_name: String::new(), // To be populated later
            token1_name: String::new(), // To be populated later
            token0_decimals,
            token1_decimals,
            additional_tokens,
            additional_token_names: Vec::new(), // To be populated later
            additional_token_decimals,
            balances,
            weights,
            swap_fee,
        }
    }
}

impl BalancerV2Pool {
    pub fn get_tokens(&self) -> Vec<Address> {
        let mut tokens = vec![self.token0, self.token1];
        tokens.extend(self.additional_tokens.iter());
        tokens
    }

    pub fn get_token_index(&self, token: &Address) -> Option<usize> {
        if *token == self.token0 {
            Some(0)
        } else if *token == self.token1 {
            Some(1)
        } else {
            self.additional_tokens
                .iter()
                .position(|&t| t == *token)
                .map(|pos| pos + 2)
        }
    }

    pub fn get_balance(&self, token: &Address) -> U256 {
        let index = self.get_token_index(token);
        if let Some(index) = index {
            self.balances[index]
        } else {
            U256::ZERO
        }
    }

    pub fn process_balance_data(&mut self, log: Log) {
        let event = Vault::Swap::decode_log(log.as_ref()).unwrap();

        let log_token_in_idx = self.get_token_index(&event.tokenIn).unwrap();
        let log_token_out_idx = self.get_token_index(&event.tokenOut).unwrap();

        self.balances[log_token_in_idx] =
            self.balances[log_token_in_idx].saturating_add(event.amountIn);
        self.balances[log_token_out_idx] =
            self.balances[log_token_out_idx].saturating_sub(event.amountOut);
    }
}
