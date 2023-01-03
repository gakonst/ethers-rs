# Contracts
In this guide, we will go over some examples of using ethers-rs to work with contracts, including using abigen to generate Rust bindings for a contract, listening for contract events, calling contract methods, and instantiating contracts.

## Generating Rust bindings with abigen
To use a contract with ethers-rs, you will need to generate Rust bindings using the abigen tool. abigen is included with the ethers-rs library and can be used to generate Rust bindings for any Solidity contract.

### Generate a Rust file
This method takes a smart contract's Application Binary Interface (ABI) file and generates a Rust file to interact with it. This is useful if the smart contract is referenced in different places in a project. File generation from ABI can also be easily included as a build step of your application.

Running the code below will generate a file called `token.rs` containing the bindings inside, which exports an `ERC20Token` struct, along with all its events and methods. Put into a `build.rs` file this will generate the bindings during cargo build.

```rust
Abigen::new("ERC20Token", "./abi.json")?.generate()?.write_to_file("token.rs")?;
```

### Generate inline Rust bindings
This method takes a smart contract's solidity definition and generates inline Rust code to interact with it. This is useful for fast prototyping and for tight scoped use-cases of your contracts. Inline Rust generation uses the `abigen!` macro to expand Rust contract bindings.

Running the code below will generate bindings for the `ERC20Token` struct, along with all its events and methods.
```rust
abigen!(
    ERC20Token,
    r#"[
        function approve(address spender, uint256 amount) external returns (bool)
        event Transfer(address indexed from, address indexed to, uint256 value)
        event Approval(address indexed owner, address indexed spender, uint256 value)
    ]"#,
);
```

Another way to get the same result, is to provide the ABI contract's definition as follows.
```rust 
abigen!(ERC20Token, "./abi.json",);
```

## Contract instances
## Contract methods
## Contract events

