use crate::{
    artifacts::Source,
    error::{Result, SolcError},
    utils, CompilerInput, CompilerOutput,
};
use semver::{Version, VersionReq};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    fmt,
    fmt::Formatter,
    io::BufRead,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    str::FromStr,
};

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

#[cfg(any(test, all(feature = "svm", feature = "async")))]
use once_cell::sync::Lazy;

#[cfg(any(test, feature = "tests"))]
use std::sync::Mutex;

#[cfg(any(test, feature = "tests"))]
#[allow(unused)]
static LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

#[cfg(all(feature = "svm", feature = "async"))]
/// A list of upstream Solc releases, used to check which version
/// we should download.
/// The boolean value marks whether there was an error.
pub static RELEASES: Lazy<(svm::Releases, Vec<Version>, bool)> = Lazy::new(|| {
    // Try to download the releases, if it fails default to empty
    match tokio::runtime::Runtime::new()
        .expect("could not create tokio rt to get remote releases")
        // we do not degrade startup performance if the consumer has a weak network?
        // use a 3 sec timeout for the request which should still be fine for slower connections
        .block_on(async {
            tokio::time::timeout(
                std::time::Duration::from_millis(3000),
                svm::all_releases(svm::platform()),
            )
            .await
        }) {
        Ok(Ok(releases)) => {
            let mut sorted_releases = releases.releases.keys().cloned().collect::<Vec<Version>>();
            sorted_releases.sort();
            (releases, sorted_releases, true)
        }
        Ok(Err(err)) => {
            tracing::error!("{:?}", err);
            (svm::Releases::default(), Vec::new(), false)
        }
        Err(err) => {
            tracing::error!("Releases request timed out: {:?}", err);
            (svm::Releases::default(), Vec::new(), false)
        }
    }
});

/// A `Solc` version is either installed (available locally) or can be downloaded, from the remote
/// endpoint
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
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
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
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
            .ok_or_else(|| SolcError::solc("svm home dir not found"))?
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
        // take the lock in tests, we use this to enforce that
        // a test does not run while a compiler version is being installed
        let _lock = LOCK.lock();

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
            _ => return Err(SolcError::VersionNotFound),
        })
    }

    /// Parses the given source looking for the `pragma` definition and
    /// returns the corresponding SemVer version requirement.
    pub fn source_version_req(source: &Source) -> Result<VersionReq> {
        let version =
            utils::find_version_pragma(&source.content).ok_or(SolcError::PragmaNotFound)?;
        Self::version_req(version)
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
        svm::install(version).await
    }

    /// Blocking version of `Self::install`
    #[cfg(all(feature = "svm", feature = "async"))]
    pub fn blocking_install(version: &Version) -> std::result::Result<(), svm::SolcVmError> {
        tracing::trace!("blocking installing solc version \"{}\"", version);
        tokio::runtime::Runtime::new().unwrap().block_on(svm::install(version))?;
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
            std::fs::read(&version_path).map_err(|err| SolcError::io(err, version_path))?;

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
            Err(SolcError::ChecksumMismatch)
        }
    }

    /// Convenience function for compiling all sources under the given path
    pub fn compile_source(&self, path: impl AsRef<Path>) -> Result<CompilerOutput> {
        let path = path.as_ref();
        self.compile(&CompilerInput::new(path)?)
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
            .map_err(|err| SolcError::io(err, &self.solc))?;
        let stdin = child.stdin.take().unwrap();

        serde_json::to_writer(stdin, input)?;
        compile_output(child.wait_with_output().map_err(|err| SolcError::io(err, &self.solc))?)
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
                .map_err(|err| SolcError::io(err, &self.solc))?,
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
            .map_err(|err| SolcError::io(err, &self.solc))?;
        let stdin = child.stdin.as_mut().unwrap();
        stdin.write_all(&content).await.map_err(|err| SolcError::io(err, &self.solc))?;
        stdin.flush().await.map_err(|err| SolcError::io(err, &self.solc))?;
        compile_output(
            child.wait_with_output().await.map_err(|err| SolcError::io(err, &self.solc))?,
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
                .map_err(|err| SolcError::io(err, &self.solc))?
                .wait_with_output()
                .await
                .map_err(|err| SolcError::io(err, &self.solc))?,
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
    pub async fn compile_many<I>(jobs: I, n: usize) -> CompiledMany
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
        CompiledMany { outputs }
    }
}

