use crate::pools::{Pool, PoolType};
use alloy::network::Network;
use alloy::primitives::{address, Address};
use alloy::providers::Provider;
use alloy::sol;
use alloy::transports::Transport;
use alloy::{
    dyn_abi::{DynSolType, DynSolValue},
    primitives::U128,
    rpc::types::Log,
};
use alloy_sol_types::SolEvent;
use rand::Rng;
use std::sync::Arc;

use std::time::Duration;

use super::pool_structure::{CurvePool, SimulatedPool};
use crate::pools::gen::ERC20;

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    SimulatedDataSync,
    "src/abi/SimulatedDataSync.json"
);

pub const INITIAL_BACKOFF: u64 = 1000; // 1 second
pub const MAX_RETRIES: u32 = 5;

pub async fn build_pools<P, T, N>(
    provider: Arc<P>,
    addresses: Vec<Address>,
    pool_type: PoolType,
) -> Vec<Pool>
where
    P: Provider<T, N> + Sync + 'static,
    T: Transport + Sync + Clone,
    N: Network,
{
    let mut retry_count = 0;
    let mut backoff = INITIAL_BACKOFF;

    loop {
        match attempt_build_pools(provider.clone(), &addresses, pool_type).await {
            Ok(pools) => return pools,
            Err(e) => {
                if retry_count >= MAX_RETRIES {
                    eprintln!("Max retries reached. Error: {:?}", e);
                }

                let jitter = rand::thread_rng().gen_range(0..=100);
                let sleep_duration = Duration::from_millis(backoff + jitter);
                tokio::time::sleep(sleep_duration).await;

                retry_count += 1;
                backoff *= 2; // Exponential backoff
            }
        }
    }
}

async fn attempt_build_pools<P, T, N>(
    provider: Arc<P>,
    addresses: &Vec<Address>,
    pool_type: PoolType,
) -> Result<Vec<Pool>, Box<dyn std::error::Error>>
where
    P: Provider<T, N> + Sync + 'static,
    T: Transport + Sync + Clone,
    N: Network,
{
    let simulated_data: DynSolType = DynSolType::Array(Box::new(DynSolType::Tuple(vec![
        DynSolType::Address,
        DynSolType::Address,
        DynSolType::Address,
        DynSolType::Uint(8),
        DynSolType::Uint(8),
    ])));

    let data = SimulatedDataSync::deploy_builder(provider.clone(), addresses.to_vec()).await?;
    let decoded_data = simulated_data.abi_decode_sequence(&data)?;

    let mut pools = Vec::new();

    if let Some(pool_data_arr) = decoded_data.as_array() {
        for pool_data_tuple in pool_data_arr {
            if let Some(pool_data) = pool_data_tuple.as_tuple() {
                let pool = SimulatedPool::from(pool_data);
                if pool.is_valid() {
                    pools.push(pool);
                }
            }
        }
    }

    // Fetch token names (you might want to batch this for efficiency)
    for pool in &mut pools {
        let token0_contract = ERC20::new(pool.token0, provider.clone());
        if let Ok(ERC20::symbolReturn { _0: name }) = token0_contract.symbol().call().await {
            pool.token0_name = name;
        }

        let token1_contract = ERC20::new(pool.token1, provider.clone());
        if let Ok(ERC20::symbolReturn { _0: name }) = token1_contract.symbol().call().await {
            pool.token1_name = name;
        }
    }

    let pools = pools
        .into_iter()
        .map(|pool| Pool::new_simulated(pool_type, pool))
        .collect();

    Ok(pools)
}

impl From<&[DynSolValue]> for SimulatedPool {
    fn from(data: &[DynSolValue]) -> Self {
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
