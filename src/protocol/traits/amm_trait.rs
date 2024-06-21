use crate::errors::AMMError;
use alloy_network::Network;
use alloy_primitives::Address;
use alloy_provider::Provider;
use alloy_transport::Transport;
use async_trait::async_trait;
use std::sync::Arc;

// Defines common functionality for AMM
#[async_trait]
pub trait AutomatedMarketMaker {
    /// Returns the address of the AMM
    fn address(&self) -> Address;

    /// Syncs the AMM data on chain via batched static calls.
    async fn sync<T, N, P>(&mut self, provider: Arc<P>) -> Result<(), AMMError>
    where
        T: Transport + Clone,
        N: Network,
        P: Provider<T, N>;

    /// Populates the AMM data via batched static calls.
    async fn populate_data<T, N, P>(
        &mut self,
        block_number: Option<u64>,
        middleware: Arc<P>,
    ) -> Result<(), AMMError>
    where
        T: Transport + Clone,
        N: Network,
        P: Provider<T, N>;
}

// Macro used to generate a common AMM enum to represent all of the pools we support
#[macro_export]
macro_rules! amm {
    ($($pool_type:ident),+ $(,)?) => {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub enum AMM {
            $($pool_type($pool_type),)+
        }

        #[async_trait]
        impl AutomatedMarketMaker for AMM {
            fn address(&self) -> Address{
                match self {
                    $(AMM::$pool_type(pool) => pool.address(),)+
                }
            }
            async fn sync<T, N, P>(&mut self, middleware: Arc<P>) -> Result<(), AMMError>
            where
                T: Transport + Clone,
                N: Network,
                P: Provider<T, N>,
            {
                match self {
                    $(AMM::$pool_type(pool) => pool.sync(middleware).await,)+
                }
            }


            async fn populate_data<T, N, P>(&mut self, block_number: Option<u64>, middleware: Arc<P>) -> Result<(), AMMError>
            where
                T: Transport + Clone,
                N: Network,
                P: Provider<T, N>,
            {
                match self {
                    $(AMM::$pool_type(pool) => pool.populate_data(block_number, middleware).await,)+
                }
            }

        }
    };
}
