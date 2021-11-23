# Clients for interacting with Ethereum nodes

This crate provides asynchronous
[Ethereum JSON-RPC](https://github.com/ethereum/wiki/wiki/JSON-RPC) compliant
clients.

For more documentation on the available calls, refer to the
[`Provider`](./struct.Provider.html) struct.

# Examples

```no_run
use ethers_providers::{Provider, Http, Middleware};
use std::convert::TryFrom;

# async fn foo() -> Result<(), Box<dyn std::error::Error>> {
let provider = Provider::<Http>::try_from(
    "https://mainnet.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27"
)?;

let block = provider.get_block(100u64).await?;
println!("Got block: {}", serde_json::to_string(&block)?);

let code = provider.get_code("0x89d24a6b4ccb1b6faa2625fe562bdd9a23260359", None).await?;
println!("Got code: {}", serde_json::to_string(&code)?);
# Ok(())
# }
```

# Websockets

The crate has support for WebSockets via Tokio.

```
# async fn foo() -> Result<(), Box<dyn std::error::Error>> {
# use ethers_providers::Ws;
let ws = Ws::connect("ws://localhost:8545").await?;
# Ok(())
# }
```

# Ethereum Name Service

The provider may also be used to resolve
[Ethereum Name Service](https://ens.domains) (ENS) names to addresses (and vice
versa). The default ENS address is
[mainnet](https://etherscan.io/address/0x00000000000C2E074eC69A0dFb2997BA6C7d2e1e)
and can be overriden by calling the [`ens`](./struct.Provider.html#method.ens)
method on the provider.

```no_run
# use ethers_providers::{Provider, Http, Middleware};
# use std::convert::TryFrom;
# async fn foo() -> Result<(), Box<dyn std::error::Error>> {
# let provider = Provider::<Http>::try_from(
#     "https://mainnet.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27"
# )?;
// Resolve ENS name to Address
let name = "vitalik.eth";
let address = provider.resolve_name(name).await?;

// Lookup ENS name given Address
let resolved_name = provider.lookup_address(address).await?;
assert_eq!(name, resolved_name);
# Ok(())
# }
```
