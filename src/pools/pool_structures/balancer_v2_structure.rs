use alloy::{dyn_abi::DynSolValue, primitives::Address};
use serde::{Deserialize, Serialize};
use alloy::primitives::U256;
use alloy::primitives::FixedBytes;

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
    pub weights: Option<Vec<U256>>,
    pub swap_fee: U256,
}

impl From<&[DynSolValue]> for BalancerV2Pool {
    fn from(data: &[DynSolValue]) -> Self {
        let pool_address = data[0].as_address().unwrap();
        let pool_id = FixedBytes::from_slice(data[1].as_fixed_bytes().unwrap().0);
        let token0 = data[2].as_address().unwrap();
        let token1 = data[3].as_address().unwrap();
        let token0_decimals = data[4].as_uint().unwrap().0.to::<u8>();
        let token1_decimals = data[5].as_uint().unwrap().0.to::<u8>();
        let additional_tokens: Vec<Address> = data[6].as_array().unwrap().iter().map(|v| v.as_address().unwrap()).collect();
        let additional_token_decimals: Vec<u8> = data[7].as_array().unwrap().iter().map(|v| v.as_uint().unwrap().0.to::<u8>()).collect();
        let balances: Vec<U256> = data[8].as_array().unwrap().iter().map(|v| v.as_uint().unwrap().0).collect();
        let weights: Option<Vec<U256>> = Some(data[9].as_array().unwrap().iter().map(|v| v.as_uint().unwrap().0).collect());
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