#[cfg(test)]
mod data_test {
    use alloy::sol;
    use alloy::providers::ProviderBuilder;
    use alloy::providers::Provider;
    use alloy::primitives::U256;
    use std::sync::Arc;
    use alloy::rpc::types::Filter;
    use crate::{Chain, PoolInfo, PoolSync, PoolType};

    #[tokio::test(flavor = "multi_thread")]
    async fn test_v2_data() {
        sol!{
            #[sol(rpc)]
            contract V2State {
                function getReserves() external view returns (uint256 reserve0, uint256 reserve1, uint256 blockTimestampLast);
            }
        }

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
        sol!{
            #[sol(rpc)]
            contract V3State {
                function slot0() external view returns (
                    uint160 sqrtPriceX96,
                    int24 tick,
                    uint16 observationIndex,
                    uint16 observationCardinality,
                    uint16 observationCardinalityNext,
                    uint8 feeProtocol,
                    bool unlocked
                );
                function liquidity() external view returns (uint128);
                function tickSpacing() external view returns (int24);
                function fee() external view returns (uint24);
                function ticks(int24 tick) external view returns (
                    uint128 liquidityGross,
                    int128 liquidityNet,
                    uint256 feeGrowthOutside0X128,
                    uint256 feeGrowthOutside1X128,
                    int56 tickCumulativeOutside,
                    uint160 secondsPerLiquidityOutsideX128,
                    uint32 secondsOutside,
                    bool initialized
                );
            }
        }

        // Sync in all uniswapv3 pools
        let pool_sync = PoolSync::builder()
            .add_pools(&[
                PoolType::UniswapV3,
                PoolType::SushiSwapV3,
                //PoolType::PancakeSwapV3,
                //PoolType::Slipstream,
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
        let current_block = provider.get_block_number().await.unwrap();
        println!("Last synced block {}, current block {}", last_synced_block, current_block);


        // for each pool, fetch the onchain data and confirm that it matches
        for pool in pools {
            let v3_pool = pool.get_v3().unwrap();
            let contract = V3State::new(pool.address(), provider.clone());

            // Get slot0 data
            let V3State::slot0Return { 
                sqrtPriceX96,
                tick,
                ..
            } = contract
                .slot0()
                .block(last_synced_block.into())
                .call()
                .await
                .unwrap();

            // Get liquidity
            let V3State::liquidityReturn { _0: liquidity } = contract
                .liquidity()
                .block(last_synced_block.into())
                .call()
                .await
                .unwrap();

            // Get tick spacing
            let V3State::tickSpacingReturn { _0: tick_spacing } = contract
                .tickSpacing()
                .block(last_synced_block.into())
                .call()
                .await
                .unwrap();

            // Get fee
            let V3State::feeReturn { _0: fee } = contract
                .fee()
                .block(last_synced_block.into())
                .call()
                .await
                .unwrap();

            // Get tick data
            /* 
            let V3State::ticksReturn { 
                liquidityNet,
                ..
            } = contract
                .ticks(tick)
                .block(last_synced_block.into())
                .call()
                .await
                .unwrap();
            */

            // Assert all values match
            assert_eq!(v3_pool.sqrt_price, U256::from(sqrtPriceX96), "SqrtPrice: Address {}, Pool Type {}", pool.address(), pool.pool_type());
            assert_eq!(v3_pool.tick, tick.as_i32(), "Tick: Address {}, Pool Type {}", pool.address(), pool.pool_type());
            assert_eq!(v3_pool.liquidity, liquidity as u128, "Liquidity: Address {}, Pool Type {}", pool.address(), pool.pool_type());
            assert_eq!(v3_pool.tick_spacing, tick_spacing.as_i32(), "Tick spacing: Address {}, Pool Type {}", pool.address(), pool.pool_type());
            assert_eq!(v3_pool.fee, fee.to::<u32>(), "Fee: Address {}, Pool Type {}", pool.address(), pool.pool_type());
        }
    }


}
