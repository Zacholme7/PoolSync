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
    AerodomeV2Factory,
    "src/pools/abis/AerodomeV2Factory.json"
);

// ERC20
sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    ERC20,
    "src/pools/abis/ERC20.json"
);