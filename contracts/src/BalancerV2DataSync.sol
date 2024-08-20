// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

interface IVault {
    function getPoolTokens(bytes32 poolId) external view returns (
        address[] memory tokens,
        uint256[] memory balances,
        uint256 lastChangeBlock
    );
}

interface IBalancerV2Pool {
    function getPoolId() external view returns (bytes32);
    function getSwapFeePercentage() external view returns (uint256);
    function getNormalizedWeights() external view returns (uint256[] memory);
}

interface IERC20 {
    function decimals() external view returns (uint8);
}

contract BalancerV2DataSync {
    struct PoolData {
        address poolAddress;
        bytes32 poolId;
        address token0;
        address token1;
        uint8 token0Decimals;
        uint8 token1Decimals;
        address[] additionalTokens;
        uint8[] additionalTokenDecimals;
        uint256[] balances;
        uint256[] weights;
        uint256 swapFee;
    }

    IVault immutable vault;

    constructor(address[] memory pools) {
        address _vault = address(0xBA12222222228d8Ba445958a75a0704d566BF2C8);
        vault = IVault(_vault);
        PoolData[] memory allPoolData = new PoolData[](pools.length);

        for (uint256 i = 0; i < pools.length; ++i) {
            address poolAddress = pools[i];
            if (poolAddress.code.length == 0) continue;

            IBalancerV2Pool pool = IBalancerV2Pool(poolAddress);
            bytes32 poolId = pool.getPoolId();
            
            (address[] memory tokens, uint256[] memory balances, ) = vault.getPoolTokens(poolId);
            uint8[] memory decimals = new uint8[](tokens.length);
            uint256[] memory weights;
            
            try pool.getNormalizedWeights() returns (uint256[] memory _weights) {
                weights = _weights;
            } catch {
                weights = new uint256[](tokens.length);
            }

            for (uint256 j = 0; j < tokens.length; ++j) {
                if (tokens[j].code.length == 0) continue;
                decimals[j] = IERC20(tokens[j]).decimals();
            }

            address[] memory additionalTokens = new address[](tokens.length > 2 ? tokens.length - 2 : 0);
            uint8[] memory additionalTokenDecimals = new uint8[](tokens.length > 2 ? tokens.length - 2 : 0);
            for (uint256 j = 2; j < tokens.length; j++) {
                additionalTokens[j-2] = tokens[j];
                additionalTokenDecimals[j-2] = decimals[j];
            }

            allPoolData[i] = PoolData({
                poolAddress: poolAddress,
                poolId: poolId,
                token0: tokens[0],
                token1: tokens[1],
                token0Decimals: decimals[0],
                token1Decimals: decimals[1],
                additionalTokens: additionalTokens,
                additionalTokenDecimals: additionalTokenDecimals,
                balances: balances,
                weights: weights,
                swapFee: pool.getSwapFeePercentage()
            });
        }

        bytes memory encodedData = abi.encode(allPoolData);
        assembly {
            let dataStart := add(encodedData, 32)
            return(dataStart, sub(msize(), dataStart))
        }
    }
}