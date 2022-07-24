// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.6.0;
pragma experimental ABIEncoderV2;

contract DeriveEip712Test {
    uint256 constant chainId = 1;
    bytes32 constant salt = keccak256("eip712-test-75F0CCte");
    bytes32 constant EIP712_DOMAIN_TYPEHASH =
        keccak256(
            "EIP712Domain(string name,string version,uint256 chainId,address verifyingContract,bytes32 salt)"
        );

    bytes32 constant FOOBAR_DOMAIN_TYPEHASH =
        keccak256(
            "FooBar(int256 foo,uint256 bar,bytes fizz,bytes32 buzz,string far,address out)"
        );

    struct FooBar {
        int256 foo;
        uint256 bar;
        bytes fizz;
        bytes32 buzz;
        string far;
        address out;
    }

    constructor() public {}

    function domainSeparator() public pure returns (bytes32) {
        return
            keccak256(
                abi.encode(
                    EIP712_DOMAIN_TYPEHASH,
                    keccak256("Eip712Test"),
                    keccak256("1"),
                    chainId,
                    address(0x0000000000000000000000000000000000000001),
                    salt
                )
            );
    }

    function typeHash() public pure returns (bytes32) {
        return FOOBAR_DOMAIN_TYPEHASH;
    }

    function structHash(FooBar memory fooBar) public pure returns (bytes32) {
        return
            keccak256(
                abi.encode(
                    typeHash(),
                    uint256(fooBar.foo),
                    fooBar.bar,
                    keccak256(fooBar.fizz),
                    fooBar.buzz,
                    keccak256(bytes(fooBar.far)),
                    fooBar.out
                )
            );
    }

    function encodeEip712(FooBar memory fooBar) public pure returns (bytes32) {
        return
            keccak256(
                abi.encodePacked(
                    "\x19\x01",
                    domainSeparator(),
                    structHash(fooBar)
                )
            );
    }

    function verifyFooBar(
        address signer,
        FooBar memory fooBar,
        bytes32 r,
        bytes32 s,
        uint8 v
    ) public pure returns (bool) {
        return signer == ecrecover(encodeEip712(fooBar), v, r, s);
    }
}
