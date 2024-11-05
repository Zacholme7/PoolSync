use alloy::providers::RootProvider;
use alloy::primitives::U256;
use std::sync::Arc;
use alloy::transports::http::{Http, Client};

use crate::PoolType;
use crate::tests::abi_gen::*;
use crate::UniswapV3Pool;

#[cfg(test)]
mod data_test {
    use alloy::providers::ProviderBuilder;
    use crate::{PoolSync, PoolInfo, Chain};
    use super::*;


    #[tokio::test(flavor = "multi_thread")]
    async fn test_v2_data() {
        // Sync in all uniswapv2 pools
        let pool_sync = PoolSync::builder()
            .add_pools(&[
                PoolType::UniswapV2,
                PoolType::SushiSwapV2,
                PoolType::PancakeSwapV2,
                PoolType::Aerodrome,
                PoolType::BaseSwapV2,
                PoolType::SwapBasedV2,
                PoolType::DackieSwapV2,
                PoolType::AlienBaseV2,
            ])
            .chain(Chain::Base)
            .rate_limit(1000)
            .build().unwrap();
        let (pools, last_synced_block) = pool_sync.sync_pools().await.unwrap();

        let provider = Arc::new(ProviderBuilder::new()
            .on_http(std::env::var("FULL").unwrap().parse().unwrap()));
        // for each pool, fetch the onchain reserves and confirm that they are right
        for pool in pools {
            let V2State::getReservesReturn { reserve0: res0, reserve1: res1, .. } = 
            V2State::new(pool.address(), provider.clone())
                .getReserves()
                .block(last_synced_block.into())
                .call()
                .await
                .unwrap();
            assert_eq!(pool.get_v2().unwrap().token0_reserves, res0, "Address {}, Pool Type {}", pool.address(), pool.pool_type());
            assert_eq!(pool.get_v2().unwrap().token1_reserves, res1, "Address {}, Pool Type {}", pool.address(), pool.pool_type());
        }

    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_v3_data() {
        // Sync in all uniswapv3 pools
        let pool_sync = PoolSync::builder()
            .add_pools(&[
                //PoolType::UniswapV3,
                //PoolType::SushiSwapV3,
                //PoolType::PancakeSwapV3,
                PoolType::Slipstream,
                //PoolType::BaseSwapV3,
                //PoolType::SwapBasedV3,
                //PoolType::DackieSwapV3,
                //PoolType::AlienBaseV3,
            ])
            .chain(Chain::Base)
            .rate_limit(1000)
            .build().unwrap();
        let (pools, last_synced_block) = pool_sync.sync_pools().await.unwrap();
        let provider = Arc::new(ProviderBuilder::new()
            .on_http(std::env::var("FULL").unwrap().parse().unwrap()));


        // for each pool, fetch the onchain data and confirm that it matches
        for pool in pools {
            fetch_v3_pool_data(pool.get_v3().unwrap(), pool.pool_type(), last_synced_block, provider.clone()).await;
        }
    }


}

async fn fetch_v3_pool_data(
    pool: &UniswapV3Pool, 
    pool_type: PoolType,
    last_synced_block: u64,
    provider: Arc<RootProvider<Http<Client>>>,
) {
    // Get common pool data from either contract type
    let (sqrt_price, tick, liquidity, tick_spacing, fee) = match pool_type {
        PoolType::Slipstream => {
            let contract = V3StateNoFee::new(pool.address, provider.clone());
            let V3StateNoFee::slot0Return { sqrtPriceX96, tick, .. } = contract.slot0().block(last_synced_block.into()).call().await.unwrap();
            let V3StateNoFee::liquidityReturn { _0: liquidity } = contract.liquidity().block(last_synced_block.into()).call().await.unwrap();
            let V3StateNoFee::tickSpacingReturn { _0: tick_spacing } = contract.tickSpacing().block(last_synced_block.into()).call().await.unwrap();
            let V3StateNoFee::feeReturn { _0: fee } = contract.fee().block(last_synced_block.into()).call().await.unwrap();

            // Get and assert tick data
            for (tick_key, tick_val) in &pool.ticks {
                let V3StateNoFee::ticksReturn { liquidityGross, liquidityNet, .. } = contract
                    .ticks((*tick_key).try_into().unwrap())
                    .block(last_synced_block.into())
                    .call()
                    .await
                    .unwrap();
                
                assert_eq!(liquidityGross as u128, tick_val.liquidity_gross as u128, "Liquidity Gross at tick {}: Address {}, Pool Type {}", tick_key, pool.address, pool_type);
                assert_eq!(liquidityNet as i128, tick_val.liquidity_net as i128, "Liquidity Net at tick {}: Address {}, Pool Type {}", tick_key, pool.address, pool_type);
            }

            (sqrtPriceX96, tick, liquidity, tick_spacing, fee)
        },
        _ => {
            let contract = V3State::new(pool.address, provider.clone());
            let V3State::slot0Return { sqrtPriceX96, tick, .. } = contract.slot0().block(last_synced_block.into()).call().await.unwrap();
            let V3State::liquidityReturn { _0: liquidity } = contract.liquidity().block(last_synced_block.into()).call().await.unwrap();
            let V3State::tickSpacingReturn { _0: tick_spacing } = contract.tickSpacing().block(last_synced_block.into()).call().await.unwrap();
            let V3State::feeReturn { _0: fee } = contract.fee().block(last_synced_block.into()).call().await.unwrap();

            // Get and assert tick data
            for (tick_key, tick_val) in &pool.ticks {
                let V3State::ticksReturn { liquidityGross, liquidityNet, .. } = contract
                    .ticks((*tick_key).try_into().unwrap())
                    .block(last_synced_block.into())
                    .call()
                    .await
                    .unwrap();
                
                assert_eq!(liquidityGross as u128, tick_val.liquidity_gross as u128, "Liquidity Gross at tick {}: Address {}, Pool Type {}", tick_key, pool.address, pool_type);
                assert_eq!(liquidityNet as i128, tick_val.liquidity_net as i128, "Liquidity Net at tick {}: Address {}, Pool Type {}", tick_key, pool.address, pool_type);
            }

            (sqrtPriceX96, tick, liquidity, tick_spacing, fee)
        }
    };

    // Assert common values outside the match
    assert_eq!(pool.sqrt_price, U256::from(sqrt_price), "SqrtPrice: Address {}, Pool Type {}", pool.address, pool_type);
    assert_eq!(pool.tick, tick.as_i32(), "Tick: Address {}, Pool Type {}", pool.address, pool_type);
    assert_eq!(pool.liquidity, liquidity as u128, "Liquidity: Address {}, Pool Type {}", pool.address, pool_type);
    assert_eq!(pool.tick_spacing, tick_spacing.as_i32(), "Tick spacing: Address {}, Pool Type {}", pool.address, pool_type);
    assert_eq!(pool.fee, fee.to::<u32>(), "Fee: Address {}, Pool Type {}", pool.address, pool_type);
}
