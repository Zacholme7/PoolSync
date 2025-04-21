//! PoolSync Error Types
//!
//! This module defines the custom error types used throughout the PoolSync operations.
//! It leverages the `thiserror` crate for deriving the `Error` trait and providing
//! formatted error messages.

use thiserror::Error;

/// Enumerates the various error types that can occur during PoolSync operations
#[derive(Error, Debug)]
pub enum PoolSyncError {
    /// Represents errors that occur when interacting with the blockchain provider
    #[error("Provider error: {0}")]
    ProviderError(String),

    /// Represents I/O errors that may occur during file operations
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Represents errors that occur during JSON serialization or deserialization
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Indicates that an unsupported pool type was encountered
    #[error("Pool not supported")]
    UnsupportedPoolType,

    #[error("Failed to fetch pool addresses")]
    FetchAddressError,

    /// Indicates that the chain was not set when it was required
    #[error("Chain not set")]
    ChainNotSet,

    /// Rpc endpoint is not set
    #[error("Rpc endpoint not set")]
    EndpointNotSet,

    // Unable to parse endpoint
    #[error("Failed to parse endpoint into URL")]
    ParseEndpointError,

    #[error("Deployment failed")]
    FailedDeployment,
}
