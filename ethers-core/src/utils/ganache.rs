use crate::types::PrivateKey;
use std::{
    io::{BufRead, BufReader},
    net::TcpListener,
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
    private_keys: Vec<PrivateKey>,
    port: u16,
}

impl GanacheInstance {
    /// Returns the private keys used to instantiate this instance
    pub fn keys(&self) -> &[PrivateKey] {
        &self.private_keys
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
/// use ethers::utils::Ganache;
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
}

impl Ganache {
    /// Creates an empty Ganache builder.
    /// The default port is 8545. The mnemonic is chosen randomly.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the port which will be used when the `ganache-cli` instance is launched.
    pub fn port<T: Into<u16>>(mut self, port: T) -> Self {
        self.port = Some(port.into());
        self
    }

    /// Sets the mnemonic which will be used when the `ganache-cli` instance is launched.
    pub fn mnemonic<T: Into<String>>(mut self, mnemonic: T) -> Self {
        self.mnemonic = Some(mnemonic.into());
        self
    }

    /// Sets the block-time which will be used when the `ganache-cli` instance is launched.
    pub fn block_time<T: Into<u64>>(mut self, block_time: T) -> Self {
        self.block_time = Some(block_time.into());
        self
    }

    /// Consumes the builder and spawns `ganache-cli` with stdout redirected
    /// to /dev/null. This takes ~2 seconds to execute as it blocks while
    /// waiting for `ganache-cli` to launch.
    pub fn spawn(self) -> GanacheInstance {
        let mut cmd = Command::new("ganache-cli");
        cmd.stdout(std::process::Stdio::piped());
        let port = if let Some(port) = self.port {
            port
        } else {
            unused_port()
        };
        cmd.arg("-p").arg(port.to_string());

        if let Some(mnemonic) = self.mnemonic {
            cmd.arg("-m").arg(mnemonic);
        }

        if let Some(block_time) = self.block_time {
            cmd.arg("-b").arg(block_time.to_string());
        }

        let mut child = cmd.spawn().expect("couldnt start ganache-cli");

        let stdout = child
            .stdout
            .expect("Unable to get stdout for ganache child process");

        let start = Instant::now();
        let mut reader = BufReader::new(stdout);

        let mut private_keys = Vec::new();
        let mut is_private_key = false;
        loop {
            if start + Duration::from_millis(GANACHE_STARTUP_TIMEOUT_MILLIS) <= Instant::now() {
                panic!("Timed out waiting for ganache to start. Is ganache-cli installed?")
            }

            let mut line = String::new();
            reader
                .read_line(&mut line)
                .expect("Failed to read line from ganache process");
            if line.starts_with("Listening on") {
                break;
            }

            if line.starts_with("Private Keys") {
                is_private_key = true;
            }

            if is_private_key && line.starts_with('(') {
                let key_str = &line[6..line.len() - 1];
                let key: PrivateKey = key_str.parse().expect("did not get private key");
                private_keys.push(key);
            }
        }

        child.stdout = Some(reader.into_inner());

        GanacheInstance {
            pid: child,
            private_keys,
            port,
        }
    }
}

/// A bit of hack to find an unused TCP port.
///
/// Does not guarantee that the given port is unused after the function exists, just that it was
/// unused before the function started (i.e., it does not reserve a port).
pub fn unused_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0")
        .expect("Failed to create TCP listener to find unused port");

    let local_addr = listener
        .local_addr()
        .expect("Failed to read TCP listener local_addr to find unused port");
    local_addr.port()
}
