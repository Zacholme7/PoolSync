use crate::protocol::traits::AutomatedMarketMaker;
use alloy_dyn_abi::{DynSolType, DynSolValue};
use alloy_network::Network;
use alloy_primitives::{Address, U256};
use alloy_provider::Provider;
use alloy_sol_types::sol;
use alloy_transport::Transport;
use async_trait::async_trait;
use log::info;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::batch_request;
use super::UniswapV2Pool;
use crate::{
    errors::AMMError, factory, protocol::constants::U256_1,
    protocol::traits::AutomatedMarketMakerFactory, protocol::AMM,
};

// Interface for the uniswapV2factory contract
sol! {
    #[derive(Debug, PartialEq, Eq)]
    #[sol(rpc)]
    contract IUniswapV2Factory {
        event PairCreated(address indexed token0, address indexed token1, address pair, uint256 index);
        function getPair(address tokenA, address tokenB) external view returns (address pair);
        function allPairs(uint256 index) external view returns (address pair);
        function allPairsLength() external view returns (uint256 length);
    }
}

// Inteface for getting all V2 amms
sol! {
    #[allow(missing_docs)]
    #[sol(rpc)]
    IGetUniswapV2PairsBatchRequest,
    "src/protocol/uniswap_v2/batch_request/GetUniswapV2PairsBatchRequestABI.json"
}

// Interface for fetching v2 amm pool data
sol! {
    #[allow(missing_docs)]
    #[sol(rpc)]
    IGetUniswapV2PoolDataBatchRequest,
    "src/protocol/uniswap_v2/batch_request/GetUniswapV2PoolDataBatchRequestABI.json"
}

// Represent the uniswap v2 factory
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct UniswapV2Factory {
    pub address: Address,
    pub creation_block: u64,
    pub fee: u32,
}

impl UniswapV2Factory {
    /// Create a new instance of the factory
    pub fn new(address: Address, creation_block: u64, fee: u32) -> UniswapV2Factory {
        UniswapV2Factory {
            address,
            creation_block,
            fee,
        }
    }

    /// Get all the AMMS from this factory
    async fn get_all_pairs_via_batched_calls<T, N, P>(
        &self,
        provider: Arc<P>,
    ) -> Result<Vec<AMM>, AMMError>
    where
        T: Transport + Clone,
        N: Network,
        P: Provider<T, N>,
    {
        // Create a new contract instance of the factory
        let factory = IUniswapV2Factory::new(self.address, provider.clone());

        // Get the length of all pairs, which is the amount of AMMs this factory has created
        let IUniswapV2Factory::allPairsLengthReturn {
            length: pairs_length,
        } = factory.allPairsLength().call().await?;

        // All of our pairs
        let mut pairs = vec![];

        // Max batch size
        let step = 766;

        // Initial indices to go from start at zero to either amount of AMMs or our step size
        let mut idx_from = U256::ZERO;
        let mut idx_to = if step > pairs_length.to::<usize>() {
            pairs_length
        } else {
            U256::from(step)
        };

        // Step through the range and fetch AMMs
        for _ in (0..pairs_length.to::<usize>()).step_by(step) {
            pairs.append(
                &mut self
                    .get_all_v2_amms(self.address, idx_from, idx_to, provider.clone())
                    .await?,
            );

            idx_from = idx_to;

            if idx_to + U256::from(step) > pairs_length {
                idx_to = pairs_length - U256_1
            } else {
                idx_to += U256::from(step);
            }
        }

        let mut amms = vec![];

        // Create new empty pools for each pair
        for addr in pairs {
            info!("{:?}", addr);
            let amm = UniswapV2Pool {
                address: addr,
                ..Default::default()
            };

            amms.push(AMM::UniswapV2Pool(amm));
        }

        Ok(amms)
    }

