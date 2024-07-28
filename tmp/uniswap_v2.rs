//! Uniswap V2 Pool Synchronization Implementation
//!
//! This module provides the Uniswap V2-specific implementations for pool synchronization,
//! including the pool structure, factory contract interface, and pool fetcher.

use std::sync::Arc;

use crate::chain::Chain;
use crate::pools::{Pool, PoolFetcher, PoolType};
use alloy::dyn_abi::{DynSolType, DynSolValue};
use alloy::network::Network;
use alloy::primitives::U128;
use alloy::primitives::{address, Address, Log};
use alloy::providers::Provider;
use alloy::sol;
use alloy::sol_types::SolEvent;
use alloy::transports::Transport;
use async_trait::async_trait;
use rand::Rng;
use serde::{Deserialize, Serialize};

// Uniswap V2 factory contract interface
sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    contract UniswapV2Factory  {
        event PairCreated(address indexed token0, address indexed token1, address pair, uint256);
    }
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    UniswapV2DataSync,
    "src/abi/UniswapV2DataSync.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    contract ERC20 {
        function symbol() public view returns (string memory name);
    }
);

/// Represents a Uniswap V2 Automated Market Maker (AMM) pool
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UniswapV2Pool {
    /// The address of the pool contract
    pub address: Address,
    /// The address of the first token in the pair
    pub token0: Address,
    /// The address of the second token in the pair
    pub token1: Address,
    // The name of the first token in the pair
    pub token0_name: String,
    // The name of the second token in the pair
    pub token1_name: String,
    /// The amount of decimals in the first token
    pub token0_decimals: u8,
    /// The amount of decimals in the second token
    pub token1_decimals: u8,
    /// The reserves for the first token
    pub token0_reserves: U128,
    /// the reserves for the second pair
    pub token1_reserves: U128,
}

impl UniswapV2Pool {
    fn is_valid(&self) -> bool {
        self.address != Address::ZERO
            && self.token0 != Address::ZERO
            && self.token1 != Address::ZERO
    }
}

/// Uniswap V2 pool fetcher implementation
pub struct UniswapV2Fetcher;

const MAX_RETRIES: u32 = 5;
const INITIAL_BACKOFF: u64 = 1000; // 1 second

impl UniswapV2Fetcher {
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
        let uniswapv2_data: DynSolType = DynSolType::Array(Box::new(DynSolType::Tuple(vec![
            DynSolType::Address,
            DynSolType::Address,
            DynSolType::Address,
            DynSolType::Uint(8),
            DynSolType::Uint(8),
            DynSolType::Uint(112),
            DynSolType::Uint(112),
        ])));

        let mut retry_count = 0;
        let mut backoff = INITIAL_BACKOFF;

        loop {
            match self
                .attempt_build_pools(provider.clone(), &addresses, &uniswapv2_data)
                .await
            {
                Ok(pools) => return pools,
                Err(e) => {
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

    async fn attempt_build_pools<P, T, N>(
        &self,
        provider: Arc<P>,
        addresses: &[Address],
        uniswapv2_data: &DynSolType,
    ) -> Result<Vec<Pool>, Box<dyn std::error::Error>>
    where
        P: Provider<T, N> + Sync + 'static,
        T: Transport + Sync + Clone,
        N: Network,
    {
        let data = UniswapV2DataSync::deploy_builder(provider.clone(), addresses.to_vec()).await?;
        let decoded_data = uniswapv2_data.abi_decode_sequence(&data)?;

        let mut uniswap_v2_pools = Vec::new();

        if let Some(pool_data_arr) = decoded_data.as_array() {
            for pool_data_tuple in pool_data_arr {
                if let Some(pool_data) = pool_data_tuple.as_tuple() {
                    let pool = UniswapV2Pool::from(pool_data);
                    if pool.is_valid() {
                        uniswap_v2_pools.push(pool);
                    }
                }
            }
        }

        for pool in &mut uniswap_v2_pools {
            let token0_contract = ERC20::new(pool.token0, provider.clone());
            if let Ok(ERC20::symbolReturn { name }) = token0_contract.symbol().call().await {
                pool.token0_name = name;
            }

            let token1_contract = ERC20::new(pool.token1, provider.clone());
            if let Ok(ERC20::symbolReturn { name }) = token1_contract.symbol().call().await {
                pool.token1_name = name;
            }
        }

        Ok(uniswap_v2_pools.into_iter().map(Pool::UniswapV2).collect())
    }
}

impl From<&[DynSolValue]> for UniswapV2Pool {
    fn from(data: &[DynSolValue]) -> Self {
        Self {
            address: data[0].as_address().unwrap(),
            token0: data[1].as_address().unwrap(),
            token1: data[2].as_address().unwrap(),
            token0_decimals: data[3].as_uint().unwrap().0.to::<u8>(),
            token1_decimals: data[4].as_uint().unwrap().0.to::<u8>(),
            token0_reserves: data[5].as_uint().unwrap().0.to::<U128>(),
            token1_reserves: data[6].as_uint().unwrap().0.to::<U128>(),
            ..Default::default()
        }
    }
}

#[async_trait]
impl PoolFetcher for UniswapV2Fetcher {
    /// Returns the pool type for Uniswap V2
    fn pool_type(&self) -> PoolType {
        PoolType::UniswapV2
    }

    /// Returns the factory address for Uniswap V2 on the given chain
    fn factory_address(&self, chain: Chain) -> Address {
        match chain {
            Chain::Ethereum => address!("5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f"),
            Chain::Base => address!("8909Dc15e40173Ff4699343b6eB8132c65e18eC6"),
        }
    }

    /// Returns the event signature for pair creation in Uniswap V2
    fn pair_created_signature(&self) -> &str {
        UniswapV2Factory::PairCreated::SIGNATURE
    }

    /// Attempts to create a `Pool` instance from a log entry
    fn log_to_address(&self, log: &Log) -> Address {
        let decoded_log = UniswapV2Factory::PairCreated::decode_log(log, false).unwrap();
        decoded_log.data.pair
    }
}
