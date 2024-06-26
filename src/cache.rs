use crate::pools::{PoolType, Pool};
use std::path::Path;
use std::fs;
use serde::{Serialize, Deserialize};
use serde_json;

/// Cache for a protocol, facilitates easier syncing
#[derive(Serialize, Deserialize)]
pub struct PoolCache {
        pub last_synced_block: u64,
        pub pool_type: PoolType,
        pub pools: Vec<Pool>
}

/// Read the cache file for the pool
pub fn read_cache_file(pool_type: &PoolType) -> PoolCache {
        let pool_cache_file = format!("{}_cache.json", pool_type);

        if Path::new(&pool_cache_file).exists() {
                let file_content = fs::read_to_string(pool_cache_file).unwrap();
                let pool_cache: PoolCache = serde_json::from_str(&file_content).unwrap();
                pool_cache
        } else {
                PoolCache {
                        last_synced_block: 10000000, 
                        pool_type: pool_type.clone(),
                        pools: Vec::new()
                }
        }
}

/// Write to the cache file
pub fn write_cache_file(pool_cache: &PoolCache) {
        let pool_cache_file = format!("{}_cache.json", pool_cache.pool_type);
        let json = serde_json::to_string(&pool_cache_file).unwrap();
        fs::write(pool_cache_file, json);
}


