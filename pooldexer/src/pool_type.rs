use crate::pooldexer::Chain;
use alloy_primitives::{Address, FixedBytes};
use alloy_rpc_types::Log;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PoolType {
    UniswapV2,
    SushiSwapV2,
    PancakeSwapV2,
    UniswapV3,
    SushiSwapV3,
    PancakeSwapV3,
    /*
    Aerodrome,
    Slipstream,
    BaseSwapV2,
    BaseSwapV3,
    AlienBaseV2,
    AlienBaseV3,
    MaverickV1,
    MaverickV2,
    CurveTwoCrypto,
    CurveTriCrypto,
    BalancerV2,
    SwapBasedV2,
    SwapBasedV3,
    DackieSwapV2,
    DackieSwapV3,
    */
}

// Dexes often yoink an entire codebase, change a fee, and slap a new name on the protocol and call
// it good. Due to this, a lot of these pool types share the exact same structure and can be
// abstracted upon
pub enum PoolStructure {
    UniswapV2,
    UniswapV3,
}

impl PoolType {
    fn structure(&self) -> PoolStructure {
        match self {
            PoolType::UniswapV2 | PoolType::SushiSwapV2 | PoolType::PancakeSwapV2 => {
                PoolStructure::UniswapV2
            }
            PoolType::UniswapV3 | PoolType::SushiSwapV3 | PoolType::PancakeSwapV3 => {
                PoolStructure::UniswapV3
            }
        }
    }

    pub fn pair_created_signature(&self) -> FixedBytes<32> {
        todo!()
    }

    // Parse a pool initialization log into its addresses
    pub fn log_to_address(&self, log: &Log) -> Address {
        todo!()
    }

    // Return the pool factory address for this specific pool type and chain
    pub fn factory_address(&self, chain: Chain) -> Address {
        todo!()
    }
}
