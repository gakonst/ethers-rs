# Contracts

In ethers-rs, contracts are a way to interact with smart contracts on the Ethereum blockchain through rust bindings, which serve as a robust rust API to these objects.

The ethers-contracts module includes the following features:

- [Abigen](): A module for generating Rust code from Solidity contracts.
- [Compile](): A module for compiling Solidity contracts into bytecode and ABI files.
- [Creating Instances](): A module for creating instances of smart contracts.
- [Deploy Anvil](): A module for deploying smart contracts on the Anvil network.
- [Deploy from ABI and bytecode](): A module for deploying smart contracts from their ABI and bytecode files.
- [Deploy Moonbeam](): A module for deploying smart contracts on the Moonbeam network.
- [Events](): A module for listening to smart contract events.
- [Events with Meta](): A module for listening to smart contract events with metadata.
- [Methods](): A module for calling smart contract methods.

The ethers-contracts module provides a convenient way to work with Ethereum smart contracts in Rust. With this module, you can easily create instances of smart contracts, deploy them to the network, and interact with their methods and events.

The Abigen module allows you to generate Rust code from Solidity contracts, which can save you a lot of time and effort when writing Rust code for Ethereum smart contracts.

The Compile module makes it easy to compile Solidity contracts into bytecode and ABI files, which are required for deploying smart contracts.

The Deploy Anvil and Deploy Moonbeam modules allow you to deploy smart contracts to specific networks, making it easy to test and deploy your smart contracts on the desired network.

The Events and Events with Meta modules allow you to listen to smart contract events and retrieve event data, which is essential for building applications that interact with Ethereum smart contracts.

Finally, the Methods module provides a simple way to call smart contract methods from Rust code, allowing you to interact with smart contracts in a programmatic way.

Overall, the ethers-contracts module provides a comprehensive set of tools for working with Ethereum smart contracts in Rust, making it an essential tool for Rust developers building decentralized applications on the Ethereum network.
