use crate::{Chain, Pool, PoolInfo, PoolSyncError};
use alloy_primitives::Address;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use std::sync::{Arc, Mutex};
use tracing::{debug, info};

/// Database connection manager for pool syncing
pub struct PoolDatabase {
    /// SQLite connection wrapped in a mutex for thread safety
    connection: Arc<Mutex<Connection>>,
}

/// Enumeration of table names in the database
enum TableName {
    SyncState,
    Pools,
}

impl TableName {
    fn as_str(&self) -> &'static str {
        match self {
            TableName::SyncState => "sync_state",
            TableName::Pools => "pools",
        }
    }
}

impl PoolDatabase {
    /// Creates a new DatabaseSyncer with the given database path
    /// If the database doesn't exist, it will be created and initialized
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self, PoolSyncError> {
        let conn = Connection::open(db_path).map_err(|e| {
            PoolSyncError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to open database: {}", e),
            ))
        })?;

        let db_syncer = Self {
            connection: Arc::new(Mutex::new(conn)),
        };

        // Initialize database schema
        db_syncer.initialize_database()?;

        Ok(db_syncer)
    }

    /// Initialize database schema if it doesn't exist
    fn initialize_database(&self) -> Result<(), PoolSyncError> {
        let mut conn = self.connection.lock().unwrap();
        let tx = conn.transaction().map_err(|e| {
            PoolSyncError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to begin transaction: {}", e),
            ))
        })?;

        // Create sync_state table
        tx.execute(
            &format!(
                "CREATE TABLE IF NOT EXISTS {} (
                    chain TEXT NOT NULL,
                    pool_type TEXT NOT NULL,
                    last_block INTEGER NOT NULL,
                    PRIMARY KEY (chain, pool_type)
                )",
                TableName::SyncState.as_str()
            ),
            [],
        )
        .map_err(|e| {
            PoolSyncError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to create sync_state table: {}", e),
            ))
        })?;

        // Create pools table
        tx.execute(
            &format!(
                "CREATE TABLE IF NOT EXISTS {} (
                    address TEXT PRIMARY KEY,
                    pool_type TEXT NOT NULL,
                    chain TEXT NOT NULL,
                    data TEXT NOT NULL,
                    updated_at INTEGER NOT NULL
                )",
                TableName::Pools.as_str()
            ),
            [],
        )
        .map_err(|e| {
            PoolSyncError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to create pools table: {}", e),
            ))
        })?;

        // Create indices for performance
        tx.execute(
            &format!(
                "CREATE INDEX IF NOT EXISTS idx_pools_chain_type ON {} (chain, pool_type)",
                TableName::Pools.as_str()
            ),
            [],
        )
        .map_err(|e| {
            PoolSyncError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to create index: {}", e),
            ))
        })?;

        tx.commit().map_err(|e| {
            PoolSyncError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to commit transaction: {}", e),
            ))
        })?;

        info!("Database schema initialized successfully");
        Ok(())
    }

    /// Gets the last processed block for a chain and pool type
    pub fn get_last_processed_block(
        &self,
        chain: Chain,
        pool_type: crate::PoolType,
    ) -> Result<Option<u64>, PoolSyncError> {
        let conn = self.connection.lock().unwrap();
        let result = conn
            .query_row(
                &format!(
                    "SELECT last_block FROM {} WHERE chain = ?1 AND pool_type = ?2",
                    TableName::SyncState.as_str()
                ),
                params![chain.to_string(), pool_type.to_string()],
                |row| row.get::<_, u64>(0),
            )
            .optional()
            .map_err(|e| {
                PoolSyncError::IoError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to query last processed block: {}", e),
                ))
            })?;

        Ok(result)
    }

    /// Updates the last processed block for a chain and pool type
    pub fn update_last_processed_block(
        &self,
        chain: Chain,
        pool_type: crate::PoolType,
        block: u64,
    ) -> Result<(), PoolSyncError> {
        let conn = self.connection.lock().unwrap();
        conn.execute(
            &format!(
                "INSERT INTO {} (chain, pool_type, last_block) VALUES (?1, ?2, ?3)
                 ON CONFLICT(chain, pool_type) DO UPDATE SET last_block = ?3",
                TableName::SyncState.as_str()
            ),
            params![chain.to_string(), pool_type.to_string(), block],
        )
        .map_err(|e| {
            PoolSyncError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to update last processed block: {}", e),
            ))
        })?;

        debug!(
            "Updated last processed block for chain {:?} pool type {:?} to {}",
            chain, pool_type, block
        );
        Ok(())
    }

    /// Save pools to the database
    pub fn save_pools(&self, pools: &[Pool], chain: Chain) -> Result<(), PoolSyncError> {
        let mut conn = self.connection.lock().unwrap();
        let tx = conn.transaction().map_err(|e| {
            PoolSyncError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to begin transaction: {}", e),
            ))
        })?;

        let now = chrono::Utc::now().timestamp() as u64;

        for pool in pools {
            let address = pool.address().to_string();
            let pool_type = pool.pool_type().to_string();
            let serialized_data =
                serde_json::to_string(pool).map_err(|e| PoolSyncError::JsonError(e))?;

            tx.execute(
                &format!(
                    "INSERT INTO {} (address, pool_type, chain, data, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5)
                     ON CONFLICT(address) DO UPDATE SET 
                     pool_type = ?2, chain = ?3, data = ?4, updated_at = ?5",
                    TableName::Pools.as_str()
                ),
                params![address, pool_type, chain.to_string(), serialized_data, now],
            )
            .map_err(|e| {
                PoolSyncError::IoError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to save pool {}: {}", address, e),
                ))
            })?;
        }

        tx.commit().map_err(|e| {
            PoolSyncError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to commit transaction: {}", e),
            ))
        })?;

        info!("Saved {} pools to database", pools.len());
        Ok(())
    }

    /// Load pools from the database for a specific chain and pool types
    pub fn load_pools(
        &self,
        chain: Chain,
        pool_types: Option<&[crate::PoolType]>,
    ) -> Result<Vec<Pool>, PoolSyncError> {
        let conn = self.connection.lock().unwrap();

        let sql = if pool_types.is_some() {
            format!(
                "SELECT data FROM {} WHERE chain = ?1 AND pool_type IN ({})",
                TableName::Pools.as_str(),
                pool_types
                    .unwrap()
                    .iter()
                    .map(|_| "?")
                    .collect::<Vec<_>>()
                    .join(",")
            )
        } else {
            format!(
                "SELECT data FROM {} WHERE chain = ?1",
                TableName::Pools.as_str()
            )
        };

        let mut stmt = conn.prepare(&sql).map_err(|e| {
            PoolSyncError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to prepare statement: {}", e),
            ))
        })?;

        // Build params
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(chain.to_string())];

        if let Some(types) = pool_types {
            for pool_type in types {
                params.push(Box::new(pool_type.to_string()));
            }
        }

        let pool_rows = stmt
            .query_map(
                rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())),
                |row| row.get::<_, String>(0),
            )
            .map_err(|e| {
                PoolSyncError::IoError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to query pools: {}", e),
                ))
            })?;

        let mut pools = Vec::new();
        for row_result in pool_rows {
            let serialized_data = row_result.map_err(|e| {
                PoolSyncError::IoError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to get pool data: {}", e),
                ))
            })?;

            let pool: Pool =
                serde_json::from_str(&serialized_data).map_err(|e| PoolSyncError::JsonError(e))?;

            pools.push(pool);
        }

        info!("Loaded {} pools from database", pools.len());
        Ok(pools)
    }

    /// Get all known addresses for a pool type
    pub fn get_pool_addresses(
        &self,
        chain: Chain,
        pool_type: crate::PoolType,
    ) -> Result<Vec<Address>, PoolSyncError> {
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn
            .prepare(&format!(
                "SELECT address FROM {} WHERE chain = ?1 AND pool_type = ?2",
                TableName::Pools.as_str()
            ))
            .map_err(|e| {
                PoolSyncError::IoError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to prepare statement: {}", e),
                ))
            })?;

        let address_rows = stmt
            .query_map(params![chain.to_string(), pool_type.to_string()], |row| {
                row.get::<_, String>(0)
            })
            .map_err(|e| {
                PoolSyncError::IoError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to query pool addresses: {}", e),
                ))
            })?;

        let mut addresses = Vec::new();
        for row_result in address_rows {
            let address_str = row_result.map_err(|e| {
                PoolSyncError::IoError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to get pool address: {}", e),
                ))
            })?;

            let address = address_str.parse::<Address>().map_err(|_| {
                PoolSyncError::IoError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to parse address: {}", address_str),
                ))
            })?;

            addresses.push(address);
        }

        debug!(
            "Loaded {} pool addresses for chain {:?}, type {:?}",
            addresses.len(),
            chain,
            pool_type
        );
        Ok(addresses)
    }
}
