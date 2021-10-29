use crate::{
    error::{Result, SolcError},
    CompilerInput, CompilerOutput,
};
use semver::Version;
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

/// Abstraction over `solc` command line utility
///
/// Supports sync and async functions.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Solc(pub PathBuf);

impl Default for Solc {
    fn default() -> Self {
        Self::new(SOLC)
    }
}

impl Solc {
    /// A new instance which points to `solc`
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Solc(path.into())
    }

    /// Convenience function for compiling all sources under the given path
    pub fn compile_source<T: Serialize>(&self, path: impl AsRef<Path>) -> Result<CompilerOutput> {
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
        use crate::artifacts::Source;
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
}
