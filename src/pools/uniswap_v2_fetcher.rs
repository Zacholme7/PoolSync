use std::sync::Arc;
use alloy::{dyn_abi::{DynSolType, DynSolValue}, network::Network, primitives::{address, Address, U128}, providers::Provider, transports::Transport};
use alloy_sol_types::SolEvent;
use async_trait::async_trait;
use alloy::primitives::Log;
use crate::Chain;
use crate::pools::gen::UniswapV2Factory;
use super::{pool_structure::UniswapV2Pool, Pool, PoolFetcher, PoolType};

pub struct UniswapV2Fetcher;

#[async_trait]
impl PoolFetcher for UniswapV2Fetcher {
    fn pool_type(&self) -> PoolType {
        PoolType::UniswapV2
    }

    fn factory_address(&self, chain: Chain) -> Address {
        match chain {
            Chain::Ethereum => address!("5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f"),
            Chain::Base => address!("8909Dc15e40173Ff4699343b6eB8132c65e18eC6"),
        }
    }

    fn pair_created_signature(&self) -> &str {
        UniswapV2Factory::PairCreated::SIGNATURE
    }

    fn log_to_address(&self, log: &Log) -> Address {
        let decoded_log = UniswapV2Factory::PairCreated::decode_log(log, false).unwrap();
        decoded_log.data.pair
    }

    async fn build_pools_from_addrs<P, T, N>(
        &self,
        provider: Arc<P>,
        addresses: Vec<Address>,
    ) -> Vec<Pool>
    where
        P: Provider<T, N> + Sync + 'static,
        T: Transport + Sync + Clone,
        N: Network,
    {
        let uniswapv2_data: DynSolType = DynSolType::Array(Box::new(DynSolType::Tuple(vec![
            DynSolType::Address,
            DynSolType::Address,
            DynSolType::Address,
            DynSolType::Uint(8),
            DynSolType::Uint(8),
            DynSolType::Uint(112),
            DynSolType::Uint(112),
        ])));

        let contract_call = |provider: Arc<P>, addresses: Vec<Address>| async move {
            UniswapV2DataSync::deploy_builder(provider, addresses).await
        };

        let parse_pool = |data: &[DynSolValue]| UniswapV2Pool {
            address: data[0].as_address().unwrap(),
            token0: data[1].as_address().unwrap(),
            token1: data[2].as_address().unwrap(),
            token0_decimals: data[3].as_uint().unwrap().0.to::<u8>(),
            token1_decimals: data[4].as_uint().unwrap().0.to::<u8>(),
            token0_reserves: data[5].as_uint().unwrap().0.to::<U128>(),
            token1_reserves:data[6].as_uint().unwrap().0.to::<U128>(),
            ..Default::default()
        };

        generic_build_pools(provider, &addresses, &uniswapv2_data, contract_call, parse_pool)
            .await
            .unwrap_or_default()
    }
}