//! ensure console.sol can be generated via abigen!

ethers_contract::abigen!(HardhatConsole, "./tests/it/solidity-contracts/console.json",);

fn assert_console_calls(_: &hardhat_console::HardhatConsoleCalls) {}
