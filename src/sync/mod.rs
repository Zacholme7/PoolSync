pub mod checkpoint;
use crate::errors::AMMError;
use crate::protocol::traits::AutomatedMarketMakerFactory;
use crate::protocol::uniswap_v2;
use crate::protocol::uniswap_v2::UniswapV2Factory;
use crate::protocol::{Factory, AMM};
use alloy_network::Network;
use alloy_provider::Provider;
use alloy_transport::Transport;
use log::info;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;
use std::sync::Arc;

// logic for managing pool caching so we dont have to refetch every execution
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Cache {
    last_block: u64,
    amms: Vec<AMM>,
}

/// Reads the cache file and returns the Cache struct if it exists.
fn read_cache<P: AsRef<Path>>(path: P) -> io::Result<Cache> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let cache: Cache = serde_json::from_str(&contents)?;
    Ok(cache)
}

/// Writes the cache data to a file.
fn write_cache<P: AsRef<Path>>(path: P, cache: &Cache) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;
    let contents = serde_json::to_string(cache)?;
    file.write_all(contents.as_bytes())?;
    Ok(())
}

/// Syncs all AMMs from the supplied factories.
pub async fn sync_amms<T, N, P>(
    factories: Vec<Factory>,
    step: u64,
    provider: Arc<P>,
) -> Result<(Vec<AMM>, u64), AMMError>
where
    T: Transport + Clone,
    N: Network,
    P: Provider<T, N> + 'static,
{
    info!("Syncing amms");


    let mut last_synced_block = 0;
    // aggreage populated amms from each thread
    let mut aggregated_amms: Vec<AMM> = vec![];

    // load from cache
    if let Ok(cache) = read_cache("pools.json") {
        last_synced_block = cache.last_block;
        aggregated_amms = cache.amms;
        return Ok((aggregated_amms, last_synced_block))
    }



    let current_block = provider.get_block_number().await?;

    let mut handles = vec![];

    // For each dex supplied, get all pair created events and get reserve values
    for factory in factories.clone() {
        let provider = provider.clone();

        // Spawn a new thread to get all pools and sync data for each dex
        handles.push(tokio::spawn(async move {
            // Get all of the amms from the factory
            let mut amms = factory
                .get_all_amms(Some(current_block), provider.clone(), step)
                .await?;

            // given each amm address, populate it with data
            populate_amms(&mut amms, current_block, provider.clone()).await?;

            // set the pool fee based off of
            if let Factory::UniswapV2Factory(factory) = factory {
                for amm in amms.iter_mut() {
                    if let AMM::UniswapV2Pool(ref mut pool) = amm {
                        pool.fee = factory.fee;
                    }
                }
            }
            // Clean empty pools
            //amms = filters::filter_empty_amms(amms);

            Ok::<_, AMMError>(amms)
        }));
    }

    for handle in handles {
        match handle.await {
            Ok(sync_result) => aggregated_amms.extend(sync_result?),
            Err(err) => println!("the error {}", err),
        }
    }

    let new_cache = Cache {
        last_block: current_block,
        amms: aggregated_amms.clone()
    };

    write_cache("pools.json", &new_cache)?;

    Ok((aggregated_amms, current_block))
}

// makes sure we are batch querying amms of the same type
pub fn amms_are_congruent(amms: &[AMM]) -> bool {
    let expected_amm = &amms[0];

    for amm in amms {
        if std::mem::discriminant(expected_amm) != std::mem::discriminant(amm) {
            return false;
        }
    }
    true
}

// Gets all pool data and sync reserves
pub async fn populate_amms<T, N, P>(
    amms: &mut [AMM],
    block_number: u64,
    provider: Arc<P>,
) -> Result<(), AMMError>
where
    T: Transport + Clone,
    N: Network,
    P: Provider<T, N>,
{
    if amms_are_congruent(amms) {
        match amms[0] {
            AMM::UniswapV2Pool(_) => {
                // Max batch size for call
                let step = 50;
                for amm_chunk in amms.chunks_mut(step) {
                    UniswapV2Factory::populate_all_v2_amms(amm_chunk, provider.clone()).await?;
                }
            }
        }
    } else {
        return Err(AMMError::IncongruentAMMs);
    }

    // For each pair in the pairs vec, get the pool data
    Ok(())
}
