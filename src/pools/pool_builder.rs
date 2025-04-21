use crate::pools::pool_structures::PoolBuilder;
use crate::Pool;
use crate::UniswapV2Pool;
use alloy_primitives::Address;
use alloy_provider::RootProvider;
use std::sync::Arc;

use crate::errors::PoolSyncError;
use crate::PoolType;

pub async fn build_pools(
    addresses: &[Address],
    pool_type: &PoolType,
    provider: Arc<RootProvider>,
    block_num: u64,
) -> Result<Vec<Pool>, PoolSyncError> {
    let pools = if pool_type.is_v2() {
        let v2_structures = UniswapV2Pool::new(block_num, provider, addresses).await?;
        v2_structures
            .into_iter()
            .map(|pool| pool.into_typed_pool(*pool_type))
            .collect()
    } else if pool_type.is_v3() {
        todo!()
    } else if pool_type.is_maverick() {
        todo!()
    } else if pool_type.is_balancer() {
        todo!()
    } else if pool_type.is_curve_two() {
        todo!()
    } else if pool_type.is_curve_tri() {
        todo!()
    } else {
        return Err(PoolSyncError::UnsupportedPoolType);
    };

    Ok(pools)
}
