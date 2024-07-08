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
    )
        external
        view
        returns (PoolData[] memory syncedPoolData, string[] memory errors)
    {
        PoolData[] memory allPoolData = new PoolData[](poolAddresses.length);
        string[] memory errorMessages = new string[](poolAddresses.length);

        for (uint256 i = 0; i < poolAddresses.length; i++) {
            try this.syncSinglePool(poolAddresses[i]) returns (
                PoolData memory poolData
            ) {
                allPoolData[i] = poolData;
            } catch Error(string memory reason) {
                errorMessages[i] = reason;
            } catch (bytes memory /*lowLevelData*/) {
                errorMessages[i] = "Unknown error";
            }
        }

        return (allPoolData, errorMessages);
    }

    function syncSinglePool(
        address poolAddress
    ) external view returns (PoolData memory poolData) {
        IUniswapV2Pair pair = IUniswapV2Pair(poolAddress);
        poolData.token0Address = pair.token0();
        poolData.token1Address = pair.token1();
        (poolData.reserve0, poolData.reserve1, ) = pair.getReserves();

        IERC20Token token0 = IERC20Token(poolData.token0Address);
        poolData.token0Symbol = token0.symbol();
        poolData.token0Decimals = token0.decimals();

        IERC20Token token1 = IERC20Token(poolData.token1Address);
        poolData.token1Symbol = token1.symbol();
        poolData.token1Decimals = token1.decimals();
    }
}
