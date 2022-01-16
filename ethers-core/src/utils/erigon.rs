use crate::utils::unused_port;
use ethabi::ethereum_types::Address;
use k256::SecretKey as K256SecretKey;
use std::{
    env::temp_dir,
    io::{BufRead, BufReader},
    process::{Child, Command},
    time::{Duration, Instant},
};

/// How long we will wait for erigon to indicate that it is ready
const ERIGON_STARTUP_TIMEOUT_MILLIS: u64 = 30_000;

/// The exposed APIs
const API: &str = "eth,erigon,net,debug,trace,txpool";

/// The erigon command
const ERIGON: &str = "erigon";
const ERIGON_DAEMON: &str = "rpcdaemon";

/// A erigon instance, Will close the instance with daemon when dropped
///
/// Construct this using [`Erigon`](crate::utils::Erigon)
pub struct ErigonInstance {
    pid: Child,
    rpc_daemon: Child,
    port: u16,
    address: Address,
    key: K256SecretKey,
}

impl ErigonInstance {
    /// Returns private key of the developer account
    pub fn key(&self) -> &K256SecretKey {
        &self.key
    }

    /// Returns the address of the developer account
    pub fn address(&self) -> Address {
        self.address
    }

    /// Returns the port that erigon is listening on
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Returns the Websocket endpoint of this instance
    pub fn endpoint(&self) -> String {
        format!("http://localhost:{}", self.port)
    }

    /// Returns the Websocket endpoint of this instance
    pub fn ws_endpoint(&self) -> String {
        format!("ws://localhost:{}", self.port)
    }
}

impl Drop for ErigonInstance {
    fn drop(&mut self) {
        self.rpc_daemon.kill().expect("Failed to kill ws daemon");
        self.pid.kill().expect("Failed to kill erigon");
    }
}

/// Builder for launching a `erigon`.
///
/// # Panics
///
/// If `spawn` is called without `erigon` with `rpcdaemon` being available in the user's $PATH.
///
/// # Example
///
/// ```no_run
/// use ethers::utils::Erigon;
///
/// let erigon = Erigon::new().block_time(2u64).spawn();
/// let provider = Provider::new(Ws::connect(erigon.ws_endpoint()).await.unwrap());
/// ```
#[derive(Clone, Default)]
pub struct Erigon {
    port: Option<u16>,
    block_time: Option<u64>,
}

impl Erigon {
    /// Creates an empty Erigon builder.
    /// The default port is 8545.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the port which will be used when the `erigon` instance is launched.
    #[must_use]
    pub fn port<T: Into<u16>>(mut self, port: T) -> Self {
        self.port = Some(port.into());
        self
    }

    /// Sets the block-time which will be used when the `erigon` instance is launched.
    #[must_use]
    pub fn block_time<T: Into<u64>>(mut self, block_time: T) -> Self {
        self.block_time = Some(block_time.into());
        self
    }

    /// Consumes the builder and spawns `erigon` with stdout redirected
    /// to /dev/null.
    pub fn spawn(self) -> ErigonInstance {
        let tempdir = temp_dir();
        let grpc_endpoint = format!("127.0.0.1:{}", unused_port());
        let port = unused_port();

        let key: K256SecretKey = K256SecretKey::from_be_bytes(
            hex::decode("26e86e45f6fc45ec6e2ecd128cec80fa1d1505e5507dcd2ae58c3130a7a97b48")
                .unwrap()
                .as_slice(),
        )
        .unwrap();
        let address = "0x67b1d87101671b127f5f8714789C7192f7ad340e".parse::<Address>().unwrap();

        let mut cmd = Command::new(ERIGON);
        cmd.stderr(std::process::Stdio::piped());

        // Specify database with grpc endpoint.
        cmd.arg("--datadir").arg(tempdir.to_str().unwrap());
        cmd.arg("--private.api.addr").arg(grpc_endpoint.as_str());

        // Specify dev chain properties.
        cmd.arg("--port").arg(unused_port().to_string());
        cmd.arg("--networkid").arg("1337");
        cmd.arg("--chain").arg("dev");

        let block_time = self.block_time.unwrap_or(1);
        cmd.arg("--dev.period").arg(block_time.to_string());

        let mut rpc_daemon = Command::new(ERIGON_DAEMON);
        rpc_daemon.stderr(std::process::Stdio::piped());

        // Open HTTP with WS endpoints.
        rpc_daemon.arg("--ws").arg(port.to_string().as_str());
        rpc_daemon.arg("--http.port").arg(port.to_string().as_str());
        rpc_daemon.arg("--http.api").arg(API);

        // Specify same path to the database so it can be shared.
        rpc_daemon.arg("--datadir").arg(temp_dir().to_str().unwrap());

        // Specify the grpc endpoint.
        rpc_daemon.arg("--txpool.api.addr").arg(grpc_endpoint.as_str());
        rpc_daemon.arg("--private.api.addr").arg(grpc_endpoint.as_str());

        let mut pid = cmd.spawn().expect("Failed to spawn erigon");

        let stderr = pid.stderr.expect("Unable to get stderr for erigon child process");
        let mut reader = BufReader::new(stderr);

        let start = Instant::now();
        loop {
            if start + Duration::from_millis(ERIGON_STARTUP_TIMEOUT_MILLIS) <= Instant::now() {
                panic!("Timed out waiting for erigon to start. Is erigon installed?")
            }

            let mut line = String::new();
            reader.read_line(&mut line).expect("Failed to read line from erigon process");
            if line.contains("Starting private RPC server") {
                break
            }
        }
        pid.stderr = Some(reader.into_inner());

        let mut rpc_daemon = rpc_daemon.spawn().expect("Failed to spawn ws daemon");
        let stderr = rpc_daemon.stderr.expect("Unable to get stderr for ws daemon child process");
        let mut reader = BufReader::new(stderr);

        let start = Instant::now();
        loop {
            if start + Duration::from_millis(ERIGON_STARTUP_TIMEOUT_MILLIS) <= Instant::now() {
                panic!("Timed out waiting for ws daemon to start. Is erigon installed?")
            }

            let mut line = String::new();
            reader.read_line(&mut line).expect("Failed to read line from ws daemon process");
            if line.contains("HTTP endpoint opened") {
                break
            }
        }
        rpc_daemon.stderr = Some(reader.into_inner());

        ErigonInstance { pid, rpc_daemon, port, key, address }
    }
}
