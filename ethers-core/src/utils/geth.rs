use super::{unused_port, Genesis};
use std::{
    env::temp_dir,
    fs::File,
    io::{BufRead, BufReader, Read},
    path::PathBuf,
    process::{Child, ChildStderr, Command},
    time::{Duration, Instant},
};

/// How long we will wait for geth to indicate that it is ready.
const GETH_STARTUP_TIMEOUT_MILLIS: u64 = 10_000;

/// The exposed APIs
const API: &str = "eth,net,web3,txpool,admin";

/// The geth command
const GETH: &str = "geth";

/// A geth instance. Will close the instance when dropped.
///
/// Construct this using [`Geth`](crate::utils::Geth)
pub struct GethInstance {
    pid: Child,
    port: u16,
    ipc: Option<PathBuf>,
    data_dir: Option<PathBuf>,
    p2p_port: Option<u16>,
}

impl GethInstance {
    /// Returns the port of this instance
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Returns the p2p port of this instance
    pub fn p2p_port(&self) -> Option<u16> {
        self.p2p_port
    }

    /// Returns the HTTP endpoint of this instance
    pub fn endpoint(&self) -> String {
        format!("http://localhost:{}", self.port)
    }

    /// Returns the Websocket endpoint of this instance
    pub fn ws_endpoint(&self) -> String {
        format!("ws://localhost:{}", self.port)
    }

    /// Returns the path to this instances' IPC socket
    pub fn ipc_path(&self) -> &Option<PathBuf> {
        &self.ipc
    }

    /// Returns the path to this instances' data directory
    pub fn data_dir(&self) -> &Option<PathBuf> {
        &self.data_dir
    }

    /// Return a `BufReader` for the stderr output of this instance
    pub fn stderr(&mut self) -> BufReader<ChildStderr> {
        BufReader::new(self.pid.stderr.take().unwrap())
    }
}

impl Drop for GethInstance {
    fn drop(&mut self) {
        self.pid.kill().expect("could not kill geth");
    }
}

/// Whether or not geth is in `dev` mode and configuration options that depend on the mode.
#[derive(Debug, Clone)]
pub enum GethMode {
    /// Options that can be set in dev mode
    Dev(DevOptions),
    /// Options that cannot be set in dev mode
    NonDev(PrivateNetOptions),
}

impl Default for GethMode {
    fn default() -> Self {
        Self::Dev(Default::default())
    }
}

/// Configuration options that can be set in dev mode.
#[derive(Debug, Clone, Default)]
pub struct DevOptions {
    /// The interval at which the dev chain will mine new blocks.
    pub block_time: Option<u64>,
}

/// Configuration options that cannot be set in dev mode.
#[derive(Debug, Clone)]
pub struct PrivateNetOptions {
    /// The p2p port to use.
    pub p2p_port: Option<u16>,

    /// Whether or not peer discovery is enabled.
    pub discovery: bool,
}

