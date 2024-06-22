// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

interface IUniswapV2Factory {
        function allPairs(uint256) external view returns (address);
        function allPairsLength() external view returns (uint);
}

contract UniswapV2Sync {
        address constant FACTORY = 0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f;
        function getAllPairs(uint256 start, uint256 end) public view returns (address [] memory addresses) {
                // Construct the factory and fetch the number of pairs
                IUniswapV2Factory factory = IUniswapV2Factory(FACTORY);
                uint totalPairs = factory.allPairsLength();

                // if we go past the end, readjust
                if (totalPairs < end) {
                        end = totalPairs;
                } 

                // fetch all of the pair addrsses for the range
                uint256 count = end - start;
                address[] memory pairs = new address[](count);
                for(uint256 i = 0; i < count; i++) {
                        pairs[i] = factory.allPairs(start + i);
                }

                return pairs;
        }
}
