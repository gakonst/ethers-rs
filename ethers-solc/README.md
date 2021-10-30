# ethers-solc

Utilities for working with native `solc` and compiling projects.

To also compile contracts during `cargo build` (so that ethers `abigen!` can pull in updated abi automatically) you can configure a `ethers_solc::Project` in your `build.rs` file

First add `ethers-solc` to your cargo build-dependencies

```toml
[build-dependencies]
ethers-solc = { git = "https://github.com/gakonst/ethers-rs" }
```

```rust
use ethers_solc::{Project, ProjectPathsConfig};

fn main() {
    // configure the project with all its paths, solc, cache etc.
    let project = Project::builder()
        .paths(ProjectPathsConfig::hardhat(env!("CARGO_MANIFEST_DIR")).unwrap())
        .build()
        .unwrap();
    let output = project.compile().unwrap();
    println!("{}", output);
}
```