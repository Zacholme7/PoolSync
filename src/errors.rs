use thiserror::Error;

#[derive(Error, Debug)]
pub enum PoolSyncError {
    #[error("Provider error: {0}")]
    ProviderError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Pool not suppored")]
    UnsupportedPoolType,
    #[error("Chain not set")]
    ChainNotSet,
}
