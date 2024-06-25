use alloy::providers::Provider;
use alloy::network::Ethereum;
use alloy::transports::Transport;

use tokio::sync::Semaphore;
use alloy::primitives::address;
use serde::{Serialize, Deserialize};
use alloy::rpc::types::Filter;
use alloy::primitives::Address;
use indicatif::{ProgressBar, ProgressStyle};
use anyhow::Result;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use alloy::sol_types::{sol, SolEvent};
use super::Pool;

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    contract UniswapV2Factory  {
        event PairCreated(address indexed token0, address indexed token1, address pair, uint256);
    }
);

/// A UniswapV2 AMM/pool
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UniswapV2Pool {
    pub address: Address,
    token0: Address,
    token1: Address,
}


impl UniswapV2Pool {
    pub async fn get_all_pools<P, T>(provider: Arc<P>) -> Vec<Pool> 
    where 
        P: Provider<T, Ethereum> + 'static,
        T: Transport + Clone + 'static
    {
        let mut all_pools: HashSet<Pool> = HashSet::new();
        let start_block = 10_000_000;

        // see if we have a cache

        // if we have a cache read them all into pools and set the start_block to the last syneced
        // block


        let latest_block = provider.get_block_number().await.unwrap();
        let step_size = 10_000;


        let total_steps = ((latest_block - start_block) as f64 / step_size as f64).ceil() as u64;

        let progress_bar = ProgressBar::new(total_steps);
        progress_bar.set_style(ProgressStyle::default_bar()
            .template("[{elapsed_precise}] UniswapV2: tasks completed {bar:40.cyan/blue} {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("##-"));


        let mut handles = vec![];
        let rate_limiter = Arc::new(Semaphore::new(25));

        for from_block in (start_block..latest_block).step_by(step_size as usize) {
            let to_block = (from_block + step_size - 1).min(latest_block);
            let provider = provider.clone();
            let semaphore = rate_limiter.clone();

            let pb = progress_bar.clone();
            let handle = tokio::spawn(async move {
                let result = process_block_range(provider, semaphore, from_block, to_block, 5).await;
                pb.inc(1);
                result

            });

            handles.push(handle);
            tokio::time::sleep(Duration::from_millis(40)).await;
        }

        for handle in handles {
            let pools = handle.await.unwrap().unwrap();
            all_pools.extend(pools);
        }

        let unique_pools: Vec<Pool> = all_pools.into_iter().collect();

        unique_pools
    }
}

async fn process_block_range<P, T>(
    provider: Arc<P>,
    semaphore: Arc<Semaphore>,
    from_block: u64,
    to_block: u64,
    max_retries: u32,
) -> Result<Vec<Pool>> 
where 
    P: Provider<T, Ethereum>,
    T: Transport + Clone + 'static
{
    let mut retries = 0;
    loop {
        let _permit = semaphore.acquire().await?;
        let sig = UniswapV2Factory::PairCreated::SIGNATURE;
        let address = address!("5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f");
        let filter = Filter::new()
            .address(address)
            .event(sig)
            .from_block(from_block)
            .to_block(to_block);

        match provider.get_logs(&filter).await {
            Ok(logs) => {
                //println!("Processing logs from block {} to {}", from_block, to_block);
                let pools: Vec<Pool> = logs
                    .into_iter()
                    .filter_map(|log| {
                        UniswapV2Factory::PairCreated::decode_log(&log.inner, false)
                            .ok()
                            .map(|res| {
                                Pool::UniswapV2(UniswapV2Pool {
                                    address: res.data.pair,
                                    token0: res.data.token0,
                                    token1: res.data.token1,
                                })
                            })
                    })
                    .collect();
                //println!("Found {} pools in block range {} to {}", pools.len(), from_block, to_block);
                return Ok(pools);
            }
            Err(e) => {
                if retries >= max_retries {
                    panic!("failed");
                }
                retries += 1;
                let delay = 2u64.pow(retries) * 1000;
                //println!("Error processing blocks {} to {}, retrying in {} ms. Error: {:?}", from_block, to_block, delay, e);
                tokio::time::sleep(Duration::from_millis(delay)).await;
            }
        }
    }
}
