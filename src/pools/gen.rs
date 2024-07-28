use alloy::sol;


sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    UniswapV2Factory,
    "src/pools/abis/UniswapV2Factory.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    ERC20,
    "src/pools/abis/ERC20.json"
);