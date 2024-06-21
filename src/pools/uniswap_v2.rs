use super::Token;
use super::Pool;
use alloy::primitives::{Address, U256};
use alloy::providers::RootProvider;
use alloy::transports::http::{Client, Http};
use crate::constants::UNISWAPV2_FACTORY;

use alloy::sol_types::{sol, SolCall};
use async_trait::async_trait;

// UniswapV2 contract interfaces
sol!(
    #[sol(rpc)]
    interface IUniswapV2Factory {
        function allPairsLength() external view returns (uint);
        function allPairs(uint) external view returns (address);
    }
);

sol!(
    #[sol(rpc)]
    interface IUniswapV2Pair {
        function token0() external view returns (address);
        function token1() external view returns (address);
        function getReserves() external view returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast);
    }
);

pub struct UniswapV2Factory {
}


/// A UniswapV2 AMM/pool
struct UniswapV2Pool {
    address: Address,
    token0: Token,
    token1: Token,
    reserve0: U256,
    reserve1: U256,
    last_synced_block: u64,
}

// Implement the PoolSync trait for UniswapV2
#[async_trait]
impl Pool for UniswapV2Pool{
    async fn get_all_pools(provider: &RootProvider<Http<Client>>) {
        // construct the univ2 factory (will get all our pools)
        let factory = IUniswapV2Factory::new(UNISWAPV2_FACTORY, provider);

        // get the amount of pairs (pools) that have been created by this factor
        let IUniswapV2Factory::allPairsLengthReturn { _0 } = factory.allPairsLength().call().await.unwrap();

        // fetch each pool address
        let num = _0.to::<u64>();
        for pair in 0..num {
            let IUniswapV2Factory::allPairsReturn { _0 } = factory.allPairs(U256::from(pair)).call().await.unwrap();
            println!("{:?}", _0);
    
        }
    }
}