/// The result of a `solc` process bundled with its `Solc` and `CompilerInput`
type CompileElement = (Result<CompilerOutput>, Solc, CompilerInput);

/// The output of multiple `solc` processes.
#[derive(Debug)]
pub struct CompiledMany {
    outputs: Vec<CompileElement>,
}

impl CompiledMany {
    /// Returns an iterator over all output elements
    pub fn outputs(&self) -> impl Iterator<Item = &CompileElement> {
        self.outputs.iter()
    }

    /// Returns an iterator over all output elements
    pub fn into_outputs(self) -> impl Iterator<Item = CompileElement> {
        self.outputs.into_iter()
    }

    /// Returns all `CompilerOutput` or the first error that occurred
    pub fn flattened(self) -> Result<Vec<CompilerOutput>> {
        self.into_iter().collect()
    }
}

impl IntoIterator for CompiledMany {
    type Item = Result<CompilerOutput>;
    type IntoIter = std::vec::IntoIter<Result<CompilerOutput>>;

    fn into_iter(self) -> Self::IntoIter {
        self.outputs.into_iter().map(|(res, _, _)| res).collect::<Vec<_>>().into_iter()
    }
}

fn compile_output(output: Output) -> Result<Vec<u8>> {
    if output.status.success() {
        Ok(output.stdout)
    } else {
        Err(SolcError::solc(String::from_utf8_lossy(&output.stderr).to_string()))
    }
}

fn version_from_output(output: Output) -> Result<Version> {
    if output.status.success() {
        let version = output
            .stdout
            .lines()
            .last()
            .ok_or_else(|| SolcError::solc("version not found in solc output"))?
            .map_err(|err| SolcError::msg(format!("Failed to read output: {}", err)))?;
        // NOTE: semver doesn't like `+` in g++ in build metadata which is invalid semver
        Ok(Version::from_str(&version.trim_start_matches("Version: ").replace(".g++", ".gcc"))?)
    } else {
        Err(SolcError::solc(String::from_utf8_lossy(&output.stderr).to_string()))
    }
}

impl AsRef<Path> for Solc {
    fn as_ref(&self) -> &Path {
        &self.solc
    }
}

