# WebSocket provider

The crate has support for WebSockets via Tokio. Please ensure that you have the “ws” and “rustls” / “openssl” features enabled if you wish to use WebSockets.

```rust
{{#include ../../examples/providers/examples/ws.rs}}
```