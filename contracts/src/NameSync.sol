//SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

interface IERC20 {
    function symbol() external view returns (string memory);
}

interface IERC20Bytes32 {
    function symbol() external view returns (bytes32);
}

contract NameSync {
    function syncNames(address token0, address token1) external view returns (string memory token0_name, string memory token1_name) {
        token0_name = getSymbol(token0);
        token1_name = getSymbol(token1);
    }

    function getSymbol(address token) internal view returns (string memory) {
        // Try string symbol first
        try IERC20(token).symbol() returns (string memory s) {
            return s;
        } catch {
            // If string symbol fails, try bytes32
            try IERC20Bytes32(token).symbol() returns (bytes32 s) {
                return bytes32ToString(s);
            } catch {
                // If both fail, return a default value
                return string(abi.encodePacked("UNK_", toAsciiString(token)));
            }
        }
    }

    function bytes32ToString(bytes32 _bytes32) internal pure returns (string memory) {
        uint8 i = 0;
        while(i < 32 && _bytes32[i] != 0) {
            i++;
        }
        bytes memory bytesArray = new bytes(i);
        for (i = 0; i < 32 && _bytes32[i] != 0; i++) {
            bytesArray[i] = _bytes32[i];
        }
        return string(bytesArray);
    }

    function toAsciiString(address x) internal pure returns (string memory) {
        bytes memory s = new bytes(40);
        for (uint i = 0; i < 20; i++) {
            bytes1 b = bytes1(uint8(uint(uint160(x)) / (2**(8*(19 - i)))));
            bytes1 hi = bytes1(uint8(b) / 16);
            bytes1 lo = bytes1(uint8(b) - 16 * uint8(hi));
            s[2*i] = char(hi);
            s[2*i+1] = char(lo);            
        }
        return string(s);
    }

    function char(bytes1 b) internal pure returns (bytes1 c) {
        if (uint8(b) < 10) return bytes1(uint8(b) + 0x30);
        else return bytes1(uint8(b) + 0x57);
    }
}
