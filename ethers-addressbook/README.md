# ethers-addressbook

A collection of commonly used smart contract addresses.

For more information, please refer to the [book](https://gakonst.com/ethers-rs).

## Examples

```rust
use ethers_addressbook::{contract, Chain};

let weth = contract("weth").unwrap();
let mainnet_address = weth.address(Chain::Mainnet).unwrap();
assert_eq!(mainnet_address, "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".parse().unwrap());
```
