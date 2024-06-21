use crate::errors::AMMError;
use crate::protocol::AMM;
use alloy_network::Network;
use alloy_provider::Provider;
use alloy_transport::Transport;
use async_trait::async_trait;
use std::sync::Arc;

#[async_trait]
pub trait AutomatedMarketMakerFactory {
    /// Gets all Pools from the factory created logs up to the `to_block` block number.
    /// Returns a vector of AMMs.
    async fn get_all_amms<T, N, P>(
        &self,
        to_block: Option<u64>,
        provider: Arc<P>,
        step: u64,
    ) -> Result<Vec<AMM>, AMMError>
    where
        T: Transport + Clone,
        N: Network,
        P: Provider<T, N>;

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
        P: Provider<T, N>;
}

// Macro used to generate a common Factory enum to represent all of the pools we support
#[macro_export]
macro_rules! factory {
    ($($factory_type:ident),+ $(,)?) => {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub enum Factory {
            $($factory_type($factory_type),)+
        }

        #[async_trait]
        impl AutomatedMarketMakerFactory for Factory {

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
                match self {
                    $(Factory::$factory_type(factory) => {
                        factory.get_all_amms(to_block, provider, step).await
                    },)+
                }
            }

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
                match self {
                    $(Factory::$factory_type(factory) => {
                        factory.populate_amm_data(amms, block_number, provider).await
                    },)+
                }
            }

        }
    };
}
