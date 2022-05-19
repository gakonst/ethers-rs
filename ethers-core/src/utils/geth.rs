use super::unused_port;
use std::{
    io::{BufRead, BufReader},
    path::PathBuf,
    process::{Child, Command},
    time::{Duration, Instant},
};

/// How long we will wait for geth to indicate that it is ready.
const GETH_STARTUP_TIMEOUT_MILLIS: u64 = 10_000;

/// The exposed APIs
const API: &str = "eth,net,web3,txpool";

/// The geth command
const GETH: &str = "geth";

/// A geth instance. Will close the instance when dropped.
///
/// Construct this using [`Geth`](crate::utils::Geth)
pub struct GethInstance {
    pid: Child,
    port: u16,
    ipc: Option<PathBuf>,
}

impl GethInstance {
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

    pub fn ipc_path(&self) -> &Option<PathBuf> {
        &self.ipc
    }
}

impl Drop for GethInstance {
    fn drop(&mut self) {
        self.pid.kill().expect("could not kill geth");
    }
}

/// Builder for launching `geth`.
///
/// # Panics
///
/// If `spawn` is called without `geth` being available in the user's $PATH
///
/// # Example
///
/// ```no_run
/// use ethers_core::utils::Geth;
///
/// let port = 8545u16;
/// let url = format!("http://localhost:{}", port).to_string();
///
/// let geth = Geth::new()
///     .port(port)
///     .block_time(5000u64)
///     .spawn();
///
/// drop(geth); // this will kill the instance
/// ```
#[derive(Clone, Default)]
pub struct Geth {
    port: Option<u16>,
    block_time: Option<u64>,
    ipc_path: Option<PathBuf>,
}

impl Geth {
    /// Creates an empty Geth builder.
    /// The default port is 8545. The mnemonic is chosen randomly.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the port which will be used when the `geth-cli` instance is launched.
    #[must_use]
    pub fn port<T: Into<u16>>(mut self, port: T) -> Self {
        self.port = Some(port.into());
        self
    }

    /// Sets the block-time which will be used when the `geth-cli` instance is launched.
    #[must_use]
    pub fn block_time<T: Into<u64>>(mut self, block_time: T) -> Self {
        self.block_time = Some(block_time.into());
        self
    }

    /// Manually sets the IPC path for the socket manually.
    #[must_use]
    pub fn ipc_path<T: Into<PathBuf>>(mut self, path: T) -> Self {
        self.ipc_path = Some(path.into());
        self
    }

    /// Consumes the builder and spawns `geth` with stdout redirected
    /// to /dev/null.
    pub fn spawn(self) -> GethInstance {
        let mut cmd = Command::new(GETH);
        // geth uses stderr for its logs
        cmd.stderr(std::process::Stdio::piped());
        let port = if let Some(port) = self.port { port } else { unused_port() };

        // Open the HTTP API
        cmd.arg("--http");
        cmd.arg("--http.port").arg(port.to_string());
        cmd.arg("--http.api").arg(API);

        // Open the WS API
        cmd.arg("--ws");
        cmd.arg("--ws.port").arg(port.to_string());
        cmd.arg("--ws.api").arg(API);

        // Dev mode with custom block time
        cmd.arg("--dev");
        if let Some(block_time) = self.block_time {
            cmd.arg("--dev.period").arg(block_time.to_string());
        }

        if let Some(ref ipc) = self.ipc_path {
            cmd.arg("--ipcpath").arg(ipc);
        }

        let mut child = cmd.spawn().expect("couldnt start geth");

        let stdout = child.stderr.expect("Unable to get stderr for geth child process");

        let start = Instant::now();
        let mut reader = BufReader::new(stdout);

        loop {
            if start + Duration::from_millis(GETH_STARTUP_TIMEOUT_MILLIS) <= Instant::now() {
                panic!("Timed out waiting for geth to start. Is geth installed?")
            }

            let mut line = String::new();
            reader.read_line(&mut line).expect("Failed to read line from geth process");

            // geth 1.9.23 uses "server started" while 1.9.18 uses "endpoint opened"
            if line.contains("HTTP endpoint opened") || line.contains("HTTP server started") {
                break
            }
        }

        child.stderr = Some(reader.into_inner());

        GethInstance { pid: child, port, ipc: self.ipc_path }
    }
}
