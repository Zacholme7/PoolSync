//SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;


interface IUniswapV2Pair {
    function getReserves() external view returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast);
}


contract V2ReserveUpdate {
    struct ReserveUpdate {
        address addr;
        uint112 tokenReserve0;
        uint112 tokenReserve1;
    }

    constructor(address[] memory pools) {
        ReserveUpdate[] memory allPools = new ReserveUpdate[](pools.length);

        for (uint256 i = 0; i < pools.length; ++i) {
            address poolAddress = pools[i];

            if (codeSizeIsZero(poolAddress)) continue;

            ReserveUpdate memory poolData;
            poolData.addr = poolAddress;

            // Get reserves
            (poolData.tokenReserve0, poolData.tokenReserve1, ) = IUniswapV2Pair(poolAddress).getReserves();

            allPools[i] = poolData;
        }

        bytes memory _abiEncodedData = abi.encode(allPools);

        assembly {
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
