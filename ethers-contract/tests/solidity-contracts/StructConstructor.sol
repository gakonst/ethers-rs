// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.8;

// To sync with StructContractor.json run:

// CONTRACT=ethers-contract/tests/solidity-contracts/StructConstructor
// BIN=$(solc --bin $CONTRACT.sol | sed '4q;d' | tr -d '\n')
// ABI=$(solc --abi $CONTRACT.sol | tail -n 1)
// echo "{\"abi\": $ABI, \"bin\": \"$BIN\"}" > $CONTRACT.json

contract MyContract {
    struct ConstructorParams {
        uint256 x;
        uint256 y;
    }

    ConstructorParams _params;

    constructor(ConstructorParams memory params) {
        _params = params;
    }
}
