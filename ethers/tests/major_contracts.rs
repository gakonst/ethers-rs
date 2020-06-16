// This test exists to ensure that the abigen macro works "reasonably" well with popular contracts
use ethers::contract::abigen;

abigen!(
    KeepBonding,
    "etherscan:0x7137701e90C6a80B0dA36922cd83942b32A8fc95"
);
abigen!(cDAI, "etherscan:0x5d3a536E4D6DbD6114cc1Ead35777bAB948E3643");
abigen!(
    Comptroller,
    "etherscan:0x3d9819210a31b4961b30ef54be2aed79b9c9cd3b"
);

// https://github.com/vyperlang/vyper/issues/1931
// abigen!(
//     Curve,
//     "etherscan:0xa2b47e3d5c44877cca798226b7b8118f9bfb7a56"
// );
abigen!(
    UmaAdmin,
    "etherscan:0x4E6CCB1dA3C7844887F9A5aF4e8450d9fd90317A"
);

// e.g. aave's `initialize` methods exist multiple times, so we should rename it
abigen!(
    AavePoolCore,
    "etherscan:0x3dfd23a6c5e8bbcfc9581d2e864a68feb6a076d3",
    methods {
        initialize(address,bytes) as initialize_proxy;
    }
);

// // Abi Encoder v2 is still buggy
// abigen!(
//     DyDxLimitOrders,
//     "etherscan:0xDEf136D9884528e1EB302f39457af0E4d3AD24EB"
// );
