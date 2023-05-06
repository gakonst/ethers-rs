# ethers-providers

Clients for interacting with Ethereum nodes.

This crate provides asynchronous
[Ethereum JSON-RPC](https://github.com/ethereum/wiki/wiki/JSON-RPC) compliant
clients.

For more information, please refer to the [book](https://gakonst.com/ethers-rs).

## Websockets

This crate supports for WebSockets via `tokio-tungstenite`.
Please ensure that you have the `ws` feature enabled if you wish to use WebSockets:

```toml
[dependencies]
ethers-providers = { version = "2.0", features = ["ws"] }
```

## Interprocess Communication (IPC)

This crate supports for Interprocess Communication via Unix sockets and Windows named pipes.
Please ensure that you have the `ipc` feature enabled if you wish to use IPC:

```toml
[dependencies]
ethers-providers = { version = "2.0", features = ["ipc"] }
```

## Ethereum Name Service

The provider may also be used to resolve [Ethereum Name Service](https://ens.domains) (ENS) names
to addresses (and vice versa).
The default ENS address is [`0x00000000000C2E074eC69A0dFb2997BA6C7d2e1e`][ens]
and can be overriden with the [`ens`](./struct.Provider.html#method.ens) method on the provider.

[ens]: https://etherscan.io/address/0x00000000000C2E074eC69A0dFb2997BA6C7d2e1e

## Examples

```rust,no_run
# use ethers_core::types::Address;
# use ethers_providers::{Provider, Http, Middleware, Ws};
# async fn foo() -> Result<(), Box<dyn std::error::Error>> {
let provider = Provider::<Http>::try_from("https://eth.llamarpc.com")?;

let block = provider.get_block(100u64).await?;
println!("Got block: {}", serde_json::to_string(&block)?);

let addr = "0x89d24a6b4ccb1b6faa2625fe562bdd9a23260359".parse::<Address>()?;
let code = provider.get_code(addr, None).await?;
println!("Got code: {}", serde_json::to_string(&code)?);
# Ok(())
# }
```

Using ENS:

```rust,no_run
# use ethers_providers::{Provider, Http, Middleware};
# async fn foo() -> Result<(), Box<dyn std::error::Error>> {
let provider = Provider::<Http>::try_from("https://eth.llamarpc.com")?;

// Resolve ENS name to Address
let name = "vitalik.eth";
let address = provider.resolve_name(name).await?;

// Lookup ENS name given Address
let resolved_name = provider.lookup_address(address).await?;
assert_eq!(name, resolved_name);

/// Lookup ENS field
let url = "https://vitalik.ca".to_string();
let resolved_url = provider.resolve_field(name, "url").await?;
assert_eq!(url, resolved_url);

/// Lookup and resolve ENS avatar
let avatar = "https://ipfs.io/ipfs/QmSP4nq9fnN9dAiCj42ug9Wa79rqmQerZXZch82VqpiH7U/image.gif".to_string();
let resolved_avatar = provider.resolve_avatar(name).await?;
assert_eq!(avatar, resolved_avatar.to_string());
# Ok(())
# }
```
