use std::process::{Child, Command};

/// A ganache CLI instance. Will close the instance when dropped.
pub struct Ganache(Child);

impl Drop for Ganache {
    fn drop(&mut self) {
        self.0.kill().expect("could not kill ganache");
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
/// ```rust,ignore
/// use ethers_utils::ganache::GanacheBuilder;
/// let port = 8545u64;
/// let url = format!("http://localhost:{}", port).to_string();
///
/// let ganache = GanacheBuilder::new()
///     .port(port)
///     .mnemonic("abstract vacuum mammal awkward pudding scene penalty purchase dinner depart evoke puzzle")
///     .spawn();
///
/// drop(ganache); // this will kill the instance
/// ```
#[derive(Clone, Default)]
pub struct GanacheBuilder {
    port: Option<u64>,
    mnemonic: Option<String>,
}

impl GanacheBuilder {
    /// Creates an empty Ganache builder.
    /// The default port is 8545. The mnemonic is chosen randomly.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the port which will be used when the `ganache-cli` instance is launched.
    pub fn port<T: Into<u64>>(mut self, port: T) -> Self {
        self.port = Some(port.into());
        self
    }

    /// Sets the mnemonic which will be used when the `ganache-cli` instance is launched.
    pub fn mnemonic<T: Into<String>>(mut self, mnemonic: T) -> Self {
        self.mnemonic = Some(mnemonic.into());
        self
    }

    /// Consumes the builder and spawns `ganache-cli` with stdout redirected
    /// to /dev/null. This takes ~2 seconds to execute as it blocks while
    /// waiting for `ganache-cli` to launch.
    pub fn spawn(self) -> Ganache {
        let mut cmd = Command::new("ganache-cli");
        cmd.stdout(std::process::Stdio::null());
        if let Some(port) = self.port {
            cmd.arg("-p").arg(port.to_string());
        }

        if let Some(mnemonic) = self.mnemonic {
            cmd.arg("-m").arg(mnemonic);
        }

        let ganache_pid = cmd.spawn().expect("couldnt start ganache-cli");

        // wait a couple of seconds for ganache to boot up
        // TODO: Change this to poll for `port`
        let sleep_time = std::time::Duration::from_secs(2);
        std::thread::sleep(sleep_time);
        Ganache(ganache_pid)
    }
}
