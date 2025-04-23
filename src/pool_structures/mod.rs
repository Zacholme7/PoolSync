//! Defines the various structures that pools map into
//! Typically, one dex will engineer a new variant of
//! a pool representation and other protcols will hit fork
//! and slap a new name on.
//!
//! Maybe theyll even get crafty and change a fee param...
//!
//! These structures define common pool representations from
//! the original protocols which happen to map to various other protocols
//!
//! Adding new protocls that contain a fork of these variants are very easy,
//! otherwise you must implement a new structure alongside the parsing and
//! liquidity populating logic
pub mod balancer_v2_structure;
pub mod maverick_structure;
pub mod tri_crypto_curve_structure;
pub mod two_crypto_curve_structure;
pub mod v2_structure;
pub mod v3_structure;

use crate::{Pool, PoolSyncError, PoolType};
use alloy_dyn_abi::{DynSolType, DynSolValue};
use alloy_primitives::Address;
use alloy_primitives::Bytes;
use alloy_provider::RootProvider;
use std::sync::Arc;

// Define a standardized interface for building pools from a set of addresses
pub trait PoolBuilder: From<Vec<DynSolValue>>
where
    Self: Sized,
{
    // Construct a new instance of the type that implements this trait
    async fn new(
        end_block: u64,
        provider: Arc<RootProvider>,
        addresses: &[Address],
    ) -> Result<Vec<Self>, PoolSyncError> {
        // Fetch the raw data
        let raw_data = Self::get_raw_pool_data(end_block, provider, addresses).await?;

        // Decode the sequence and parse into Self
        let raw_data = Self::get_pool_repr()
            .abi_decode_sequence(&raw_data)
            .map_err(|_| PoolSyncError::FailedDeployment)?;

        let raw_data_iter = iter_raw_pool_data(raw_data);

        Ok(raw_data_iter
            .map(|pool_bytes| Self::from(pool_bytes))
            //.filter(|pool| pool.is_valid())
            .collect())
    }

    // Fetch the raw pool data for the address set at end_block
    // eth_Call a deployment at end_block for initial information
    async fn get_raw_pool_data(
        end_block: u64,
        provider: Arc<RootProvider>,
        addresses: &[Address],
    ) -> Result<Bytes, PoolSyncError>;

    // Dynamic type to parse byte sequence from contract deployment into understandable
    // representation
    fn get_pool_repr() -> DynSolType;

    // Consume self and construct a top level Pool
    fn into_typed_pool(self, pool_type: PoolType) -> Pool;
}


// Provide an iterator over the data. Useful for pasing into typed pool
fn iter_raw_pool_data(data: DynSolValue) -> impl Iterator<Item = Vec<DynSolValue>> {
    // Extract the array (or return empty iterator if not an array)
    let array = match data.as_array() {
        Some(arr) => arr.to_owned(),
        None => Vec::new(),
    };

    // Create an iterator that yields owned Vec<DynSolValue> tuples
    array.into_iter().filter_map(|tuple_value| {
        // Convert to an owned Vec if it's actually a tuple
        tuple_value.as_tuple().map(|t| t.to_vec())
    })
}
