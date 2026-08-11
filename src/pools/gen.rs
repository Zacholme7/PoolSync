use alloy::sol;

// UNISWAP
sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    UniswapV2Factory,
    "evm_abi/UniswapV2Factory.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    UniswapV3Factory,
    "evm_abi/UniswapV3Factory.json"
);

// SUSHISWAP
sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    SushiSwapV2Factory,
    "evm_abi/SushiSwapV2Factory.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    SushiSwapV3Factory,
    "evm_abi/SushiSwapV3Factory.json"
);

// PANCAKESWAP
sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    PancakeSwapV2Factory,
    "evm_abi/PancakeSwapV2Factory.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    PancakeSwapV3Factory,
    "evm_abi/PancakeSwapV3Factory.json"
);

// AERODOME
sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    AerodromeV2Factory,
    "evm_abi/AerodromeV2Factory.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    SlipstreamFactory,
    "evm_abi/SlipstreamFactory.json"
);

// ERC20
sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    ERC20,
    "evm_abi/ERC20.json"
);

// BASESWAP
sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    BaseSwapV2Factory,
    "evm_abi/BaseSwapV2Factory.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    BaseSwapV3Factory,
    "evm_abi/BaseSwapV3Factory.json"
);

// ALIENBASE
sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    AlienBaseV2Factory,
    "evm_abi/AlienBaseV2Factory.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    AlienBaseV3Factory,
    "evm_abi/AlienBaseV3Factory.json"
);

// MAVERICK
sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    MaverickV1Factory,
    "evm_abi/MaverickV1Factory.json"
);

// Curve
sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    TwoCryptoFactory,
    "evm_abi/TwoCryptoFactory.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    TriCryptoFactory,
    "evm_abi/TriCryptoFactory.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    BalancerV2Factory,
    "evm_abi/BalancerV2Factory.json"
);

// SwapBased
sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    SwapBasedV2Factory,
    "evm_abi/SwapBasedV2Factory.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    SwapBasedV3Factory,
    "evm_abi/SwapBasedV3Factory.json"
);

// dackieswap
sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    DackieSwapV2Factory,
    "evm_abi/DackieSwapV2Factory.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    DackieSwapV3Factory,
    "evm_abi/DackieSwapV3Factory.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    V2DataSync,
    "evm_abi/V2DataSync.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    V3DataSync,
    "evm_abi/V3DataSync.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    SlipStreamDataSync,
    "evm_abi/SlipStreamDataSync.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    MaverickDataSync,
    "evm_abi/MaverickDataSync.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    BalancerV2DataSync,
    "evm_abi/BalancerV2DataSync.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    TwoCurveDataSync,
    "evm_abi/TwoCurveDataSync.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    TriCurveDataSync,
    "evm_abi/TriCurveDataSync.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    Vault,
    "evm_abi/Vault.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    contract AerodromePool {
        function stable() external view returns (bool);
    }
);
