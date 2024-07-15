//SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

interface IUniswapV2Pair {
    function token0() external view returns (address);
    function token1() external view returns (address);
    function getReserves() external view returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast);
}

interface IERC20 {
    function decimals() external view returns (uint8);
    function name() external view returns (string memory);
}

contract UniswapV2DataSync {
    struct PoolData {
        address poolAdr;
        address tokenA;
        uint8 tokenADecimals;
        string tokenAName;
        address tokenB;
        uint8 tokenBDecimals;
        string tokenBName;
        uint112 reserve0;
        uint112 reserve1;
    }

    constructor(address[] memory pools) {
        PoolData[] memory allPoolData = new PoolData[](pools.length);

        for (uint256 i = 0; i < pools.length; ++i) {
            address poolAddress = pools[i];
            if (poolAddress.code.length == 0) continue;

            PoolData memory poolData;
            poolData.poolAdr = poolAddress;

            poolData.tokenA = IUniswapV2Pair(poolAddress).token0();
            poolData.tokenB = IUniswapV2Pair(poolAddress).token1();

            if (poolData.tokenA.code.length == 0 || poolData.tokenB.code.length == 0) continue;

            (bool success, bytes memory data) = poolData.tokenA.staticcall(abi.encodeWithSignature("decimals()"));
            if (success && data.length == 32) {
                uint256 decimals = abi.decode(data, (uint256));
                if (decimals > 0 && decimals <= 255) poolData.tokenADecimals = uint8(decimals);
                else continue;
            } else continue;

            (success, data) = poolData.tokenA.staticcall(abi.encodeWithSignature("name()"));
            poolData.tokenAName = success && data.length > 0 ? abi.decode(data, (string)) : "";

            (success, data) = poolData.tokenB.staticcall(abi.encodeWithSignature("decimals()"));
            if (success && data.length == 32) {
                uint256 decimals = abi.decode(data, (uint256));
                if (decimals > 0 && decimals <= 255) poolData.tokenBDecimals = uint8(decimals);
                else continue;
            } else continue;

            (success, data) = poolData.tokenB.staticcall(abi.encodeWithSignature("name()"));
            poolData.tokenBName = success && data.length > 0 ? abi.decode(data, (string)) : "";

            (poolData.reserve0, poolData.reserve1,) = IUniswapV2Pair(poolAddress).getReserves();

            allPoolData[i] = poolData;
        }

        bytes memory encodedData = abi.encode(allPoolData);
        assembly {
            return(add(encodedData, 32), mload(encodedData))
        }
    }
}