    // give a block number and the step, fetch the address of all V2 AMMs that
    // were created in that lbock range
    async fn get_all_v2_amms<T, N, P>(
        &self,
        factory: Address,
        from: U256,
        step: U256,
        provider: Arc<P>,
    ) -> Result<Vec<Address>, AMMError>
    where
        T: Transport + Clone,
        N: Network,
        P: Provider<T, N>,
    {
        let deployer =
            IGetUniswapV2PairsBatchRequest::deploy_builder(provider, from, step, factory);
        let res = deployer.call_raw().await?;

        let constructor_return = DynSolType::Array(Box::new(DynSolType::Address));
        let return_data_tokens = constructor_return.abi_decode_sequence(&res)?;

        let mut pairs = vec![];
        if let Some(tokens_arr) = return_data_tokens.as_array() {
            for token in tokens_arr {
                if let Some(addr) = token.as_address() {
                    if !addr.is_zero() {
                        info!("address {}", addr);
                        pairs.push(addr);
                    }
                }
            }
        };

        Ok(pairs)
    }

    // given a chunk of v2 amm addresses, construct pools and populate them with data
    pub async fn populate_all_v2_amms<T, N, P>(
        amms: &mut [AMM],
        provider: Arc<P>,
    ) -> Result<(), AMMError>
    where
        T: Transport + Clone,
        N: Network,
        P: Provider<T, N>,
    {
        // retrieve all the target addresses
        let target_addresses: Vec<_> = amms.iter().map(AMM::address).collect();

        // deploy our contract with the target addresses and call the constructor
        let deployer =
            IGetUniswapV2PoolDataBatchRequest::deploy_builder(provider, target_addresses);
        let res = deployer.call().await?;

        // decode the return, which is an array of tuples that contains all amm information
        let constructor_return = DynSolType::Array(Box::new(DynSolType::Tuple(vec![
            DynSolType::Address,
            DynSolType::Uint(8),
            DynSolType::Address,
            DynSolType::Uint(8),
            DynSolType::Uint(112),
            DynSolType::Uint(112),
        ])));
        let return_data_tokens = constructor_return.abi_decode_sequence(&res)?;

        // convert the returns into an array
        if let Some(tokens_arr) = return_data_tokens.as_array() {
            // for every token pool in the return
            for (pool_idx, token) in tokens_arr.iter().enumerate() {
                // extract the data
                if let Some(pool_data) = token.as_tuple() {
                    // extract the address
                    if let Some(address) = pool_data[0].as_address() {
                        // make sure the address is not zero then update the pool data
                        if !address.is_zero() {
                            let AMM::UniswapV2Pool(uniswap_v2_pool) = &mut amms[pool_idx];
                            if let Some(updated_pool) = Self::populate_pool_data_from_tokens(
                                uniswap_v2_pool.clone(),
                                pool_data,
                            ) {
                                info!("{:?}", updated_pool);
                                *uniswap_v2_pool = updated_pool;
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Populate pool data from the return
    #[inline]
    fn populate_pool_data_from_tokens(
        mut pool: UniswapV2Pool,
        tokens: &[DynSolValue],
    ) -> Option<UniswapV2Pool> {
        pool.token_a = tokens[0].as_address()?;
        pool.token_a_decimals = tokens[1].as_uint()?.0.to::<u8>();
        pool.token_b = tokens[2].as_address()?;
        pool.token_b_decimals = tokens[3].as_uint()?.0.to::<u8>();
        pool.reserve_0 = tokens[4].as_uint()?.0.to::<u128>();
        pool.reserve_1 = tokens[5].as_uint()?.0.to::<u128>();

        Some(pool)
    }
}

// Implement the factory trait
#[async_trait]
impl AutomatedMarketMakerFactory for UniswapV2Factory {
    // Get all the amms that were created with this factory
    async fn get_all_amms<T, N, P>(
        &self,
        to_block: Option<u64>,
        provider: Arc<P>,
        step: u64,
    ) -> Result<Vec<AMM>, AMMError>
    where
        T: Transport + Clone,
        N: Network,
        P: Provider<T, N>,
    {
        self.get_all_pairs_via_batched_calls(provider).await
    }

    /// Populates all AMMs data via batched static calls.
    async fn populate_amm_data<T, N, P>(
        &self,
        amms: &mut [AMM],
        block_number: Option<u64>,
        provider: Arc<P>,
    ) -> Result<(), AMMError>
    where
        T: Transport + Clone,
        N: Network,
        P: Provider<T, N>,
    {
        todo!()
    }
}
