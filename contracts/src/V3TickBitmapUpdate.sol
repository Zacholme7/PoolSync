// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

interface IUniswapV3Pool {
    function tickBitmap(int16 wordPosition) external view returns (uint256);
    function slot0() external view returns (uint160 sqrtPriceX96, int24 tick, uint16 observationIndex, uint16 observationCardinality, uint16 observationCardinalityNext, uint8 feeProtocol, bool unlocked);
    function tickSpacing() external view returns (int24);
}

contract V3TickBitmapUpdate {
    struct V3TickBitmapData {
        address poolAddr;
        uint256[] tickBitmaps;
        int16[] wordPositions;
    }

    int24 constant TICKS_TO_FETCH = 20; // Number of ticks to fetch in each direction

    constructor(address[] memory pools) {
        V3TickBitmapData[] memory allPoolData = new V3TickBitmapData[](pools.length);

        for (uint256 i = 0; i < pools.length; ++i) {
            allPoolData[i] = processPool(pools[i]);
        }

        bytes memory encodedData = abi.encode(allPoolData);
        assembly {
            let dataStart := add(encodedData, 0x20)
            return(dataStart, sub(msize(), dataStart))
        }
    }

    function processPool(address poolAddress) internal view returns (V3TickBitmapData memory) {
        IUniswapV3Pool pool = IUniswapV3Pool(poolAddress);
        (, int24 currentTick,,,,,) = pool.slot0();
        int24 tickSpacing = pool.tickSpacing();
        
        int24 tickRange = int24(TICKS_TO_FETCH) * tickSpacing;
        int24 minTick = currentTick - tickRange;
        int24 maxTick = currentTick + tickRange;

        int16 minWord = int16(minTick >> 8);
        int16 maxWord = int16(maxTick >> 8);
        uint256 wordsToFetch = uint256(int256(maxWord - minWord + 1));

        V3TickBitmapData memory poolData;
        poolData.poolAddr = poolAddress;
        poolData.tickBitmaps = new uint256[](wordsToFetch);
        poolData.wordPositions = new int16[](wordsToFetch);

        for (int16 word = minWord; word <= maxWord; word++) {
            uint256 index = uint256(int256(word - minWord));
            poolData.wordPositions[index] = word;
            poolData.tickBitmaps[index] = pool.tickBitmap(word);
        }

        return poolData;
    }
}