//! Pool Synchronization Cache Implementation
//!
//! This module provides functionality for caching pool synchronization data,
//! including structures and functions for reading from and writing to cache files.
//! 
use crate::chain::Chain;
use crate::pools::{Pool, PoolType};
use alloy::primitives::{Address, U256};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter};
use std::path::Path;
use anyhow::{Result, Context};

#[derive(Serialize, Deserialize, Debug)]
pub struct PoolCache {
    pub last_synced_block: u64,
    pub pool_type: PoolType,
    pub pools: Vec<Pool>,
}

pub fn read_cache_file(pool_type: &PoolType, chain: Chain) -> Result<PoolCache> {
    let pool_cache_file = format!("cache/{}_{}_cache.json", chain, pool_type);
    if Path::new(&pool_cache_file).exists() {
        let file = File::open(&pool_cache_file)
            .with_context(|| format!("Failed to open cache file: {}", pool_cache_file))?;
        let reader = BufReader::new(file);
        let pool_cache: PoolCache = serde_json::from_reader(reader)
            .with_context(|| format!("Failed to deserialize cache from file: {}", pool_cache_file))?;
        Ok(pool_cache)
    } else {
        if Chain::Base == chain {
            Ok(PoolCache {
                last_synced_block: 0,
                pool_type: *pool_type,
                pools: Vec::new(),
            })
        } else {
            Ok(PoolCache {
                last_synced_block: 9_999_999,
                pool_type: *pool_type,
                pools: Vec::new(),
            })
        }
    }
}

pub fn write_cache_file(pool_cache: &PoolCache, chain: Chain) -> Result<()> {
    let pool_cache_file = format!("cache/{}_{}_cache.json", chain, pool_cache.pool_type);
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&pool_cache_file)
        .with_context(|| format!("Failed to create or open cache file: {}", pool_cache_file))?;
    let writer = BufWriter::new(file);
    serde_json::to_writer(writer, &pool_cache)
        .with_context(|| format!("Failed to serialize cache to file: {}", pool_cache_file))?;
    Ok(())
}