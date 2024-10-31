use alloy::dyn_abi::DynSolValue;
use alloy::primitives::Address;
use serde::{Deserialize, Serialize};

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

impl From<&[DynSolValue]> for CurveTriCryptoPool {
    fn from(data: &[DynSolValue]) -> Self {
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
