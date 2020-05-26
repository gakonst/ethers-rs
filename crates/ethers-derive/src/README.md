# `ethcontract-generate`

An alternative API for generating type-safe contract bindings from `build.rs`
scripts. Using this method instead of the procedural macro has a couple
advantages:
- Proper integration with with RLS and Racer for autocomplete support
- Ability to inspect the generated code

The downside of using the generator API is the requirement of having a build
script instead of a macro invocation.

## Getting Started

Using crate requires two dependencies - one for the runtime and one for the
generator:

```toml
[dependencies]
ethcontract = { version = "...", default-features = false }

[build-dependencies]
ethcontract-generate = "..."
```

It is recommended that both versions be kept in sync or else unexpected
behaviour may occur.

Then, in your `build.rs` include the following code:

```rs
use ethcontract_generate::Builder;
use std::env;
use std::path::Path;

fn main() {
    let dest = env::var("OUT_DIR").unwrap();
    Builder::new("path/to/truffle/build/contract/Contract.json")
        .generate()
        .unwrap()
        .write_to_file(Path::new(&dest).join("rust_coin.rs"))
        .unwrap();
}
```

## Relation to `ethcontract-derive`

`ethcontract-derive` uses `ethcontract-generate` under the hood so their
generated bindings should be identical, they just provide different APIs to the
same functionality.

The long term goal of this project is to maintain `ethcontract-derive`. For now
there is no extra work in having it split into two separate crates. That being
said if RLS support improves for procedural macro generated code, it is possible
that this crate be deprecated in favour of `ethcontract-derive` as long as there
is no good argument to keep it around.
