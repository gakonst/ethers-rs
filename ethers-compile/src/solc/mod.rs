/// The name of the `solc` binary on the system
pub const SOLC: &str = "solc";

/// Support for configuring the EVM version
/// https://blog.soliditylang.org/2018/03/08/solidity-0.4.21-release-announcement/
pub const CONSTANTINOPLE_SOLC: Version = Version::new(0, 4, 21);

/// Petersburg support
/// https://blog.soliditylang.org/2019/03/05/solidity-0.5.5-release-announcement/
pub const PETERSBURG_SOLC: Version = Version::new(0, 5, 5);

/// Istanbul support
/// https://blog.soliditylang.org/2019/12/09/solidity-0.5.14-release-announcement/
pub const ISTANBUL_SOLC: Version = Version::new(0, 5, 14);

/// Berlin support
/// https://blog.soliditylang.org/2021/06/10/solidity-0.8.5-release-announcement/
pub const BERLIN_SOLC: Version = Version::new(0, 8, 5);

/// London support
/// https://blog.soliditylang.org/2021/08/11/solidity-0.8.7-release-announcement/
pub const LONDON_SOLC: Version = Version::new(0, 8, 7);

#[cfg(any(test, feature = "tests"))]
use std::sync::Mutex;

#[cfg(any(test, feature = "tests"))]
#[allow(unused)]
static LOCK: once_cell::sync::Lazy<Mutex<()>> = once_cell::sync::Lazy::new(|| Mutex::new(()));

/// take the lock in tests, we use this to enforce that
/// a test does not run while a compiler version is being installed
///
/// This ensures that only one thread installs a missing `solc` exe.
/// Instead of taking this lock in `Solc::blocking_install`, the lock should be taken before
/// installation is detected.
#[cfg(any(test, feature = "tests"))]
#[allow(unused)]
pub(crate) fn take_solc_installer_lock() -> std::sync::MutexGuard<'static, ()> {
    LOCK.lock().unwrap()
}

/// A list of upstream Solc releases, used to check which version
/// we should download.
/// The boolean value marks whether there was an error.
#[cfg(all(feature = "svm"))]
pub static RELEASES: once_cell::sync::Lazy<(svm::Releases, Vec<Version>, bool)> =
    once_cell::sync::Lazy::new(|| match svm::blocking_all_releases(svm::platform()) {
        Ok(releases) => {
            let sorted_versions = releases.clone().into_versions();
            (releases, sorted_versions, true)
        }
        Err(err) => {
            tracing::error!("{:?}", err);
            (svm::Releases::default(), Vec::new(), false)
        }
    });

/// A `Solc` version is either installed (available locally) or can be downloaded, from the remote
/// endpoint
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SolcVersion {
    Installed(Version),
    Remote(Version),
}

impl SolcVersion {
    /// Whether this version is installed
    pub fn is_installed(&self) -> bool {
        matches!(self, SolcVersion::Installed(_))
    }
}

impl AsRef<Version> for SolcVersion {
    fn as_ref(&self) -> &Version {
        match self {
            SolcVersion::Installed(v) | SolcVersion::Remote(v) => v,
        }
    }
}

impl From<SolcVersion> for Version {
    fn from(s: SolcVersion) -> Version {
        match s {
            SolcVersion::Installed(v) | SolcVersion::Remote(v) => v,
        }
    }
}

impl fmt::Display for SolcVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

/// Abstraction over `solc` command line utility
///
/// Supports sync and async functions.
///
/// By default the solc path is configured as follows, with descending priority:
///   1. `SOLC_PATH` environment variable
///   2. [svm](https://github.com/roynalnaruto/svm-rs)'s  `global_version` (set via `svm use <version>`), stored at `<svm_home>/.global_version`
///   3. `solc` otherwise
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Solc {
    /// Path to the `solc` executable
    pub solc: PathBuf,
    /// Additional arguments passed to the `solc` exectuable
    pub args: Vec<String>,
}

impl Default for Solc {
    fn default() -> Self {
        if let Ok(solc) = std::env::var("SOLC_PATH") {
            return Solc::new(solc)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(solc) = Solc::svm_global_version()
                .and_then(|vers| Solc::find_svm_installed_version(&vers.to_string()).ok())
                .flatten()
            {
                return solc
            }
        }

        Solc::new(SOLC)
    }
}

impl fmt::Display for Solc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.solc.display())?;
        if !self.args.is_empty() {
            write!(f, " {}", self.args.join(" "))?;
        }
        Ok(())
    }
}

