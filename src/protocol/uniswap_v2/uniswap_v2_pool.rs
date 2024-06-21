use crate::errors::AMMError;
use crate::protocol::traits::AutomatedMarketMaker;
use async_trait::async_trait;
use std::sync::Arc;

use alloy_network::Network;
use alloy_primitives::Address;
use alloy_provider::Provider;
use alloy_sol_types::sol;
use alloy_transport::Transport;
use serde::{Deserialize, Serialize};

use crate::amm;
// Interface for UniswapV2Pair/Pool
sol! {
    #[derive(Debug, PartialEq, Eq)]
    #[sol(rpc)]
    contract IUniswapV2Pair {
        event Sync(uint112 reserve0, uint112 reserve1);
        function getReserves() external view returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast);
        function token0() external view returns (address);
        function token1() external view returns (address);
        function swap(uint256 amount0Out, uint256 amount1Out, address to, bytes calldata data);
    }
}


// A single amm pool between two tokens using uniswapV2
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UniswapV2Pool {
    pub address: Address,
    pub token_a: Address,
    pub token_a_decimals: u8,
    pub token_b: Address,
    pub token_b_decimals: u8,
    pub reserve_0: u128,
    pub reserve_1: u128,
    pub fee: u32,
}

impl UniswapV2Pool {
    /// Constructor function
    pub fn new(
        address: Address,
        token_a: Address,
        token_a_decimals: u8,
        token_b: Address,
        token_b_decimals: u8,
        reserve_0: u128,
        reserve_1: u128,
        fee: u32,
    ) -> UniswapV2Pool {
        UniswapV2Pool {
            address,
            token_a,
            token_a_decimals,
            token_b,
            token_b_decimals,
            reserve_0,
            reserve_1,
            fee,
        }
    }

    // Get the reserves this pool
    pub async fn get_reserves<T, N, P>(&self, provider: Arc<P>) -> Result<(u128, u128), AMMError>
    where
        T: Transport + Clone,
        N: Network,
        P: Provider<T, N>,
    {
        // Initialize a new instance of the Pool
        let v2_pair = IUniswapV2Pair::new(self.address, provider);

        // Make a call to get the reserves
        let IUniswapV2Pair::getReservesReturn {
            reserve0: reserve_0,
            reserve1: reserve_1,
            ..
        } = match v2_pair.getReserves().call().await {
            Ok(result) => result,
            Err(contract_error) => return Err(AMMError::ContractError(contract_error)),
        };

        Ok((reserve_0, reserve_1))
    }
}


// implement the automated market market trait
#[async_trait]
impl AutomatedMarketMaker for UniswapV2Pool {
    fn address(&self) -> Address {
        self.address
    }
    async fn sync<T, N, P>(&mut self, provider: Arc<P>) -> Result<(), AMMError>
    where
        T: Transport + Clone,
        N: Network,
        P: Provider<T, N>,
    {
        let (reserve_0, reserve_1) = self.get_reserves(provider.clone()).await?;

        self.reserve_0 = reserve_0;
        self.reserve_1 = reserve_1;

        Ok(())
    }

    async fn populate_data<T, N, P>(
        &mut self,
        _block_number: Option<u64>,
        provider: Arc<P>,
    ) -> Result<(), AMMError>
    where
        T: Transport + Clone,
        N: Network,
        P: Provider<T, N>,
    {
        todo!();
        //batch_request::get_v2_pool_data_batch_request(self, provider.clone()).await?;

        //Ok(())
    }
}


amm!(UniswapV2Pool);