impl<T: Into<PathBuf>> From<T> for Solc {
    fn from(solc: T) -> Self {
        Solc::new(solc.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CompilerInput;

    fn solc() -> Solc {
        Solc::default()
    }

    #[test]
    fn solc_version_works() {
        solc().version().unwrap();
    }

    #[test]
    fn can_parse_version_metadata() {
        let _version = Version::from_str("0.6.6+commit.6c089d02.Linux.gcc").unwrap();
    }

    #[cfg(feature = "async")]
    #[tokio::test]
    async fn async_solc_version_works() {
        let _version = solc().async_version().await.unwrap();
    }

    #[test]
    fn solc_compile_works() {
        let input = include_str!("../test-data/in/compiler-in-1.json");
        let input: CompilerInput = serde_json::from_str(input).unwrap();
        let out = solc().compile(&input).unwrap();
        let other = solc().compile(&serde_json::json!(input)).unwrap();
        assert_eq!(out, other);
    }

    #[cfg(feature = "async")]
    #[tokio::test]
    async fn async_solc_compile_works() {
        let input = include_str!("../test-data/in/compiler-in-1.json");
        let input: CompilerInput = serde_json::from_str(input).unwrap();
        let out = solc().async_compile(&input).await.unwrap();
        let other = solc().async_compile(&serde_json::json!(input)).await.unwrap();
        assert_eq!(out, other);
    }
    #[cfg(feature = "async")]
    #[tokio::test]
    async fn async_solc_compile_works2() {
        let input = include_str!("../test-data/in/compiler-in-2.json");
        let input: CompilerInput = serde_json::from_str(input).unwrap();
        let out = solc().async_compile(&input).await.unwrap();
        let other = solc().async_compile(&serde_json::json!(input)).await.unwrap();
        assert_eq!(out, other);
        let sync_out = solc().compile(&input).unwrap();
        assert_eq!(out, sync_out);
    }

    #[test]
    fn test_version_req() {
        let versions = ["=0.1.2", "^0.5.6", ">=0.7.1", ">0.8.0"];
        let sources = versions.iter().map(|version| source(version));

        sources.zip(versions).for_each(|(source, version)| {
            let version_req = Solc::source_version_req(&source).unwrap();
            assert_eq!(version_req, VersionReq::from_str(version).unwrap());
        });

        // Solidity defines version ranges with a space, whereas the semver package
        // requires them to be separated with a comma
        let version_range = ">=0.8.0 <0.9.0";
        let source = source(version_range);
        let version_req = Solc::source_version_req(&source).unwrap();
        assert_eq!(version_req, VersionReq::from_str(">=0.8.0,<0.9.0").unwrap());
    }

    #[test]
    // This test might be a bit hard to maintain
    #[cfg(all(feature = "svm", feature = "async"))]
    fn test_detect_version() {
        for (pragma, expected) in [
            // pinned
            ("=0.4.14", "0.4.14"),
            // pinned too
            ("0.4.14", "0.4.14"),
            // The latest patch is 0.4.26
            ("^0.4.14", "0.4.26"),
            // latest version above 0.5.0 -> we have to
            // update this test whenever there's a new sol
            // version. that's ok! good reminder to check the
            // patch notes.
            (">=0.5.0", "0.8.11"),
            // range
            (">=0.4.0 <0.5.0", "0.4.26"),
        ]
        .iter()
        {
            let source = source(pragma);
            let res = Solc::detect_version(&source).unwrap();
            assert_eq!(res, Version::from_str(expected).unwrap());
        }
    }

    #[test]
    #[cfg(feature = "full")]
    fn test_find_installed_version_path() {
        // this test does not take the lock by default, so we need to manually
        // add it here.
        let _lock = LOCK.lock();
        let ver = "0.8.6";
        let version = Version::from_str(ver).unwrap();
        if utils::installed_versions(svm::SVM_HOME.as_path())
            .map(|versions| !versions.contains(&version))
            .unwrap_or_default()
        {
            Solc::blocking_install(&version).unwrap();
        }
        let res = Solc::find_svm_installed_version(&version.to_string()).unwrap().unwrap();
        let expected = svm::SVM_HOME.join(ver).join(format!("solc-{}", ver));
        assert_eq!(res.solc, expected);
    }

    #[test]
    fn does_not_find_not_installed_version() {
        let ver = "1.1.1";
        let version = Version::from_str(ver).unwrap();
        let res = Solc::find_svm_installed_version(&version.to_string()).unwrap();
        assert!(res.is_none());
    }

    #[test]
    fn test_find_latest_matching_installation() {
        let versions = ["0.4.24", "0.5.1", "0.5.2"]
            .iter()
            .map(|version| Version::from_str(version).unwrap())
            .collect::<Vec<_>>();

        let required = VersionReq::from_str(">=0.4.24").unwrap();

        let got = Solc::find_matching_installation(&versions, &required).unwrap();
        assert_eq!(got, versions[2]);
    }

    #[test]
    fn test_no_matching_installation() {
        let versions = ["0.4.24", "0.5.1", "0.5.2"]
            .iter()
            .map(|version| Version::from_str(version).unwrap())
            .collect::<Vec<_>>();

        let required = VersionReq::from_str(">=0.6.0").unwrap();
        let got = Solc::find_matching_installation(&versions, &required);
        assert!(got.is_none());
    }

    ///// helpers

    fn source(version: &str) -> Source {
        Source { content: format!("pragma solidity {};\n", version) }
    }
}
