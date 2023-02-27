# ethers-etherscan

Bindings for the [etherscan.io web API](https://docs.etherscan.io).

For more information, please refer to the [book](https://gakonst.com/ethers-rs).

## Examples

```rust,no_run
# use ethers_core::types::Chain;
# use ethers_etherscan::Client;
# async fn foo() -> Result<(), Box<dyn std::error::Error>> {
let client = Client::new(Chain::Mainnet, "<your_api_key>")?;
// Or using environment variables
let client = Client::new_from_env(Chain::Mainnet)?;

let address = "0xBB9bc244D798123fDe783fCc1C72d3Bb8C189413".parse()?;
let metadata = client.contract_source_code(address).await?;
assert_eq!(metadata.items[0].contract_name, "DAO");
# Ok(())
# }
```