impl Solc {
    /// A new instance which points to `solc`
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Solc { solc: path.into(), args: Vec::new() }
    }

    /// Adds an argument to pass to the `solc` command.
    #[must_use]
    pub fn arg<T: Into<String>>(mut self, arg: T) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Adds multiple arguments to pass to the `solc`.
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

    /// Returns the directory in which [svm](https://github.com/roynalnaruto/svm-rs) stores all versions
    ///
    /// This will be `~/.svm` on unix
    #[cfg(not(target_arch = "wasm32"))]
    pub fn svm_home() -> Option<PathBuf> {
        home::home_dir().map(|dir| dir.join(".svm"))
    }

    /// Returns the `semver::Version` [svm](https://github.com/roynalnaruto/svm-rs)'s `.global_version` is currently set to.
    ///  `global_version` is configured with (`svm use <version>`)
    ///
    /// This will read the version string (eg: "0.8.9") that the  `~/.svm/.global_version` file
    /// contains
    #[cfg(not(target_arch = "wasm32"))]
    pub fn svm_global_version() -> Option<Version> {
        let version =
            std::fs::read_to_string(Self::svm_home().map(|p| p.join(".global_version"))?).ok()?;
        Version::parse(&version).ok()
    }

    /// Returns the list of all solc instances installed at `SVM_HOME`
    #[cfg(not(target_arch = "wasm32"))]
    pub fn installed_versions() -> Vec<SolcVersion> {
        if let Some(home) = Self::svm_home() {
            utils::installed_versions(home)
                .unwrap_or_default()
                .into_iter()
                .map(SolcVersion::Installed)
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Returns the list of all versions that are available to download and marking those which are
    /// already installed.
    #[cfg(all(feature = "svm", feature = "async"))]
    pub fn all_versions() -> Vec<SolcVersion> {
        let mut all_versions = Self::installed_versions();
        let mut uniques = all_versions
            .iter()
            .map(|v| {
                let v = v.as_ref();
                (v.major, v.minor, v.patch)
            })
            .collect::<std::collections::HashSet<_>>();
        all_versions.extend(
            RELEASES
                .1
                .clone()
                .into_iter()
                .filter(|v| uniques.insert((v.major, v.minor, v.patch)))
                .map(SolcVersion::Remote),
        );
        all_versions.sort_unstable();
        all_versions
    }

    /// Returns the path for a [svm](https://github.com/roynalnaruto/svm-rs) installed version.
    ///
    /// # Example
    /// ```no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///  use ethers_solc::Solc;
    /// let solc = Solc::find_svm_installed_version("0.8.9").unwrap();
    /// assert_eq!(solc, Some(Solc::new("~/.svm/0.8.9/solc-0.8.9")));
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(not(target_arch = "wasm32"))]
    pub fn find_svm_installed_version(version: impl AsRef<str>) -> Result<Option<Self>> {
        let version = version.as_ref();
        let solc = Self::svm_home()
            .ok_or_else(|| CompilerError::solc("svm home dir not found"))?
            .join(version)
            .join(format!("solc-{}", version));

        if !solc.is_file() {
            return Ok(None)
        }
        Ok(Some(Solc::new(solc)))
    }

    /// Assuming the `versions` array is sorted, it returns the first element which satisfies
    /// the provided [`VersionReq`]
    pub fn find_matching_installation(
        versions: &[Version],
        required_version: &VersionReq,
    ) -> Option<Version> {
        // iterate in reverse to find the last match
        versions.iter().rev().find(|version| required_version.matches(version)).cloned()
    }

    /// Given a Solidity source, it detects the latest compiler version which can be used
    /// to build it, and returns it.
    ///
    /// If the required compiler version is not installed, it also proceeds to install it.
    #[cfg(all(feature = "svm", feature = "async"))]
    pub fn detect_version(source: &Source) -> Result<Version> {
        // detects the required solc version
        let sol_version = Self::source_version_req(source)?;
        Self::ensure_installed(&sol_version)
    }

    /// Given a Solidity version requirement, it detects the latest compiler version which can be
    /// used to build it, and returns it.
    ///
    /// If the required compiler version is not installed, it also proceeds to install it.
    #[cfg(all(feature = "svm", feature = "async"))]
    pub fn ensure_installed(sol_version: &VersionReq) -> Result<Version> {
        #[cfg(any(test, feature = "tests"))]
        let _lock = take_solc_installer_lock();

        // load the local / remote versions
        let versions = utils::installed_versions(svm::SVM_HOME.as_path()).unwrap_or_default();

        let local_versions = Self::find_matching_installation(&versions, sol_version);
        let remote_versions = Self::find_matching_installation(&RELEASES.1, sol_version);
        // if there's a better upstream version than the one we have, install it
        Ok(match (local_versions, remote_versions) {
            (Some(local), None) => local,
            (Some(local), Some(remote)) => {
                if remote > local {
                    Self::blocking_install(&remote)?;
                    remote
                } else {
                    local
                }
            }
            (None, Some(version)) => {
                Self::blocking_install(&version)?;
                version
            }
            // do nothing otherwise
            _ => return Err(CompilerError::VersionNotFound),
        })
    }

    /// Parses the given source looking for the `pragma` definition and
    /// returns the corresponding SemVer version requirement.
    pub fn source_version_req(source: &Source) -> Result<VersionReq> {
        let version =
            utils::find_version_pragma(&source.content).ok_or(CompilerError::PragmaNotFound)?;
        Self::version_req(version.as_str())
    }

    /// Returns the corresponding SemVer version requirement for the solidity version
    pub fn version_req(version: &str) -> Result<VersionReq> {
        let version = version.replace(' ', ",");

        // Somehow, Solidity semver without an operator is considered to be "exact",
        // but lack of operator automatically marks the operator as Caret, so we need
        // to manually patch it? :shrug:
        let exact = !matches!(&version[0..1], "*" | "^" | "=" | ">" | "<" | "~");
        let mut version = VersionReq::parse(&version)?;
        if exact {
            version.comparators[0].op = semver::Op::Exact;
        }

        Ok(version)
    }

    /// Installs the provided version of Solc in the machine under the svm dir
    /// # Example
    /// ```no_run
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    ///  use ethers_solc::{Solc, ISTANBUL_SOLC};
    ///  Solc::install(&ISTANBUL_SOLC).await.unwrap();
    ///  let solc = Solc::find_svm_installed_version(&ISTANBUL_SOLC.to_string());
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "svm")]
    pub async fn install(version: &Version) -> std::result::Result<(), svm::SolcVmError> {
        tracing::trace!("installing solc version \"{}\"", version);
        crate::report::solc_installation_start(version);
        let result = svm::install(version).await;
        crate::report::solc_installation_success(version);
        result
    }

    /// Blocking version of `Self::install`
    #[cfg(all(feature = "svm", feature = "async"))]
    pub fn blocking_install(version: &Version) -> std::result::Result<(), svm::SolcVmError> {
        tracing::trace!("blocking installing solc version \"{}\"", version);
        crate::report::solc_installation_start(version);
        svm::blocking_install(version)?;
        crate::report::solc_installation_success(version);
        Ok(())
    }

    /// Verify that the checksum for this version of solc is correct. We check against the SHA256
    /// checksum from the build information published by binaries.soliditylang
    #[cfg(all(feature = "svm", feature = "async"))]
    pub fn verify_checksum(&self) -> Result<()> {
        let version = self.version_short()?;
        let mut version_path = svm::version_path(version.to_string().as_str());
        version_path.push(format!("solc-{}", version.to_string().as_str()));
        let content =
            std::fs::read(&version_path).map_err(|err| CompilerError::io(err, version_path))?;

        if !RELEASES.2 {
            // we skip checksum verification because the underlying request to fetch release info
            // failed so we have nothing to compare against
            return Ok(())
        }

        use sha2::Digest;
        let mut hasher = sha2::Sha256::new();
        hasher.update(&content);
        let checksum_calc = &hasher.finalize()[..];

        let checksum_found = &RELEASES.0.get_checksum(&version).expect("checksum not found");

        if checksum_calc == checksum_found {
            Ok(())
        } else {
            Err(CompilerError::ChecksumMismatch)
        }
    }

    /// Convenience function for compiling all sources under the given path
    pub fn compile_source(&self, path: impl AsRef<Path>) -> Result<CompilerOutput> {
        let path = path.as_ref();
        self.compile(&CompilerInput::new(path)?)
    }

    /// Same as [`Self::compile()`], but only returns those files which are included in the
    /// `CompilerInput`.
    ///
    /// In other words, this removes those files from the `CompilerOutput` that are __not__ included
    /// in the provided `CompilerInput`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///  use ethers_solc::{CompilerInput, Solc};
    /// let solc = Solc::default();
    /// let input = CompilerInput::new("./contracts")?;
    /// let output = solc.compile_exact(&input)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn compile_exact(&self, input: &CompilerInput) -> Result<CompilerOutput> {
        let mut out = self.compile(input)?;
        out.retain_files(input.sources.keys().filter_map(|p| p.to_str()));
        Ok(out)
    }

    /// Run `solc --stand-json` and return the `solc`'s output as
    /// `CompilerOutput`
    ///
    /// # Example
    ///
    /// ```no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///  use ethers_solc::{CompilerInput, Solc};
    /// let solc = Solc::default();
    /// let input = CompilerInput::new("./contracts")?;
    /// let output = solc.compile(&input)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn compile<T: Serialize>(&self, input: &T) -> Result<CompilerOutput> {
        self.compile_as(input)
    }

    /// Run `solc --stand-json` and return the `solc`'s output as the given json
    /// output
    pub fn compile_as<T: Serialize, D: DeserializeOwned>(&self, input: &T) -> Result<D> {
        let output = self.compile_output(input)?;
        Ok(serde_json::from_slice(&output)?)
    }

    pub fn compile_output<T: Serialize>(&self, input: &T) -> Result<Vec<u8>> {
        let mut cmd = Command::new(&self.solc);
        let mut child = cmd
            .args(&self.args)
            .arg("--standard-json")
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|err| CompilerError::io(err, &self.solc))?;
        let stdin = child.stdin.take().expect("Stdin exists.");
        serde_json::to_writer(stdin, input)?;
        compile_output(child.wait_with_output().map_err(|err| CompilerError::io(err, &self.solc))?)
    }

    pub fn version_short(&self) -> Result<Version> {
        let version = self.version()?;
        Ok(Version::new(version.major, version.minor, version.patch))
    }

    /// Returns the version from the configured `solc`
    pub fn version(&self) -> Result<Version> {
        version_from_output(
            Command::new(&self.solc)
                .arg("--version")
                .stdin(Stdio::piped())
                .stderr(Stdio::piped())
                .stdout(Stdio::piped())
                .output()
                .map_err(|err| CompilerError::io(err, &self.solc))?,
        )
    }
}


