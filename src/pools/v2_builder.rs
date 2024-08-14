use crate::pools::{Pool, PoolType};
use alloy::network::Network;
use alloy::primitives::{Address, address};
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

use super::pool_structure::UniswapV2Pool;
use crate::pools::gen::AerodromeV2Factory;
use crate::pools::gen::ERC20;
use crate::rpc::DataEvents;

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    V2DataSync,
    "src/abi/V2DataSync.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    contract AerodromePool {
        function stable() external view returns (bool);
    }
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
    let v2_data: DynSolType = DynSolType::Array(Box::new(DynSolType::Tuple(vec![
        DynSolType::Address,
        DynSolType::Address,
        DynSolType::Address,
        DynSolType::Uint(8),
        DynSolType::Uint(8),
        DynSolType::Uint(112),
        DynSolType::Uint(112),
    ])));

    let data = V2DataSync::deploy_builder(provider.clone(), addresses.to_vec()).await?;
    let decoded_data = v2_data.abi_decode_sequence(&data)?;

    let mut pools = Vec::new();

    if let Some(pool_data_arr) = decoded_data.as_array() {
        for pool_data_tuple in pool_data_arr {
            if let Some(pool_data) = pool_data_tuple.as_tuple() {
                let pool = UniswapV2Pool::from(pool_data);
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

    // if aerodrome, we need to fetch if the pool is stable or not
    if pool_type == PoolType::Aerodrome {
        let factory_address = address!("420DD381b31aEf6683db6B902084cB0FFECe40Da");
        let factory_contract = AerodromeV2Factory::new(factory_address, provider.clone());

        for pool in &mut pools {
            let token_contract = AerodromePool::new(pool.address, provider.clone());
            // get the stable vaalue
            let AerodromePool::stableReturn { _0: stable } = token_contract.stable().call().await.unwrap();
            pool.stable = Some(stable);

            let AerodromeV2Factory::getFeeReturn { _0: fee } = factory_contract.getFee(pool.address, stable).call().await.unwrap();
            pool.fee = Some(fee);
        }
    }

    let pools = pools
        .into_iter()
        .map(|pool| Pool::new_v2(pool_type, pool))
        .collect();

    Ok(pools)
}

pub fn process_sync_data(pool: &mut UniswapV2Pool, log: Log) {
    let sync_event = DataEvents::Sync::decode_log(log.as_ref(), true).unwrap();
    pool.token0_reserves = U128::from(sync_event.reserve0);
    pool.token1_reserves = U128::from(sync_event.reserve1);
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
