// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

interface IUniswapV2Pair {
    function token0() external view returns (address);
    function token1() external view returns (address);
    function getReserves() external view returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast);
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

    constructor(address[] memory poolAddresses) {
        PoolData[] memory allPoolData = new PoolData[](poolAddresses.length);
        string[] memory errorMessages = new string[](poolAddresses.length);

        for (uint256 i = 0; i < poolAddresses.length; i++) {
            (allPoolData[i], errorMessages[i]) = syncSinglePool(poolAddresses[i]);
        }

        bytes memory encodedData = abi.encode(allPoolData, errorMessages);

        assembly {
            return(add(encodedData, 32), sub(mload(encodedData), 32))
        }
    }

    function syncSinglePool(address poolAddress) internal view returns (PoolData memory poolData, string memory errorMessage) {
        try IUniswapV2Pair(poolAddress).token0() returns (address token0Address) {
            poolData.token0Address = token0Address;
            poolData.token1Address = IUniswapV2Pair(poolAddress).token1();
            
            (poolData.reserve0, poolData.reserve1, ) = IUniswapV2Pair(poolAddress).getReserves();

            try IERC20Token(poolData.token0Address).symbol() returns (string memory symbol) {
                poolData.token0Symbol = symbol;
            } catch {
                poolData.token0Symbol = "UNKNOWN";
            }

            try IERC20Token(poolData.token0Address).decimals() returns (uint8 decimals) {
                poolData.token0Decimals = decimals;
            } catch {
                poolData.token0Decimals = 18; // Default to 18 if unable to fetch
            }

            try IERC20Token(poolData.token1Address).symbol() returns (string memory symbol) {
                poolData.token1Symbol = symbol;
            } catch {
                poolData.token1Symbol = "UNKNOWN";
            }

            try IERC20Token(poolData.token1Address).decimals() returns (uint8 decimals) {
                poolData.token1Decimals = decimals;
            } catch {
                poolData.token1Decimals = 18; // Default to 18 if unable to fetch
            }

            errorMessage = "";
        } catch Error(string memory reason) {
            errorMessage = reason;
        } catch (bytes memory) {
            errorMessage = "Unknown error";
        }
    }
}