#[cfg(feature = "async")]
impl Solc {
    /// Convenience function for compiling all sources under the given path
    pub async fn async_compile_source<T: Serialize>(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<CompilerOutput> {
        self.async_compile(&CompilerInput::with_sources(Source::async_read_all_from(path).await?))
            .await
    }

    /// Run `solc --stand-json` and return the `solc`'s output as
    /// `CompilerOutput`
    pub async fn async_compile<T: Serialize>(&self, input: &T) -> Result<CompilerOutput> {
        self.async_compile_as(input).await
    }

    /// Run `solc --stand-json` and return the `solc`'s output as the given json
    /// output
    pub async fn async_compile_as<T: Serialize, D: DeserializeOwned>(
        &self,
        input: &T,
    ) -> Result<D> {
        let output = self.async_compile_output(input).await?;
        Ok(serde_json::from_slice(&output)?)
    }

    pub async fn async_compile_output<T: Serialize>(&self, input: &T) -> Result<Vec<u8>> {
        use tokio::io::AsyncWriteExt;
        let content = serde_json::to_vec(input)?;
        let mut child = tokio::process::Command::new(&self.solc)
            .args(&self.args)
            .arg("--standard-json")
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|err| CompilerError::io(err, &self.solc))?;
        let stdin = child.stdin.as_mut().unwrap();
        stdin.write_all(&content).await.map_err(|err| CompilerError::io(err, &self.solc))?;
        stdin.flush().await.map_err(|err| CompilerError::io(err, &self.solc))?;
        compile_output(
            child.wait_with_output().await.map_err(|err| CompilerError::io(err, &self.solc))?,
        )
    }

