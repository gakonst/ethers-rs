# ethers-contract

Type-safe abstractions for interacting with Ethereum smart contracts.

Interacting with a smart contract requires broadcasting carefully crafted
[transactions](ethers_core::types::TransactionRequest) where the `data` field
contains the
[function's selector](https://ethereum.stackexchange.com/questions/72363/what-is-a-function-selector)
along with the arguments of the called function.

This module provides the [`Contract`] and [`ContractFactory`] abstractions so
that you do not have to worry about that. It also provides typesafe bindings via
the [`abigen`] macro and the [`Abigen` builder].

For more information, please refer to the [book](https://gakonst.com/ethers-rs).

[`contractfactory`]: ./struct.ContractFactory.html
[`contract`]: ./struct.Contract.html
[`abigen`]: ./macro.abigen.html
[`abigen` builder]: ./struct.Abigen.html
