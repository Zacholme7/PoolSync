use alloy::sol;


sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    UniswapV2Factory,
    "src/pools/abis/UniswapV2Factory.json"
);