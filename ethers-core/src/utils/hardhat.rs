use crate::{
    types::Address,
    utils::{secret_key_to_address, unused_port},
};
use k256::{ecdsa::SigningKey, SecretKey as K256SecretKey};
use std::{
    io::{BufRead, BufReader},
    process::{Child, Command},
    time::{Duration, Instant},
};

/// How long we will wait for hardhat to indicate that it is ready.
const HARDHAT_STARTUP_TIMEOUT_MILLIS: u64 = 10_000;

/// A hardhat instance. Will close the instance when dropped.
///
/// Construct this using [`Hardhat`](crate::utils::Hardhat)
pub struct HardhatInstance {
    pid: Child,
    private_keys: Vec<K256SecretKey>,
    addresses: Vec<Address>,
    port: u16,
}

impl HardhatInstance {
    /// Returns the private keys used to instantiate this instance
    pub fn keys(&self) -> &[K256SecretKey] {
        &self.private_keys
    }

    /// Returns the addresses used to instantiate this instance
    pub fn addresses(&self) -> &[Address] {
        &self.addresses
    }

    /// Returns the port of this instance
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Returns the HTTP endpoint of this instance
    pub fn endpoint(&self) -> String {
        format!("http://localhost:{}", self.port)
    }

    /// Returns the Websocket endpoint of this instance
    pub fn ws_endpoint(&self) -> String {
        format!("ws://localhost:{}", self.port)
    }
}

impl Drop for HardhatInstance {
    fn drop(&mut self) {
        let _ = self.pid.kill().expect("could not kill hardhat");
    }
}

/// Builder for launching `hardhat`.
///
/// # Panics
///
/// If `spawn` is called without `hardhat` being available in the user's $PATH
///
/// # Example
///
/// ```no_run
/// use ethers::utils::Hardhat;
///
/// let port = 8545u16;
/// let url = format!("http://localhost:{}", port).to_string();
///
/// let hardhat = Hardhat::new()
///     .port(port)
///     .spawn();
///
/// drop(hardhat); // this will kill the instance
/// ```
#[derive(Clone, Default)]
pub struct Hardhat {
    port: Option<u16>,
    fork: Option<String>,
    args: Vec<String>,
}

impl Hardhat {
    /// Creates an empty Hardhat builder.
    /// The default port is 8545. The mnemonic is chosen randomly.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the port which will be used when the `hardhat` instance is launched.
    pub fn port<T: Into<u16>>(mut self, port: T) -> Self {
        self.port = Some(port.into());
        self
    }

    /// Sets the `fork` argument to fork from another currently running Ethereum client
    /// at a given block. Input should be the HTTP location and port of the other client,
    /// e.g. `http://localhost:8545`. You can optionally specify the block to fork from
    /// using an @ sign: `http://localhost:8545@1599200`
    pub fn fork<T: Into<String>>(mut self, fork: T) -> Self {
        self.fork = Some(fork.into());
        self
    }

    /// Adds an argument to pass to the `hardhat`.
    pub fn arg<T: Into<String>>(mut self, arg: T) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Adds multiple arguments to pass to the `hardhat`.
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for arg in args {
            self = self.arg(arg);
        }
        self
    }
    // node /home/grw/src/wildcredit-keeper/node_modules/.bin/hardhat node
    /// Consumes the builder and spawns `hardhat` with stdout redirected
    /// to /dev/null. This takes ~2 seconds to execute as it blocks while
    /// waiting for `hardhat` to launch.
    pub fn spawn(self) -> HardhatInstance {
        let mut cmd = Command::new("hardhat");
        cmd.args(&["node"]);
        cmd.stdout(std::process::Stdio::piped());
        let port = if let Some(port) = self.port {
            port
        } else {
            unused_port()
        };
        cmd.arg("--port").arg(port.to_string());

        if let Some(fork) = self.fork {
            cmd.arg("--fork").arg(fork);
        }

        cmd.args(self.args);

        let mut child = cmd.spawn().expect("couldnt start hardhat");

        let stdout = child
            .stdout
            .expect("Unable to get stdout for hardhat child process");

        let start = Instant::now();
        let mut reader = BufReader::new(stdout);

        let mut private_keys = Vec::new();
        let mut addresses = Vec::new();

        loop {
            if addresses.len() == 20 {
                break;
            }

            if start + Duration::from_millis(HARDHAT_STARTUP_TIMEOUT_MILLIS) <= Instant::now() {
                panic!("Timed out waiting for hardhat to start. Is hardhat installed?")
            }

            let mut line = String::new();
            reader
                .read_line(&mut line)
                .expect("Failed to read line from hardhat process");

            if line.starts_with("Private Key:") {
                let key_str = &line[15..line.len() - 1];
                let key_hex = hex::decode(key_str).expect("could not parse as hex");
                let key = K256SecretKey::from_bytes(&key_hex).expect("did not get private key");
                addresses.push(secret_key_to_address(&SigningKey::from(&key)));
                private_keys.push(key);
            }
        }

        child.stdout = Some(reader.into_inner());

        HardhatInstance {
            pid: child,
            private_keys,
            addresses,
            port,
        }
    }
}
