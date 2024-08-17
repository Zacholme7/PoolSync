
use alloy::primitives::{address, Address};
use alloy_sol_types::SolEvent;
use crate::pools::PoolFetcher;
use alloy::primitives::Log;
use crate::pools::PoolType;
use crate::Chain;
use alloy::sol;

sol!(
    #[derive(Debug)]
    event PoolCreated(
        IMaverickV2Pool poolAddress,
        uint8 protocolFeeRatio,
        uint256 feeAIn,
        uint256 feeBIn,
        uint256 tickSpacing,
        uint256 lookback,
        int32 activeTick,
        IERC20 tokenA,
        IERC20 tokenB,
        uint8 kinds,
        address accessor
    );
);

pub struct MaverickV1Fetcher;

impl PoolFetcher for MaverickV1Fetcher {
    fn pool_type(&self) -> PoolType {
        PoolType::MaverickV1
    }

    fn factory_address(&self, chain: Chain) -> Address {
        match chain {
            Chain::Ethereum => address!("Eb6625D65a0553c9dBc64449e56abFe519bd9c9B"),
            Chain::Base => address!("B2855783a346735e4AAe0c1eb894DEf861Fa9b45"),
        }
    }

    fn pair_created_signature(&self) -> &str {
        MaverickV1Factory::PoolCreated::SIGNATURE
        
    }

    fn log_to_address(&self, log: &Log) -> Address {
        let decoded_log = MaverickV1Factory::PoolCreated::decode_log(log, false).unwrap();
        decoded_log.data.poolAddress
    }
}

