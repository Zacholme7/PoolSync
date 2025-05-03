use alloy_sol_types::sol;

// Generate bindings for onchain contract events
sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    contract AerodromeSync {
        event Sync(uint256 reserve0, uint256 reserve1);
    }
);

sol!(
    #[derive(Debug)]
    event PoolCreated(
        address poolAddress,
        uint8 protocolFeeRatio,
        uint256 feeAIn,
        uint256 feeBIn,
        uint256 tickSpacing,
        uint256 lookback,
        int32 activeTick,
        address tokenA,
        address tokenB,
        uint8 kinds,
        address accessor
    );
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    contract BalancerV2Event {
        event Swap(
            bytes32 indexed poolId,
            address indexed tokenIn,
            address indexed tokenOut,
            uint256 amountIn,
            uint256 amountOut
        );
    }
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    contract PancakeSwapEvents {
        event Swap(
            address indexed sender,
            address indexed recipient,
            int256 amount0,
            int256 amount1,
            uint160 sqrtPriceX96,
            uint128 liquidity,
            int24 tick,
            uint128 protocolFeesToken0,
            uint128 protocolFeesToken1
        );
    }
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    contract DataEvents {
        event Sync(uint112 reserve0, uint112 reserve1);
        event Swap(address indexed sender, address indexed recipient, int256 amount0, int256 amount1, uint160 sqrtPriceX96, uint128 liquidity, int24 tick);
        event Burn(address indexed owner, int24 indexed tickLower, int24 indexed tickUpper, uint128 amount, uint256 amount0, uint256 amount1);
        event Mint(address sender, address indexed owner, int24 indexed tickLower, int24 indexed tickUpper, uint128 amount, uint256 amount0, uint256 amount1);
    }
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    contract AerodromePool {
        function stable() external view returns (bool);
    }
);

// Pool contract binding gen
sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    UniswapV2Factory,
    "src/onchain/pool_abis/UniswapV2Factory.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    UniswapV3Factory,
    "src/onchain/pool_abis/UniswapV3Factory.json"
);

// SUSHISWAP
sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    SushiSwapV2Factory,
    "src/onchain/pool_abis/SushiSwapV2Factory.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    SushiSwapV3Factory,
    "src/onchain/pool_abis/SushiSwapV3Factory.json"
);

// PANCAKESWAP
sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    PancakeSwapV2Factory,
    "src/onchain/pool_abis/PancakeSwapV2Factory.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    PancakeSwapV3Factory,
    "src/onchain/pool_abis/PancakeSwapV3Factory.json"
);

// AERODOME
sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    AerodromeV2Factory,
    "src/onchain/pool_abis/AerodromeV2Factory.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    SlipstreamFactory,
    "src/onchain/pool_abis/SlipstreamFactory.json"
);

// ERC20
sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    ERC20,
    "src/onchain/pool_abis/ERC20.json"
);

// BASESWAP
sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    BaseSwapV2Factory,
    "src/onchain/pool_abis/BaseSwapV2Factory.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    BaseSwapV3Factory,
    "src/onchain/pool_abis/BaseSwapV3Factory.json"
);

// ALIENBASE
sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    AlienBaseV2Factory,
    "src/onchain/pool_abis/AlienBaseV2Factory.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    AlienBaseV3Factory,
    "src/onchain/pool_abis/AlienBaseV3Factory.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    Vault,
    "src/onchain/pool_abis/Vault.json"
);

// MAVERICK
sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    MaverickV1Factory,
    "src/onchain/pool_abis/MaverickV1Factory.json"
);

// Curve
sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    TwoCryptoFactory,
    "src/onchain/pool_abis/TwoCryptoFactory.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    TriCryptoFactory,
    "src/onchain/pool_abis/TriCryptoFactory.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    BalancerV2Factory,
    "src/onchain/pool_abis/BalancerV2Factory.json"
);

// SwapBased
sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    SwapBasedV2Factory,
    "src/onchain/pool_abis/SwapBasedV2Factory.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    SwapBasedV3Factory,
    "src/onchain/pool_abis/SwapBasedV3Factory.json"
);

// dackieswap
sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    DackieSwapV2Factory,
    "src/onchain/pool_abis/DackieSwapV2Factory.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    DackieSwapV3Factory,
    "src/onchain/pool_abis/DackieSwapV3Factory.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    V2DataSync,
    "src/onchain/contract_abis/V2DataSync.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    V3DataSync,
    "src/onchain/contract_abis/V3DataSync.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    SlipStreamDataSync,
    "src/onchain/contract_abis/SlipStreamDataSync.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    MaverickDataSync,
    "src/onchain/contract_abis/MaverickDataSync.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    BalancerV2DataSync,
    "src/onchain/contract_abis/BalancerV2DataSync.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    TwoCurveDataSync,
    "src/onchain/contract_abis/TwoCurveDataSync.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    TriCurveDataSync,
    "src/onchain/contract_abis/TriCurveDataSync.json"
);