impl Default for PrivateNetOptions {
    fn default() -> Self {
        Self { p2p_port: None, discovery: true }
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
    ipc_path: Option<PathBuf>,
    data_dir: Option<PathBuf>,
    chain_id: Option<u64>,
    genesis: Option<Genesis>,
    mode: GethMode,
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

    /// Sets the port which will be used for incoming p2p connections.
    ///
    /// This will put the geth instance into non-dev mode, discarding any previously set dev-mode
    /// options.
    #[must_use]
    pub fn p2p_port(mut self, port: u16) -> Self {
        match self.mode {
            GethMode::Dev(_) => {
                self.mode = GethMode::NonDev(PrivateNetOptions {
                    p2p_port: Some(port),
                    ..Default::default()
                })
            }
            GethMode::NonDev(ref mut opts) => opts.p2p_port = Some(port),
        }
        self
    }

    /// Sets the block-time which will be used when the `geth-cli` instance is launched.
    ///
    /// This will put the geth instance in `dev` mode, discarding any previously set options that
    /// cannot be used in dev mode.
    #[must_use]
    pub fn block_time<T: Into<u64>>(mut self, block_time: T) -> Self {
        self.mode = GethMode::Dev(DevOptions { block_time: Some(block_time.into()) });
        self
    }

    /// Sets the chain id for the geth instance.
    #[must_use]
    pub fn chain_id<T: Into<u64>>(mut self, chain_id: T) -> Self {
        self.chain_id = Some(chain_id.into());
        self
    }

    /// Disable discovery for the geth instance.
    ///
    /// This will put the geth instance into non-dev mode, discarding any previously set dev-mode
    /// options.
    #[must_use]
    pub fn disable_discovery(mut self) -> Self {
        match self.mode {
            GethMode::Dev(_) => {
                self.mode =
                    GethMode::NonDev(PrivateNetOptions { discovery: false, ..Default::default() })
            }
            GethMode::NonDev(ref mut opts) => opts.discovery = false,
        }
        self
    }

    /// Manually sets the IPC path for the socket manually.
    #[must_use]
    pub fn ipc_path<T: Into<PathBuf>>(mut self, path: T) -> Self {
        self.ipc_path = Some(path.into());
        self
    }

    /// Sets the data directory for geth.
    #[must_use]
    pub fn data_dir<T: Into<PathBuf>>(mut self, path: T) -> Self {
        self.data_dir = Some(path.into());
        self
    }

    /// Sets the `genesis.json` for the geth instance.
    ///
    /// If this is set, geth will be initialized with `geth init` and the `--datadir` option will be
    /// set to the same value as `data_dir`.
    ///
    /// This is destructive and will overwrite any existing data in the data directory.
    #[must_use]
    pub fn genesis(mut self, genesis: Genesis) -> Self {
        self.genesis = Some(genesis);
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

        // use geth init to initialize the datadir if the genesis exists
        if let Some(genesis) = self.genesis {
            // create a temp dir to store the genesis file
            let temp_genesis_path = temp_dir().join("genesis.json");

            // create the genesis file
            let mut file = File::create(&temp_genesis_path).expect("could not create genesis file");

            // serialize genesis and write to file
            serde_json::to_writer_pretty(&mut file, &genesis)
                .expect("could not write genesis to file");

            let mut init_cmd = Command::new(GETH);
            if let Some(ref data_dir) = self.data_dir {
                init_cmd.arg("--datadir").arg(data_dir);
            }

            init_cmd.arg("init").arg(temp_genesis_path);
            init_cmd
                .spawn()
                .expect("failed to spawn geth init")
                .wait()
                .expect("failed to wait for geth init");
        }

        if let Some(ref data_dir) = self.data_dir {
            cmd.arg("--datadir").arg(data_dir);
        }

        // Dev mode with custom block time
        match self.mode {
            GethMode::Dev(DevOptions { block_time }) => {
                cmd.arg("--dev");
                if let Some(block_time) = block_time {
                    cmd.arg("--dev.period").arg(block_time.to_string());
                }
            }
            GethMode::NonDev(PrivateNetOptions { p2p_port, .. }) => {
                if let Some(p2p_port) = p2p_port {
                    cmd.arg("--port").arg(p2p_port.to_string());
                }
            }
        }

        if let Some(chain_id) = self.chain_id {
            cmd.arg("--networkid").arg(chain_id.to_string());
        }

        // disable discovery if the flag is set
        if let GethMode::NonDev(PrivateNetOptions { discovery, .. }) = self.mode {
            if !discovery {
                cmd.arg("--nodiscover");
            }
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
            match self.mode {
                GethMode::Dev(_) => {
                    if line.contains("HTTP endpoint opened") || line.contains("HTTP server started")
                    {
                        break
                    }
                }
                GethMode::NonDev(_) => {
                    if line.contains("Started P2P networking") {
                        break
                    }
                }
            }
        }

        child.stderr = Some(reader.into_inner());

        let p2p_port = match self.mode {
            GethMode::Dev(_) => None,
            GethMode::NonDev(PrivateNetOptions { p2p_port, .. }) => p2p_port,
        };

        GethInstance { pid: child, port, ipc: self.ipc_path, data_dir: self.data_dir, p2p_port }
    }
}
