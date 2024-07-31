//SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

interface IUniswapV3Pool {
    function liquidity() external view returns (uint128);
    function slot0()
        external
        view
        returns (
            uint160 sqrtPriceX96,
            int24 tick,
            uint16 observationIndex,
            uint16 observationCardinality,
            uint16 observationCardinalityNext,
            uint8 feeProtocol,
            bool unlocked
        );
    function ticks(int24 tick)
        external
        view
        returns (
            uint128 liquidityGross,
            int128 liquidityNet,
            uint256 feeGrowthOutside0X128,
            uint256 feeGrowthOutside1X128,
            int56 tickCumulativeOutside,
            uint160 secondsPerLiquidityOutsideX128,
            uint32 secondsOutside,
            bool initialized
        );
}

interface IERC20 {
    function decimals() external view returns (uint8);
}

contract V3StateUpdate {
    struct StateUpdate {
        address poolAddr;
        uint128 liquidity;
        uint160 sqrtPrice;
        int24 tick;
    }


    constructor(address[] memory pools) {
        StateUpdate[] memory allPools = new StateUpdate[](pools.length);

        for (uint256 i = 0; i < pools.length; ++i) {
            address poolAddress = pools[i];

            if (codeSizeIsZero(poolAddress)) continue;

            StateUpdate memory poolData;

            poolData.poolAddr = poolAddress;

            IUniswapV3Pool pool = IUniswapV3Pool(poolAddress);
            (uint160 sqrtPriceX96, int24 tick, , , , , ) = pool.slot0();
            poolData.liquidity = pool.liquidity();
            poolData.sqrtPrice = sqrtPriceX96;
            poolData.tick = tick;

            allPools[i] = poolData;
        }

        bytes memory _abiEncodedData = abi.encode(allPools);
        assembly {
            // Return from the start of the data (discarding the original data address)
            // up to the end of the memory used
            let dataStart := add(_abiEncodedData, 0x20)
            return(dataStart, sub(msize(), dataStart))
        }
    }

    function codeSizeIsZero(address target) internal view returns (bool) {
        if (target.code.length == 0) {
            return true;
        } else {
            return false;
        }
    }
}