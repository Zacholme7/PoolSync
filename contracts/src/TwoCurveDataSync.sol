//SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

interface TwoCryptoFactory{
    function get_coins(address _pool) external view returns (address[2] memory);
}

interface IERC20 {
    function decimals() external view returns (uint8);
}

contract TwoCurveDataSync {
    struct PoolData {
        address poolAddr;
        address tokenA;
        address tokenB;
        uint8 tokenADecimals;
        uint8 tokenBDecimals;
    }

    constructor(address[] memory pools) {
        PoolData[] memory allPoolData = new PoolData[](pools.length);
        TwoCryptoFactory factory = TwoCryptoFactory(0x98EE851a00abeE0d95D08cF4CA2BdCE32aeaAF7F);

        for (uint256 i = 0; i < pools.length; ++i) {
            address poolAddress = pools[i];

            if (codeSizeIsZero(poolAddress)) continue;

            PoolData memory poolData;

            address[2] memory coins = factory.get_coins(poolAddress);

            // Get tokens A and B
            poolData.tokenA = coins[0];
            poolData.tokenB = coins[1];
            poolData.poolAddr = poolAddress;

            // Check that tokenA and tokenB do not have codesize of 0
            if (codeSizeIsZero(poolData.tokenA)) continue;
            if (codeSizeIsZero(poolData.tokenB)) continue;

            // Get tokenA decimals
            (
                bool tokenADecimalsSuccess,
                bytes memory tokenADecimalsData
            ) = poolData.tokenA.call{gas: 20000}(
                    abi.encodeWithSignature("decimals()")
                );

            if (tokenADecimalsSuccess) {
                uint256 tokenADecimals;

                if (tokenADecimalsData.length == 32) {
                    (tokenADecimals) = abi.decode(
                        tokenADecimalsData,
                        (uint256)
                    );

                    if (tokenADecimals == 0 || tokenADecimals > 255) {
                        continue;
                    } else {
                        poolData.tokenADecimals = uint8(tokenADecimals);
                    }
                } else {
                    continue;
                }
            } else {
                continue;
            }

            // Get tokenB decimals
            (
                bool tokenBDecimalsSuccess,
                bytes memory tokenBDecimalsData
            ) = poolData.tokenB.call{gas: 20000}(
                    abi.encodeWithSignature("decimals()")
                );

            if (tokenBDecimalsSuccess) {
                uint256 tokenBDecimals;

                if (tokenBDecimalsData.length == 32) {
                    (tokenBDecimals) = abi.decode(
                        tokenBDecimalsData,
                        (uint256)
                    );

                    if (tokenBDecimals == 0 || tokenBDecimals > 255) {
                        continue;
                    } else {
                        poolData.tokenBDecimals = uint8(tokenBDecimals);
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
