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

use super::sushiswap_v2::UniswapV2DataSync;


sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    AerodomeFactory,
    "src/pools/abis/AeordomeFactory.json"
);

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AerodomePool {
    pub address: Address,
    pub token0: Address,
    pub token1: Address,
    pub token0_name: String,
    pub token1_name: String,
    pub token0_decimals: u8,
    pub token1_decimals: u8,
    pub token0_reserves: U128,
    pub token1_reserves: U128
}

impl AerodomePool {
    fn is_valid(&self) -> bool {
        self.address != Address::ZERO
            && self.token0 != Address::ZERO
            && self.token1 != Address::ZERO
    }
}

/// Aerodome pool fetcher implementation
pub struct AerodomeFetcher;

const MAX_RETRIES: u32 = 5;
const INITIAL_BACKOFF: u64 = 1000; // 1 second

impl AerodomeFetcher {
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
        let aerodome_data: DynSolType = DynSolType::Array(Box::new(DynSolType::Tuple(vec![
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
            match self.attempt_build_pools(provider.clone(), &addresses, &aerodome_data).await {
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
        aerodome_data: &DynSolType
    ) -> Result<Vec<Pool>, Box<dyn std::error::Error>>
    where
        P: Provider<T, N> + Sync + 'static,
        T: Transport + Sync + Clone,
        N: Network,
    {
        let data = UniswapV2DataSync::deploy_builder(provider.clone(), addresses.to_vec()).await?;
        let decoded_data = aerodome_data.abi_decode_sequence(&data)?;        
        let mut aerodome_pools = Vec::new();
        if let Some(pool_data_arr) = decoded_data.as_array() {
            for pool_data_tuple in pool_data_arr {
                if let Some(pool_data) = pool_data_tuple.as_tuple() {
                    let pool = AerodomePool::from(pool_data);
                    if pool.is_valid() {
                        aerodome_pools.push(pool);
                    }
                }
            }
        }
        for pool in &mut aerodome_pools {
            
        }
        Ok(aerodome_pools.into_iter().map(Pool::Aerodome).collect())
    }


}

impl From<&[DynSolValue]> for AerodomePool {
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
impl PoolFetcher for AerodomeFetcher {
    /// Returns the pool type for Aerodome
    fn pool_type(&self) -> PoolType {
        PoolType::Aerodome
    }

    /// Returns the factory address for Aerodome on the given chain
    fn factory_address(&self, chain: Chain) -> Address {
        match chain {
            Chain::Base => address!("420DD381b31aEf6683db6B902084cB0FFECe40Da"),
            _ => panic!("Aerodome not supported on this chain")
        }
    }

    /// Returns the event signature for pool creation in Aerodome
    fn pair_created_signature(&self) -> &str {
        AerodomeFactory::PoolCreated::SIGNATURE
    }

    /// Attempts to create a `Pool` instance from a log entry
    fn log_to_address(&self, log: &Log) -> Address {
        let decoded_log = AerodomeFactory::PoolCreated::decode_log(log, false).unwrap();
        decoded_log.data.pool
    }

}