//SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;


interface IUniswapV3Pool {
    function tickBitmap(int16 wordPosition) external view returns (uint256);
}

contract V3TickBitmapUpdate {
    struct V3TickBitmapData {
        address poolAddr;
        uint256[7] tickBitmaps;  // Fixed-length array of 7 elements
    }

    function getTickBitmaps(address[] memory pools, int24[] memory currentTicks) 
        external 
        view 
        returns (V3TickBitmapData[] memory) 
    {
        V3TickBitmapData[] memory allPoolData = new V3TickBitmapData[](pools.length);
        
        for (uint256 i = 0; i < pools.length; ++i) {
            address poolAddress = pools[i];
            int24 currentTick = currentTicks[i];
            int16 wordPosition = int16(currentTick / 256);  // Cast to int16 for tickBitmap function
            
            V3TickBitmapData memory poolData;
            poolData.poolAddr = poolAddress;

            // Get tickBitmaps: 3 before, current, and 3 after
            for(int16 j = -3; j <= 3; j++) {
                uint256 bitmap = IUniswapV3Pool(poolAddress).tickBitmap(wordPosition + j);
                poolData.tickBitmaps[uint256(j + 3)] = bitmap;
            }

            allPoolData[i] = poolData;
        }

        return allPoolData;
    }
}