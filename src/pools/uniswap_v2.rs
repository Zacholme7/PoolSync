use super::Token;
use alloy::providers::Provider;
use tokio::sync::Semaphore;
use alloy::rpc::types::Filter;
use alloy::primitives::{Address, U256};
use std::sync::Arc;
use std::time::Duration;
use alloy::providers::RootProvider;
use alloy::transports::http::{Client, Http};

use alloy::sol_types::{sol, SolCall, SolEvent};
use async_trait::async_trait;

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    contract UniswapV2Factory  {
        event PairCreated(address indexed token0, address indexed token1, address pair, uint256);
    }
);

/// A UniswapV2 AMM/pool
pub struct UniswapV2Pool {
    address: Address,
    token0: Token,
    token1: Token,
    reserve0: U256,
    reserve1: U256,
    last_synced_block: u64,
}


impl UniswapV2Pool {
    pub async fn get_all_pools(provider: Arc<RootProvider<Http<Client>>>) {
        // get the start and end blocks
        let start_block = 10_000_000;
        let latest_block = provider.get_block_number().await.unwrap();
        let step_size = 10_000;

        // alll of our task handles
        let mut handles = vec![];

        // will control the rate limit
        let rate_limiter = Arc::new(Semaphore::new(25));

        // go through the entire range
        for from_block in (start_block..latest_block).step_by(step_size as usize) {
            let to_block = (from_block + step_size - 1).min(latest_block);
            let provider = provider.clone();
            let semaphore = rate_limiter.clone();

            // spawn the task
            let handle = tokio::spawn(async move {
                if let Err(e) = process_block_range(provider, semaphore, from_block, to_block, 5).await {
                    eprintln!("Failed to process blocks {} to {} after all retries: {:?}", from_block, to_block, e);
                }
            });

            handles.push(handle);
            tokio::time::sleep(Duration::from_millis(40)).await;
        }


        for handle in handles {
            handle.await;
        }
    }
}



async fn process_block_range(
    provider: Arc<RootProvider<Http<Client>>>,
    semaphore: Arc<Semaphore>,
    from_block: u64,
    to_block: u64,
    max_retries: u32,
) -> Result<(), eyre::Report> {
    let mut retries = 0;
    loop {
        let _permit = semaphore.acquire().await.unwrap();
        let sig = UniswapV2Factory::PairCreated::SIGNATURE;
        let filter = Filter::new()
            .event(sig)
            .from_block(from_block)
            .to_block(to_block);

        match provider.get_logs(&filter).await {
            Ok(logs) => {
                println!("Logs from block {} to {}:", from_block, to_block);
                for log in logs {
                    let res = UniswapV2Factory::PairCreated::decode_log(&log.inner, false)?;
                    println!("{} {} {:?}", from_block, to_block, res);
                }
                println!("--------------------");
                return Ok(());
            }
            Err(e) => {
                if retries >= max_retries {
                    return Err(e.into());
                }
                retries += 1;
                let delay = 2u64.pow(retries) * 1000;
                println!("Error processing blocks {} to {}, retrying in {} ms. Error: {:?}", from_block, to_block, delay, e);
                tokio::time::sleep(Duration::from_millis(delay)).await;
            }
        }
    }
}

