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

/// How long we will wait for ganache to indicate that it is ready.
const GANACHE_STARTUP_TIMEOUT_MILLIS: u64 = 10_000;

/// A ganache CLI instance. Will close the instance when dropped.
///
/// Construct this using [`Ganache`](crate::utils::Ganache)
pub struct GanacheInstance {
    pid: Child,
    private_keys: Vec<K256SecretKey>,
    addresses: Vec<Address>,
    port: u16,
}

impl GanacheInstance {
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

impl Drop for GanacheInstance {
    fn drop(&mut self) {
        let _ = self.pid.kill().expect("could not kill ganache");
    }
}

/// Builder for launching `ganache-cli`.
///
/// # Panics
///
/// If `spawn` is called without `ganache-cli` being available in the user's $PATH
///
/// # Example
///
/// ```no_run
/// use ethers_core::utils::Ganache;
///
/// let port = 8545u16;
/// let url = format!("http://localhost:{}", port).to_string();
///
/// let ganache = Ganache::new()
///     .port(port)
///     .mnemonic("abstract vacuum mammal awkward pudding scene penalty purchase dinner depart evoke puzzle")
///     .spawn();
///
/// drop(ganache); // this will kill the instance
/// ```
#[derive(Clone, Default)]
pub struct Ganache {
    port: Option<u16>,
    block_time: Option<u64>,
    mnemonic: Option<String>,
    fork: Option<String>,
    args: Vec<String>,
}

impl Ganache {
    /// Creates an empty Ganache builder.
    /// The default port is 8545. The mnemonic is chosen randomly.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the port which will be used when the `ganache-cli` instance is launched.
    #[must_use]
    pub fn port<T: Into<u16>>(mut self, port: T) -> Self {
        self.port = Some(port.into());
        self
    }

    /// Sets the mnemonic which will be used when the `ganache-cli` instance is launched.
    #[must_use]
    pub fn mnemonic<T: Into<String>>(mut self, mnemonic: T) -> Self {
        self.mnemonic = Some(mnemonic.into());
        self
    }

    /// Sets the block-time which will be used when the `ganache-cli` instance is launched.
    #[must_use]
    pub fn block_time<T: Into<u64>>(mut self, block_time: T) -> Self {
        self.block_time = Some(block_time.into());
        self
    }

    /// Sets the `fork` argument to fork from another currently running Ethereum client
    /// at a given block. Input should be the HTTP location and port of the other client,
    /// e.g. `http://localhost:8545`. You can optionally specify the block to fork from
    /// using an @ sign: `http://localhost:8545@1599200`
    #[must_use]
    pub fn fork<T: Into<String>>(mut self, fork: T) -> Self {
        self.fork = Some(fork.into());
        self
    }

    /// Adds an argument to pass to the `ganache-cli`.
    #[must_use]
    pub fn arg<T: Into<String>>(mut self, arg: T) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Adds multiple arguments to pass to the `ganache-cli`.
    #[must_use]
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

    /// Consumes the builder and spawns `ganache-cli` with stdout redirected
    /// to /dev/null. This takes ~2 seconds to execute as it blocks while
    /// waiting for `ganache-cli` to launch.
    pub fn spawn(self) -> GanacheInstance {
        let mut cmd = Command::new("ganache-cli");
        cmd.stdout(std::process::Stdio::piped());
        let port = if let Some(port) = self.port { port } else { unused_port() };
        cmd.arg("-p").arg(port.to_string());

        if let Some(mnemonic) = self.mnemonic {
            cmd.arg("-m").arg(mnemonic);
        }

        if let Some(block_time) = self.block_time {
            cmd.arg("-b").arg(block_time.to_string());
        }

        if let Some(fork) = self.fork {
            cmd.arg("-f").arg(fork);
        }

        cmd.args(self.args);

        let mut child = cmd.spawn().expect("couldnt start ganache-cli");

        let stdout = child.stdout.expect("Unable to get stdout for ganache child process");

        let start = Instant::now();
        let mut reader = BufReader::new(stdout);

        let mut private_keys = Vec::new();
        let mut addresses = Vec::new();
        let mut is_private_key = false;
        loop {
            if start + Duration::from_millis(GANACHE_STARTUP_TIMEOUT_MILLIS) <= Instant::now() {
                panic!("Timed out waiting for ganache to start. Is ganache-cli installed?")
            }

            let mut line = String::new();
            reader.read_line(&mut line).expect("Failed to read line from ganache process");
            if line.contains("Listening on") {
                break
            }

            if line.starts_with("Private Keys") {
                is_private_key = true;
            }

            if is_private_key && line.starts_with('(') {
                let key_str = &line[6..line.len() - 1];
                let key_hex = hex::decode(key_str).expect("could not parse as hex");
                let key = K256SecretKey::from_be_bytes(&key_hex).expect("did not get private key");
                addresses.push(secret_key_to_address(&SigningKey::from(&key)));
                private_keys.push(key);
            }
        }

        child.stdout = Some(reader.into_inner());

        GanacheInstance { pid: child, private_keys, addresses, port }
    }
}
