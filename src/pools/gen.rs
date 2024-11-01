use alloy::sol;

// UNISWAP
sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    UniswapV2Factory,
    "src/pools/abis/UniswapV2Factory.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    UniswapV3Factory,
    "src/pools/abis/UniswapV3Factory.json"
);

// SUSHISWAP
sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    SushiSwapV2Factory,
    "src/pools/abis/SushiSwapV2Factory.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    SushiSwapV3Factory,
    "src/pools/abis/SushiSwapV3Factory.json"
);

// PANCAKESWAP
sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    PancakeSwapV2Factory,
    "src/pools/abis/PancakeSwapV2Factory.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    PancakeSwapV3Factory,
    "src/pools/abis/PancakeSwapV3Factory.json"
);

// AERODOME
sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    AerodromeV2Factory,
    "src/pools/abis/AerodromeV2Factory.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    SlipstreamFactory,
    "src/pools/abis/SlipstreamFactory.json"
);

// ERC20
sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    ERC20,
    "src/pools/abis/ERC20.json"
);

// BASESWAP
sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    BaseSwapV2Factory,
    "src/pools/abis/BaseSwapV2Factory.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    BaseSwapV3Factory,
    "src/pools/abis/BaseSwapV3Factory.json"
);

// ALIENBASE
sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    AlienBaseV2Factory,
    "src/pools/abis/AlienBaseV2Factory.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    AlienBaseV3Factory,
    "src/pools/abis/AlienBaseV3Factory.json"
);

// MAVERICK
sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    MaverickV1Factory,
    "src/pools/abis/MaverickV1Factory.json"
);

// Curve
sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    TwoCryptoFactory,
    "src/pools/abis/TwoCryptoFactory.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    TriCryptoFactory,
    "src/pools/abis/TriCryptoFactory.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    BalancerV2Factory,
    "src/pools/abis/BalancerV2Factory.json"
);

// SwapBased
sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    SwapBasedV2Factory,
    "src/pools/abis/SwapBasedV2Factory.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    SwapBasedV3Factory,
    "src/pools/abis/SwapBasedV3Factory.json"
);

// dackieswap
sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    DackieSwapV2Factory,
    "src/pools/abis/DackieSwapV2Factory.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    DackieSwapV3Factory,
    "src/pools/abis/DackieSwapV3Factory.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    V2DataSync,
    "src/abi/V2DataSync.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    V3DataSync,
    "src/abi/V3DataSync.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    SlipStreamDataSync,
    "src/abi/SlipStreamDataSync.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    MaverickDataSync,
    "src/abi/MaverickDataSync.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    BalancerV2DataSync,
    "src/abi/BalancerV2DataSync.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    TwoCurveDataSync,
    "src/abi/TwoCurveDataSync.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    TriCurveDataSync,
    "src/abi/TriCurveDataSync.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    Vault,
    "src/pools/abis/Vault.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    contract AerodromePool {
        function stable() external view returns (bool);
    }
);
