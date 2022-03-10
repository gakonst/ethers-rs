# ethers-compile

A refactored compiling framework.

First add `ethers-compile` to your cargo build-dependencies.

[Explain how to implement/use]

```toml
[build-dependencies]
ethers-compile = { git = "https://github.com/gakonst/ethers-rs" }
```

```rust
use ethers_compile::{Project, ProjectPathsConfig};
fn main() {
    // configure the project with all its paths, solc, cache etc.
    let project = Project::builder()
        .paths(ProjectPathsConfig::hardhat(env!("CARGO_MANIFEST_DIR")).unwrap())
        .build()
        .unwrap();
    let output = project.compile().unwrap();
    
    // Tell Cargo that if a source file changes, to rerun this build script.
    project.rerun_if_sources_changed();
}
```