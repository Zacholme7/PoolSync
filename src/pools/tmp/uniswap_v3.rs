//! Uniswap V3 Pool Synchronization Implementation
//!
//! This module provides the Uniswap V3-specific implementations for pool synchronization,
//! including the pool structure, factory contract interface, and pool fetcher.

use std::sync::Arc;

use crate::chain::Chain;
use crate::pools::{Pool, PoolFetcher, PoolType};
use alloy::network::Network;
use alloy::primitives::address;
use alloy::primitives::{Address, Log, U128, U256};
 
use alloy::dyn_abi::{DynSolType, DynSolValue};

use alloy::providers::Provider;
use alloy::sol_types::{sol, SolEvent};
use alloy::transports::Transport;
use async_trait::async_trait;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Uniswap V3 factory contract interface
sol! {
    #[derive(Debug)]
    #[sol(rpc)]
    contract IUniswapV3Factory {
        event PoolCreated(
            address indexed token0,
            address indexed token1,
            uint24 indexed fee,
            int24 tickSpacing,
            address pool
        );
    }
}

sol! {
    #[derive(Debug)]
    #[sol(rpc)]
    contract IUniswapV3Pool {
        function token0() external view returns (address);
        function token1() external view returns (address);
        function fee() external view returns (uint24 fee);
        function liquidity() external view returns (uint128 liquidity);
        function slot0() external view returns (uint160 sqrtPriceX96, int24 tick, uint16 observationIndex, uint16 observationCardinality, uint16 observationCardinalityNext, uint8 feeProtocol, bool unlocked);
        function tickBitmap(int16 wordPosition) external view returns (uint256);
        function tickSpacing() external view returns (int24);
        function ticks(int24 tick) external view returns (uint128 liquidityGross, int128 liquidityNet, uint256 feeGrowthOutside0X128, uint256 feeGrowthOutside1X128, int56 tickCumulativeOutside, uint160 secondsPerLiquidityOutsideX128, uint32 secondsOutside, bool initialized);
    }
}

sol! {
    #[derive(Debug)]
    #[sol(rpc)]
    contract ERC20 {
        function symbol() public view returns (string memory);
        function decimals() public view returns (uint8);
    }
}

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    UniswapV3DataSync,
    "src/abi/UniswapV3DataSync.json"
);


sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    UniswapV3TickSync,
    "src/abi/UniswapV3TickSync.json"
);





/// Represents a Uniswap V3 pool
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct UniswapV3Pool {
    /// The address of the pool contract
    pub address: Address,
    /// The address of the first token in the pair
    pub token0: Address,
    /// The address of the second token in the pair
    pub token1: Address,
    /// The name of the first token in the pair
    pub token0_name: String,
    /// The name of the second token in the pair
    pub token1_name: String,
    /// The amount of decimals in the first token
    pub token0_decimals: u8,
    /// The amount of decimals in the second token
    pub token1_decimals: u8,
    pub liquidity: U128,
    pub sqrt_price: U256,
    pub fee: u32,
    pub tick: i32,
    pub tick_spacing: i32,
    pub tick_bitmap: HashMap<i16, U256>,
    pub ticks: HashMap<i32, Info>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Info {
    pub liquidity_gross: u128,
    pub liquidity_net: i128,
    pub initialized: bool,
}

const MAX_RETRIES: u32 = 5;
const INITIAL_BACKOFF: u64 = 1000; // 1 second

/// Uniswap V3 pool fetcher implementation
pub struct UniswapV3Fetcher;

impl UniswapV3Fetcher {
    pub async fn build_pools_from_addrs<P, T, N>(
        &self,
        provider: Arc<P>,
        addresses: Vec<Address>,
    ) -> Vec<Pool>
    where
        P: Provider<T, N> + Sync + 'static,
        T: Transport + Sync + Clone,
        N: Network,
    {
        let uniswapv3_data = DynSolType::Array(Box::new(DynSolType::Tuple(vec![
            DynSolType::Address,
            DynSolType::Uint(8),
            DynSolType::Address,
            DynSolType::Uint(8),
            DynSolType::Uint(128),
            DynSolType::Uint(160),
            DynSolType::Int(24),
            DynSolType::Int(24),
            DynSolType::Uint(24),
            DynSolType::Int(128),
        ])));

        let mut retry_count = 0;
        let mut backoff = INITIAL_BACKOFF;

        loop {
            match self.attempt_build_pools(provider.clone(), &addresses, &uniswapv3_data).await {
                Ok(pools) => return pools,
                Err(e) => {
                    println!("Err {:?}", e);
                    if retry_count >= MAX_RETRIES {
                        eprintln!("Max retries reached. Error: {:?}", e);
                        return Vec::new();
                    }

                    let jitter = rand::thread_rng().gen_range(0..=100);
                    let sleep_duration = std::time::Duration::from_millis(backoff + jitter);
                    tokio::time::sleep(sleep_duration).await;

                    retry_count += 1;
                    backoff *= 2; // Exponential backoff
                }
            }
        }
    }

    pub async fn attempt_build_pools<P, T, N>(
        &self,
        provider: Arc<P>,
        addresses: &[Address],
        uniswapv3_data: &DynSolType
    ) -> Result<Vec<Pool>, Box<dyn std::error::Error>>
    where
        P: Provider<T, N> + Sync + 'static,
        T: Transport + Sync + Clone,
        N: Network,
    {
        let mut pools: Vec<Pool> = Vec::new();

        let data = UniswapV3DataSync::deploy_builder(provider.clone(), addresses.to_vec()).await?;
        let decoded_data = uniswapv3_data.abi_decode_sequence(&data)?;

        Ok(pools)
    }
}


impl From<&[DynSolValue]> for UniswapV3Pool {
    fn from(data: &[DynSolValue]) -> Self {
        Self {
            token0: data[0].as_address().unwrap(),
            token0_decimals: data[1].as_uint().unwrap().0.to::<u8>(),
            token1: data[2].as_address().unwrap(),
            token1_decimals: data[3].as_uint().unwrap().0.to::<u8>(),
            liquidity: data[4].as_uint().unwrap().0.to::<U128>(),
            sqrt_price: data[5].as_uint().unwrap().0,
            tick: data[6].as_int().unwrap().0.as_i32(),
            tick_spacing: data[7].as_int().unwrap().0.as_i32(),
            fee: data[8].as_uint().unwrap().0.to::<u32>(),
            ..Default::default()
        }
    }
}



#[async_trait]
impl PoolFetcher for UniswapV3Fetcher {
    /// Returns the pool type for Uniswap V3
    fn pool_type(&self) -> PoolType {
        PoolType::UniswapV3
    }

    /// Returns the factory address for Uniswap V3 on the given chain
    fn factory_address(&self, chain: Chain) -> Address {
        match chain {
            Chain::Ethereum => address!("1F98431c8aD98523631AE4a59f267346ea31F984"),
            Chain::Base => address!("33128a8fC17869897dcE68Ed026d694621f6FDfD"),
        }
    }

    /// Returns the event signature for pool creation in Uniswap V3
    fn pair_created_signature(&self) -> &str {
        IUniswapV3Factory::PoolCreated::SIGNATURE
    }

    /// Attempts to create a `Pool` instance from a log entry
    fn log_to_address(&self, log: &Log) -> Address {
        let decoded_log = IUniswapV3Factory::PoolCreated::decode_log(log, false)
            .ok()
            .unwrap();
        decoded_log.data.pool
    }
}
