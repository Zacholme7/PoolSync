use crate::onchain::UniswapV2Factory;
use crate::pools::PoolFetcher;
use crate::pools::PoolType;
use crate::Pool;
use crate::Chain;
use alloy_dyn_abi::DynSolType;
use alloy_primitives::{address, Address, Log};
use alloy_sol_types::SolEvent;


pub struct UniswapV2Fetcher;

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
        let decoded_log = UniswapV2Factory::PairCreated::decode_log(log).unwrap();
        decoded_log.data.pair
    }

    fn get_pool_repr(&self) -> DynSolType {
        todo!()
    }

}
/*
*
*
*
*
* impl UniswapV2Pool {
    // Public function to get raw pool data
    pub async fn get_raw_pool_data(
        end_block: u64,
        provider: Arc<RootProvider>,
        addresses: &[Address],
    ) -> Result<impl Iterator<Item = Vec<DynSolValue>>, PoolSyncError> {
        let bytes = V2DataSync::deploy_builder(provider.clone(), addresses.to_vec())
            .call_raw()
            .block(end_block.into())
            .await
            .map_err(|_| PoolSyncError::FailedDeployment)?;
            
        let data = Self::get_pool_repr().abi_decode_sequence(&bytes).unwrap();
        
        Ok(Self::iter_raw_pool_data(data))
    }
}
Then in your code, you could use it like this:


*/
