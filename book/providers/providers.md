# Providers

A Provider is an abstraction of a connection to the Ethereum network, providing a concise, consistent interface to standard Ethereum node functionality.

This is achieved through the [`Middleware` trait][middleware], which provides the interface for the [Ethereum JSON-RPC API](https://ethereum.github.io/execution-apis/api-documentation) and other helpful methods, explained in more detail in [the Middleware chapter](../middleware/middleware.md), and the [`Provider`][provider] struct, which implements `Middleware`.

## Data transports

A [`Provider`][provider] wraps a generic data transport `P`, through which all JSON-RPC API calls are routed.

Ethers provides concrete transport implementations for [HTTP](./http.md), [WebSockets](./ws.md), and [IPC](./ipc.md), as well as higher level transports which wrap a single or multiple transports. Of course, it is also possible to [define custom data transports](./custom.md).

Transports implement the [`JsonRpcClient`](https://docs.rs/ethers/latest/ethers/providers/trait.JsonRpcClient.html) trait, which defines a `request` method, used for sending data to the underlying Ethereum node using [JSON-RPC](https://www.jsonrpc.org/specification).

Transports can optionally implement the [`PubsubClient`](https://docs.rs/ethers/latest/ethers/providers/trait.PubsubClient.html) trait, if they support the [Publish-subscribe pattern](https://en.wikipedia.org/wiki/Publish%E2%80%93subscribe_pattern), like `Websockets` and `IPC`. This is a [supertrait](https://doc.rust-lang.org/book/ch19-03-advanced-traits.html#using-supertraits-to-require-one-traits-functionality-within-another-trait) of `JsonRpcClient`. It defines the `subscribe` and `unsubscribe` methods.

## The Provider type

This is the definition of the [`Provider`][provider] type:

```rust
#[derive(Clone, Debug)]
pub struct Provider<P> {
    inner: P,
    ens: Option<Address>,
    interval: Option<Duration>,
    from: Option<Address>,
    node_client: Arc<Mutex<Option<NodeClient>>>,
}
```

-   `inner`: stores the generic data transport, which sends the requests;
-   `ens`: optional override for the default ENS registry address;
-   `interval`: optional value that defines the polling interval for `watch_*` streams;
-   `from`: optional address that sets a default `from` address when constructing calls and transactions;
-   `node_client`: the type of node the provider is connected to, like Geth, Erigon, etc.

Now that you have a basis for what the `Provider` type actually is, the next few sections will walk through each implementation of the `Provider`, starting with the HTTP provider.

[middleware]: https://docs.rs/ethers/latest/ethers/providers/trait.Middleware.html
[provider]: https://docs.rs/ethers/latest/ethers/providers/struct.Provider.html
