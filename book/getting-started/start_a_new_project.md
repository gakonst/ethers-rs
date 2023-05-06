# Start a new project

To set up a new project with ethers-rs, you will need to install the Rust programming language toolchain and the Cargo package manager on your system.

1. Install Rust by following the instructions at [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install).
2. Once Rust is installed, create a new Rust project by running the following command:

    ```bash
    cargo new my-project
    ```

    This will create a new directory called my-project with the necessary files for a new Rust project.

3. Navigate to the project directory and add ethers-rs as a dependency in your `Cargo.toml` file:

    ```toml
    [dependencies]
    ethers = "2.0"
    # Ethers' async features rely upon the Tokio async runtime.
    tokio = { version = "1", features = ["macros"] }
    # Flexible concrete Error Reporting type built on std::error::Error with customizable Reports
    eyre = "0.6"
    ```

    If you want to make experiments and/or play around with early ethers-rs features link our GitHub repo in the `Cargo.toml`.

    ```toml
    [dependencies]
    ethers = { git = "https://github.com/gakonst/ethers-rs" }

    # Use the "branch" attribute to specify a branch other than master
    [dependencies]
    ethers = { git = "https://github.com/gakonst/ethers-rs", branch = "branch-name" }

    # You can specify a tag or commit hash with the "rev" attribute
    [dependencies]
    ethers = { git = "https://github.com/gakonst/ethers-rs", rev = "84dda78" }
    ```

    > **Note:** using a Git repository as a dependency is generally not recommended
    > for production projects, as it can make it difficult to ensure that you are using
    > a specific and stable version of the dependency.
    > It is usually better to specify a version number or range to ensure that your project
    > is reproducible.

## Enable transports

Ethers-rs enables interactions with Ethereum nodes through different "transport" types, or communication protocols.
The following transport types are currently supported by ethers.rs:

-   **HTTP(S):** The HTTP(S) transport is used to communicate with Ethereum nodes over the HTTP or HTTPS protocols. This is the most common way to interact with Ethereum nodes. If you are looking to connect to a HTTPS endpoint, then you need to enable the `rustls` or `openssl` features:

    ```toml
    [dependencies]
    ethers = { version = "2.0", features = ["rustls"] }
    ```

-   **WebSocket:** The WebSocket transport is used to communicate with Ethereum nodes over the WebSocket protocol, which is a widely-supported standard for establishing a bi-directional communication channel between a client and a server. This can be used for a variety of purposes, including receiving real-time updates from an Ethereum node, or submitting transactions to the Ethereum network. Websockets support is turned on via the feature-flag ws:

    ```toml
    [dependencies]
    ethers = { version = "2.0", features = ["ws"] }
    ```

-   **IPC (Interprocess Communication):** The IPC transport is used to communicate with a local Ethereum node using the IPC protocol, which is a way for processes to communicate with each other on a single computer. This is commonly used in Ethereum development to allow applications to communicate with a local Ethereum node, such as geth or parity. IPC support is turned on via the feature-flag `ipc`:

    ```toml
    [dependencies]
    ethers = { version = "2.0", features = ["ipc"] }
    ```
