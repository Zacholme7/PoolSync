//SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

interface IUniswapV2Pair {
    function token0() external view returns (address);

    function token1() external view returns (address);

    function getReserves()
        external
        view
        returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast);
}

interface IERC20Token {
    function decimals() external view returns (uint8);

    function symbol() external view returns (string memory);
}

contract UniswapV2DataSync {
    struct PoolData {
        address token0Address;
        address token1Address;
        string token0Symbol;
        string token1Symbol;
        uint8 token0Decimals;
        uint8 token1Decimals;
        uint112 reserve0;
        uint112 reserve1;
    }

    function syncPoolData(
        address[] memory poolAddresses
    ) external view returns (PoolData[] memory syncedPoolData) {
        PoolData[] memory allPoolData = new PoolData[](poolAddresses.length);

        for (uint256 i = 0; i < poolAddresses.length; i++) {
            address poolAddress = poolAddresses[i];

            PoolData memory poolData;

            // Pair
            IUniswapV2Pair pair = IUniswapV2Pair(poolAddress);

            // token addresss
            poolData.token0Address = pair.token0();
            poolData.token1Address = pair.token1();

            (poolData.reserve0, poolData.reserve1, ) = pair.getReserves();

            // Get the token specific information

            // token0
            IERC20Token token0 = IERC20Token(poolData.token0Address);
            poolData.token0Symbol = token0.symbol();
            poolData.token0Decimals = token0.decimals();

            // token1
            IERC20Token token1 = IERC20Token(poolData.token1Address);
            poolData.token1Symbol = token1.symbol();
            poolData.token1Decimals = token1.decimals();

            // save the populated data
            allPoolData[i] = poolData;
        }
        return allPoolData;
    }
}

