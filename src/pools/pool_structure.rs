use alloy::primitives::{Address, U128, U256};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

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
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UniswapV3Pool {
    pub address: Address,
    pub token0: Address,
    pub token1: Address,
    pub token0_name: String,
    pub token1_name: String,
    pub token0_decimals: u8,
    pub token1_decimals: u8,
    pub liquidity: U128,
    pub sqrt_price: U256,
    pub fee: u32,
    pub tick: i32,
    pub tick_spacing: i32,
    pub tick_bitmap: HashMap<i16, U256>,
    pub ticks: HashMap<i32, TickInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TickInfo {
    pub liquidity_gross: u128,
    pub liquidity_net: i128,
    pub initialized: bool,
}

impl UniswapV2Pool {
    pub fn is_valid(&self) -> bool {
        self.address != Address::ZERO
            && self.token0 != Address::ZERO
            && self.token1 != Address::ZERO
    }
}

impl UniswapV3Pool {
    pub fn is_valid(&self) -> bool {
        self.address != Address::ZERO
            && self.token0 != Address::ZERO
            && self.token1 != Address::ZERO
    }
}