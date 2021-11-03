use crate::{
    artifacts::Source,
    error::{Result, SolcError},
    CompilerInput, CompilerOutput,
};
use semver::{Version, VersionReq};
use serde::{de::DeserializeOwned, Serialize};
use std::{
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
static LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

#[cfg(all(feature = "svm", feature = "async"))]
/// A list of upstream Solc releases, used to check which version
/// we should download.
pub static RELEASES: Lazy<Vec<Version>> = Lazy::new(|| {
    // Try to download the releases, if it fails default to empty
    match tokio::runtime::Runtime::new()
        .expect("could not create tokio rt to get remote releases")
        // TODO: Can we make this future timeout at a small time amount so that
        // we do not degrade startup performance if the consumer has a weak network?
        .block_on(svm::all_versions())
    {
        Ok(inner) => inner,
        Err(_) => Vec::new(),
    }
});

/// Abstraction over `solc` command line utility
///
/// Supports sync and async functions.
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct Solc(pub PathBuf);

impl Default for Solc {
    fn default() -> Self {
        std::env::var("SOLC_PATH").map(Solc::new).unwrap_or_else(|_| Solc::new(SOLC))
    }
}

impl Solc {
    /// A new instance which points to `solc`
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Solc(path.into())
    }

    /// Returns the directory in which [svm](https://github.com/roynalnaruto/svm-rs) stores all versions
    ///
    /// This will be `~/.svm` on unix
    #[cfg(not(target_arch = "wasm32"))]
    pub fn svm_home() -> Option<PathBuf> {
        home::home_dir().map(|dir| dir.join(".svm"))
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
        let solc = walkdir::WalkDir::new(
            Self::svm_home().ok_or_else(|| SolcError::solc("svm home dir not found"))?,
        )
        .max_depth(1)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.file_type().is_dir())
        .find(|e| e.path().ends_with(version))
        .map(|e| e.path().join(format!("solc-{}", version)))
        .map(Solc::new);
        Ok(solc)
    }

    /// Assuming the `versions` array is sorted, it returns the latest element which satisfies
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
        let sol_version = Self::version_req(source)?;

        #[cfg(any(test, feature = "tests"))]
        // take the lock in tests, we use this to enforce that
        // a test does not run while a compiler version is being installed
        let _lock = LOCK.lock();

        // load the local / remote versions
        let versions = svm::installed_versions().unwrap_or_default();
        let local_versions = Self::find_matching_installation(&versions, &sol_version);
        let remote_versions = Self::find_matching_installation(&RELEASES, &sol_version);

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
    pub fn version_req(source: &Source) -> Result<VersionReq> {
        let version = crate::utils::find_version_pragma(&source.content)
            .ok_or(SolcError::PragmaNotFound)?
            .replace(" ", ",");

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
        svm::install(version).await
    }

    /// Blocking version of `Self::install`
    #[cfg(all(feature = "svm", feature = "async"))]
    pub fn blocking_install(version: &Version) -> std::result::Result<(), svm::SolcVmError> {
        tokio::runtime::Runtime::new().unwrap().block_on(svm::install(version))?;
        Ok(())
    }

    /// Convenience function for compiling all sources under the given path
    pub fn compile_source(&self, path: impl AsRef<Path>) -> Result<CompilerOutput> {
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
        let mut child = Command::new(&self.0)
            .arg("--standard-json")
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
        let stdin = child.stdin.take().unwrap();

        serde_json::to_writer(stdin, input)?;
        compile_output(child.wait_with_output()?)
    }

    /// Returns the version from the configured `solc`
    pub fn version(&self) -> Result<Version> {
        version_from_output(
            Command::new(&self.0)
                .arg("--version")
                .stdin(Stdio::piped())
                .stderr(Stdio::piped())
                .stdout(Stdio::piped())
                .output()?,
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
        let mut child = tokio::process::Command::new(&self.0)
            .arg("--standard-json")
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
        let stdin = child.stdin.as_mut().unwrap();
        stdin.write(&content).await?;
        stdin.flush().await?;
        compile_output(child.wait_with_output().await?)
    }

    pub async fn async_version(&self) -> Result<Version> {
        version_from_output(
            tokio::process::Command::new(&self.0)
                .arg("--version")
                .stdin(Stdio::piped())
                .stderr(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()?
                .wait_with_output()
                .await?,
        )
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
            .ok_or_else(|| SolcError::solc("version not found in solc output"))??;
        // NOTE: semver doesn't like `+` in g++ in build metadata which is invalid semver
        Ok(Version::from_str(&version.trim_start_matches("Version: ").replace(".g++", ".gcc"))?)
    } else {
        Err(SolcError::solc(String::from_utf8_lossy(&output.stderr).to_string()))
    }
}

impl AsRef<Path> for Solc {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl<T: Into<PathBuf>> From<T> for Solc {
    fn from(solc: T) -> Self {
        Solc(solc.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CompilerInput;

    fn solc() -> Solc {
        std::env::var("SOLC_PATH").map(Solc::new).unwrap_or_default()
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

    #[test]
    fn test_version_req() {
        let versions = ["=0.1.2", "^0.5.6", ">=0.7.1", ">0.8.0"];
        let sources = versions.iter().map(|version| source(version));

        sources.zip(versions).for_each(|(source, version)| {
            let version_req = Solc::version_req(&source).unwrap();
            assert_eq!(version_req, VersionReq::from_str(version).unwrap());
        });

        // Solidity defines version ranges with a space, whereas the semver package
        // requires them to be separated with a comma
        let version_range = ">=0.8.0 <0.9.0";
        let source = source(version_range);
        let version_req = Solc::version_req(&source).unwrap();
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
            (">=0.5.0", "0.8.9"),
            // range
            (">=0.4.0 <0.5.0", "0.4.26"),
        ]
        .iter()
        {
            // println!("Checking {}", pragma);
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
        if !svm::installed_versions().unwrap().contains(&version) {
            Solc::blocking_install(&version).unwrap();
        }
        let res = Solc::find_svm_installed_version(&version.to_string()).unwrap().unwrap();
        let expected = svm::SVM_HOME.join(ver).join(format!("solc-{}", ver));
        assert_eq!(res.0, expected);
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
