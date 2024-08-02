// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

interface IUniswapV3Pool {
    function tickBitmap(int16 wordPosition) external view returns (uint256);
    function slot0() external view returns (uint160 sqrtPriceX96, int24 tick, uint16 observationIndex, uint16 observationCardinality, uint16 observationCardinalityNext, uint8 feeProtocol, bool unlocked);
}

contract V3TickBitmapUpdate {
    struct V3TickBitmapData {
        address poolAddr;
        uint256[31] tickBitmaps; // Fixed-length array of 7 elements
        int16[31] wordPositions;
    }

    constructor(address[] memory pools) {
        V3TickBitmapData[] memory allPoolData = new V3TickBitmapData[](pools.length);

        for (uint256 i = 0; i < pools.length; ++i) {
            address poolAddress = pools[i];
            (, int24 currentTick,,,,,) = IUniswapV3Pool(poolAddress).slot0();
            int16 wordPosition = int16(currentTick >> 8); // Equivalent to dividing by 256

            allPoolData[i].poolAddr = poolAddress;
            // Get tickBitmaps: 3 before, current, and 3 after
            for (int256 j = -15; j <= 15; j++) {
                uint256 bitmap = IUniswapV3Pool(poolAddress).tickBitmap(wordPosition + int16(j));
                allPoolData[i].wordPositions[uint256(j + 15)] = int16(wordPosition + int16(j));
                allPoolData[i].tickBitmaps[uint256(j + 15)] = bitmap;
            }
        }

        bytes memory _abiEncodedData = abi.encode(allPoolData);
        assembly {
            let dataStart := add(_abiEncodedData, 0x20)
            return(dataStart, sub(msize(), dataStart))
        }
    }
}
