use super::{unused_port, Genesis};
use crate::types::H256;
use std::{
    env::temp_dir,
    fs::{create_dir, File},
    io::{BufRead, BufReader},
    path::PathBuf,
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

/// How long we will wait for geth to indicate that it is ready.
const GETH_STARTUP_TIMEOUT_MILLIS: u64 = 10_000;

/// Timeout for waiting for geth to add a peer.
const GETH_DIAL_LOOP_TIMEOUT: Duration = Duration::new(20, 0);

/// The exposed APIs
const API: &str = "eth,net,web3,txpool,admin";

/// The geth command
const GETH: &str = "geth";

/// Errors that can occur when working with the [`GethInstance`].
#[derive(Debug)]
pub enum GethInstanceError {
    /// Timed out waiting for a message from geth's stderr.
    Timeout(String),

    /// A line could not be read from the geth stderr.
    ReadLineError(std::io::Error),

    /// The child geth process's stderr was not captured.
    NoStderr,
}

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

    /// Blocks until geth adds the specified peer, using 20s as the timeout.
    pub fn wait_to_add_peer(&mut self, id: H256) -> Result<(), GethInstanceError> {
        let mut stderr = self.pid.stderr.as_mut().ok_or(GethInstanceError::NoStderr)?;
        let mut err_reader = BufReader::new(&mut stderr);
        let mut line = String::new();
        let start = Instant::now();

        while start.elapsed() < GETH_DIAL_LOOP_TIMEOUT {
            line.clear();
            err_reader.read_line(&mut line).map_err(GethInstanceError::ReadLineError)?;

            // geth ids are trunated
            let truncated_id = hex::encode(&id.0[..8]);
            if line.contains("Adding p2p peer") && line.contains(&truncated_id) {
                return Ok(())
            }
        }
        Err(GethInstanceError::Timeout("Timed out waiting for geth to add a peer".into()))
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
    authrpc_port: Option<u16>,
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

    /// Sets the port for authenticated RPC connections.
    #[must_use]
    pub fn authrpc_port(mut self, port: u16) -> Self {
        self.authrpc_port = Some(port);
        self
    }

    /// Consumes the builder and spawns `geth` with stdout redirected
    /// to /dev/null.
    pub fn spawn(self) -> GethInstance {
        let mut cmd = Command::new(GETH);
        // geth uses stderr for its logs
        cmd.stderr(std::process::Stdio::piped());
        let port = if let Some(port) = self.port { port } else { unused_port() };
        let authrpc_port = if let Some(port) = self.authrpc_port { port } else { unused_port() };

        // Open the HTTP API
        cmd.arg("--http");
        cmd.arg("--http.port").arg(port.to_string());
        cmd.arg("--http.api").arg(API);

        // Open the WS API
        cmd.arg("--ws");
        cmd.arg("--ws.port").arg(port.to_string());
        cmd.arg("--ws.api").arg(API);

        // Set the port for authenticated APIs
        cmd.arg("--authrpc.port").arg(authrpc_port.to_string());

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

            // set the stderr to null so we don't pollute the test output
            init_cmd.stderr(Stdio::null());

            init_cmd.arg("init").arg(temp_genesis_path);
            init_cmd
                .spawn()
                .expect("failed to spawn geth init")
                .wait()
                .expect("failed to wait for geth init to exit");
        }

        if let Some(ref data_dir) = self.data_dir {
            cmd.arg("--datadir").arg(data_dir);

            // create the directory if it doesn't exist
            if !data_dir.exists() {
                create_dir(data_dir).expect("could not create data dir");
            }
        }

        // Dev mode with custom block time
        match self.mode {
            GethMode::Dev(DevOptions { block_time }) => {
                cmd.arg("--dev");
                if let Some(block_time) = block_time {
                    cmd.arg("--dev.period").arg(block_time.to_string());
                }
            }
            GethMode::NonDev(PrivateNetOptions { p2p_port, discovery }) => {
                // automatically enable and set the p2p port if we are in non-dev mode
                let port = if let Some(port) = p2p_port { port } else { unused_port() };
                cmd.arg("--port").arg(port.to_string());

                // disable discovery if the flag is set
                if !discovery {
                    cmd.arg("--nodiscover");
                }
            }
        }

        if let Some(chain_id) = self.chain_id {
            cmd.arg("--networkid").arg(chain_id.to_string());
        }

        // debug verbosity is needed to check when peers are added
        cmd.arg("--verbosity").arg("4");

        if let Some(ref ipc) = self.ipc_path {
            cmd.arg("--ipcpath").arg(ipc);
        }

        let mut child = cmd.spawn().expect("couldnt start geth");

        let stderr = child.stderr.expect("Unable to get stderr for geth child process");

        let start = Instant::now();
        let mut reader = BufReader::new(stderr);

        // we shouldn't need to wait for p2p to start if geth is in dev mode - p2p is disabled in
        // dev mode
        let mut p2p_started = matches!(self.mode, GethMode::Dev(_));
        let mut http_started = false;

        loop {
            if start + Duration::from_millis(GETH_STARTUP_TIMEOUT_MILLIS) <= Instant::now() {
                panic!("Timed out waiting for geth to start. Is geth installed?")
            }

            let mut line = String::new();
            reader.read_line(&mut line).expect("Failed to read line from geth process");

            if matches!(self.mode, GethMode::NonDev(_)) && line.contains("Started P2P networking") {
                p2p_started = true;
            }

            // geth 1.9.23 uses "server started" while 1.9.18 uses "endpoint opened"
            // the unauthenticated api is used for regular non-engine API requests
            if line.contains("HTTP endpoint opened") ||
                (line.contains("HTTP server started") && !line.contains("auth=true"))
            {
                http_started = true;
            }

            if p2p_started && http_started {
                break
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
