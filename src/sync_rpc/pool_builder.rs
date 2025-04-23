use crate::pool_structures::PoolBuilder;
use crate::Pool;
use crate::{
    BalancerV2Pool, CurveTriCryptoPool, CurveTwoCryptoPool, MaverickPool, UniswapV2Pool,
    UniswapV3Pool,
};
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
        let v3_structures = UniswapV3Pool::new(block_num, provider, addresses).await?;
        v3_structures
            .into_iter()
            .map(|pool| pool.into_typed_pool(*pool_type))
            .collect()
    } else if pool_type.is_maverick() {
        let maverick_structures = MaverickPool::new(block_num, provider, addresses).await?;
        maverick_structures
            .into_iter()
            .map(|pool| pool.into_typed_pool(*pool_type))
            .collect()
    } else if pool_type.is_balancer() {
        let balancer_structures = BalancerV2Pool::new(block_num, provider, addresses).await?;
        balancer_structures
            .into_iter()
            .map(|pool| pool.into_typed_pool(*pool_type))
            .collect()
    } else if pool_type.is_curve_two() {
        let curve_two_structures = CurveTwoCryptoPool::new(block_num, provider, addresses).await?;
        curve_two_structures
            .into_iter()
            .map(|pool| pool.into_typed_pool(*pool_type))
            .collect()
    } else if pool_type.is_curve_tri() {
        let curve_tri_structures = CurveTriCryptoPool::new(block_num, provider, addresses).await?;
        curve_tri_structures
            .into_iter()
            .map(|pool| pool.into_typed_pool(*pool_type))
            .collect()
    } else {
        return Err(PoolSyncError::UnsupportedPoolType);
    };

    Ok(pools)
}