    pub async fn async_version(&self) -> Result<Version> {
        version_from_output(
            tokio::process::Command::new(&self.solc)
                .arg("--version")
                .stdin(Stdio::piped())
                .stderr(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .map_err(|err| CompilerError::io(err, &self.solc))?
                .wait_with_output()
                .await
                .map_err(|err| CompilerError::io(err, &self.solc))?,
        )
    }

    /// Compiles all `CompilerInput`s with their associated `Solc`.
    ///
    /// This will buffer up to `n` `solc` processes and then return the `CompilerOutput`s in the
    /// order in which they complete. No more than `n` futures will be buffered at any point in
    /// time, and less than `n` may also be buffered depending on the state of each future.
    ///
    /// # Example
    ///
    /// Compile 2 `CompilerInput`s at once
    ///
    /// ```no_run
    /// # async fn example() {
    /// use ethers_solc::{CompilerInput, Solc};
    /// let solc1 = Solc::default();
    /// let solc2 = Solc::default();
    /// let input1 = CompilerInput::new("contracts").unwrap();
    /// let input2 = CompilerInput::new("src").unwrap();
    ///
    /// let outputs = Solc::compile_many([(solc1, input1), (solc2, input2)], 2).await.flattened().unwrap();
    /// # }
    /// ```
    pub async fn compile_many<I>(jobs: I, n: usize) -> crate::many::CompiledMany
    where
        I: IntoIterator<Item = (Solc, CompilerInput)>,
    {
        use futures_util::stream::StreamExt;

        let outputs = futures_util::stream::iter(
            jobs.into_iter()
                .map(|(solc, input)| async { (solc.async_compile(&input).await, solc, input) }),
        )
        .buffer_unordered(n)
        .collect::<Vec<_>>()
        .await;

        crate::many::CompiledMany::new(outputs)
    }
}
