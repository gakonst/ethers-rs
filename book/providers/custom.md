# Custom data transport

As [we've previously seen](./providers.md#data-transports), a transport must implement [`JsonRpcClient`](https://docs.rs/ethers/latest/ethers/providers/trait.JsonRpcClient.html), and can also optionally implement [`PubsubClient`](https://docs.rs/ethers/latest/ethers/providers/trait.PubsubClient.html).

Let's see how we can create a custom data transport by implementing one that stores either a `Ws` or an `Ipc` transport:

```rust
{{#include ../../examples/providers/examples/custom.rs}}
```
