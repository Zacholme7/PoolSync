//! Pool Synchronization Cache Implementation
//!
//! This module provides functionality for caching pool synchronization data,
//! including structures and functions for reading from and writing to cache files.

use crate::chain::Chain;
use crate::pools::{Pool, PoolType};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Cache for a protocol, facilitates easier syncing
#[derive(Serialize, Deserialize)]
pub struct PoolCache {
    /// The last block number that was synced
    pub last_synced_block: u64,
    /// The type of pool this cache is for
    pub pool_type: PoolType,
    /// The list of pools that have been synced
    pub pools: Vec<Pool>,
}

/// Reads the cache file for the specified pool type and chain
pub fn read_cache_file(pool_type: &PoolType, chain: Chain) -> PoolCache {
    let pool_cache_file = format!("cache/{}_{}_cache.json", chain, pool_type);
    if Path::new(&pool_cache_file).exists() {
        let file_content = fs::read_to_string(pool_cache_file).unwrap();
        let pool_cache: PoolCache = serde_json::from_str(&file_content).unwrap();
        pool_cache
    } else {
        PoolCache {
            last_synced_block: 20000000,
            pool_type: *pool_type,
            pools: Vec::new(),
        }
    }
}

/// Writes the provided PoolCache to a cache file
pub fn write_cache_file(pool_cache: &PoolCache, chain: Chain) {
    let pool_cache_file = format!("cache/{}_{}_cache.json", chain, pool_cache.pool_type);
    let json = serde_json::to_string(&pool_cache).unwrap();
    let _ = fs::write(pool_cache_file, json);
}
