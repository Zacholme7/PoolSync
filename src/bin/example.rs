use std::time::Duration;

use alloy::network::EthereumWallet;
use alloy::node_bindings::Anvil;
use std::sync::Arc;
use alloy::primitives::address;
use tokio::sync::Semaphore;
use alloy::primitives::{U128, U256};
use alloy::providers::{Provider, RootProvider};
use alloy::providers::ProviderBuilder;
use alloy::sol_types::{sol, SolCall};
use alloy_sol_types::SolEvent;
use alloy::rpc::types::{BlockNumberOrTag, Filter};
use pool_sync::{PoolSync, PoolType};
use alloy::primitives::Log;
use eyre::Result;
use alloy::transports::http::{Http, Client};


sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    contract UniswapV2Factory  {
        event PairCreated(address indexed token0, address indexed token1, address pair, uint256);
    }
);

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

#[tokio::main]
async fn main() -> Result<()> {
    let url = "https://rpc.merkle.io/1/sk_mbs_f3cc7544d55b8976b06f881c6910921c";
    let provider = Arc::new(ProviderBuilder::new().on_http(url.parse()?));
    let sig = UniswapV2Factory::PairCreated::SIGNATURE;
    let start_block = 10_000_000;
    let current_block = provider.get_block_number().await?;
    let step_size = 10_000;
    let semaphore = Arc::new(Semaphore::new(25)); // Allow 25 concurrent requests
    let mut handles = vec![];

    for from_block in (start_block..current_block).step_by(step_size as usize) {
        let to_block = (from_block + step_size - 1).min(current_block);
        let provider = provider.clone();
        let semaphore = semaphore.clone();

        let handle = tokio::spawn(async move {
            if let Err(e) = process_block_range(provider, semaphore, from_block, to_block, 5).await {
                eprintln!("Failed to process blocks {} to {} after all retries: {:?}", from_block, to_block, e);
            }
        });

        handles.push(handle);
        tokio::time::sleep(Duration::from_millis(40)).await;
    }

    for handle in handles {
        handle.await?;
    }

    Ok(())
}