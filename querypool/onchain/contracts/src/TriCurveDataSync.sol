//SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
interface TriCryptoFactory{
    function get_coins(address _pool) external view returns (address[3] memory);
}

interface IERC20 {
    function decimals() external view returns (uint8);
}

contract TriCurveDataSync {
    struct PoolData {
        address poolAddress;
        address token0;
        address token1;
        address token2;
        uint8 token0Decimals;
        uint8 token1Decimals;
        uint8 token2Decimals;
    }

    constructor(address factoryAddr, address[] memory pools) {
        PoolData[] memory allPoolData = new PoolData[](pools.length);
        TriCryptoFactory factory = TriCryptoFactory(factoryAddr);

        for (uint256 i = 0; i < pools.length; ++i) {
            address poolAddress = pools[i];

            if (codeSizeIsZero(poolAddress)) continue;

            PoolData memory poolData;

            address[3] memory coins = factory.get_coins(poolAddress);

            // Get tokens A and B
            poolData.token0 = coins[0];
            poolData.token1 = coins[1];
            poolData.token2 = coins[2];
            poolData.poolAddress = poolAddress;

            // Check that tokenA and tokenB do not have codesize of 0
            if (codeSizeIsZero(poolData.token0)) continue;
            if (codeSizeIsZero(poolData.token1)) continue;
            if (codeSizeIsZero(poolData.token2)) continue;

            // Get token0 decimals
            (
                bool token0DecimalsSuccess,
                bytes memory token0DecimalsData
            ) = poolData.token0.call{gas: 20000}(
                    abi.encodeWithSignature("decimals()")
                );

            if (token0DecimalsSuccess) {
                uint256 token0Decimals;

                if (token0DecimalsData.length == 32) {
                    (token0Decimals) = abi.decode(
                        token0DecimalsData,
                        (uint256)
                    );

                    if (token0Decimals == 0 || token0Decimals > 255) {
                        continue;
                    } else {
                        poolData.token0Decimals = uint8(token0Decimals);
                    }
                } else {
                    continue;
                }
            } else {
                continue;
            }

            // Get token1 decimals
            (
                bool token1DecimalsSuccess,
                bytes memory token1DecimalsData
            ) = poolData.token1.call{gas: 20000}(
                    abi.encodeWithSignature("decimals()")
                );

            if (token1DecimalsSuccess) {
                uint256 token1Decimals;

                if (token1DecimalsData.length == 32) {
                    (token1Decimals) = abi.decode(
                        token1DecimalsData,
                        (uint256)
                    );

                    if (token1Decimals == 0 || token1Decimals > 255) {
                        continue;
                    } else {
                        poolData.token1Decimals = uint8(token1Decimals);
                    }
                } else {
                    continue;
                }
            } else {
                continue;
            }

            // Get token2 decimals
            (
                bool token2DecimalsSuccess,
                bytes memory token2DecimalsData
            ) = poolData.token2.call{gas: 20000}(
                    abi.encodeWithSignature("decimals()")
                );

            if (token2DecimalsSuccess) {
                uint256 token2Decimals;

                if (token2DecimalsData.length == 32) {
                    (token2Decimals) = abi.decode(
                        token2DecimalsData,
                        (uint256)
                    );

                    if (token2Decimals == 0 || token2Decimals > 255) {
                        continue;
                    } else {
                        poolData.token2Decimals = uint8(token2Decimals);
                    }
                } else {
                    continue;
                }
            } else {
                continue;
            }

            allPoolData[i] = poolData;
        }

        // ensure abi encoding, not needed here but increase reusability for different return types
        // note: abi.encode add a first 32 bytes word with the address of the original data
        bytes memory _abiEncodedData = abi.encode(allPoolData);

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
