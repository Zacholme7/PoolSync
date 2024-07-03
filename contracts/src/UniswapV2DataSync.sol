//SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

interface IUniswapV2Pair {
    function token0() external view returns (address);

    function token1() external view returns (address);

    function getReserves() external view returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast);
}

interface IERC20 {
    function decimals() external view returns (uint8);
}


contract UniswapV2DataSync {
        struct PoolData {
                string token0Name;
                string token1Name;
                uint112 reserve0;
                uint112 reserve1;
        }


        constructor(address[] memory pools) {
                PoolData[] memory allPoolData = new PoolData[](pools.length);

                for (uint256 = 0; i < pools.length; i++) {
                        address pool = pools[i];

                        

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