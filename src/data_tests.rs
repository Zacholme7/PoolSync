
#[cfg(test)]
mod data_test {
    use alloy::sol;
    use alloy::providers::ProviderBuilder;
    use alloy::primitives::U256;
    use std::sync::Arc;
    use crate::{Chain, PoolInfo, PoolSync, PoolType};

    #[tokio::test(flavor = "multi_thread")]
    async fn test_v2_data() {
        sol!{
            #[sol(rpc)]
            contract V2State {
                function getReserves() external view returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast);
            }
        }

        // Sync in all uniswapv2 pools
        let pool_sync = PoolSync::builder()
            .add_pool(PoolType::UniswapV2)
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
            assert_eq!(pool.get_v2().unwrap().token0_reserves, U256::from(res0));
            assert_eq!(pool.get_v2().unwrap().token1_reserves, U256::from(res1));
        }

    }

}