# Providers

Providers play a central role in `ethers-rs`, enabling you to establish asynchronous [Ethereum JSON-RPC](https://github.com/ethereum/wiki/wiki/JSON-RPC) compliant clients.

 your program to a node to get data, interact with smart contracts, listen to the mempool and much more. There are a few different types of default providers that are built into the library. The default providers are `Http`,`WS`,`Ipc`,`RWClient`,`Quorum`,`Mock` and `RetryClient`. In addition to all of these options, you can also create your own custom provider, which we will walk through later in this chapter. For now let take a look at what the `Provider` actually looks like.


```rust
#[derive(Clone, Debug)]
pub struct Provider<P> {
    inner: P,
    ens: Option<Address>,
    interval: Option<Duration>,
    from: Option<Address>,
    /// Node client hasn't been checked yet= `None`
    /// Unsupported node client = `Some(None)`
    /// Supported node client = `Some(Some(NodeClient))`
    _node_client: Arc<Mutex<Option<NodeClient>>>,
}
```


The `Provider` struct defines a generic type `P` that can be any type that implements the [`JsonRpcClient` trait](https://docs.rs/ethers/latest/ethers/providers/trait.JsonRpcClient.html). The `inner` field stores the type that implements the `JsonRpcClient` type, allowing the Provider to make RPC calls to the node. The `ens` field is an optional value that specifies the ENS address for the provider's default sender. The `interval` field is an optional value that defines the polling interval when for streams (subscribing to logs, block headers, etc.). The `from` field is an optional type that allows you to set a default "from" address when constructing transactions and making calls. Lastly, the `_node_client` field is another optional value that allows the user to specify the node they are using to access node specific API calls. 


Note that all providers implement the [`Middleware` trait](https://docs.rs/ethers/latest/ethers/providers/trait.Middleware.html), which gives every provider access to [commonly used methods](https://docs.rs/ethers/latest/ethers/providers/struct.Provider.html#impl-Middleware-for-Provider%3CP%3E) to interact with the node. Later in this chapter, we will go over these methods and examples for how to use them in detail. Additionally, `Middleware` will be covered extensively in a later chapter.

Now that you have a basis for what the `Provider` type actually is, the next few sections will walk through each implementation of the `Provider`, starting with the HTTP provider.

