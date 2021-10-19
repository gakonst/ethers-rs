use crate::CompilerOutput;
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

    /// Run `solc --stand-json` and return the `solc`'s output as
    /// `CompilerOutput`
    pub fn compile<T: Serialize>(&self, input: &T) -> eyre::Result<CompilerOutput> {
        self.compile_as(input)
    }

    /// Run `solc --stand-json` and return the `solc`'s output as the given json
    /// output
    pub fn compile_as<T: Serialize, D: DeserializeOwned>(&self, input: &T) -> eyre::Result<D> {
        let output = self.compile_output(input)?;
        Ok(serde_json::from_slice(&output)?)
    }

    pub fn compile_output<T: Serialize>(&self, input: &T) -> eyre::Result<Vec<u8>> {
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
    pub fn version(&self) -> eyre::Result<Version> {
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
    /// Run `solc --stand-json` and return the `solc`'s output as
    /// `CompilerOutput`
    pub async fn async_compile<T: Serialize>(&self, input: &T) -> eyre::Result<CompilerOutput> {
        self.async_compile_as(input).await
    }

    /// Run `solc --stand-json` and return the `solc`'s output as the given json
    /// output
    pub async fn async_compile_as<T: Serialize, D: DeserializeOwned>(
        &self,
        input: &T,
    ) -> eyre::Result<D> {
        let output = self.async_compile_output(input).await?;
        Ok(serde_json::from_slice(&output)?)
    }

    pub async fn async_compile_output<T: Serialize>(&self, input: &T) -> eyre::Result<Vec<u8>> {
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

    pub async fn async_version(&self) -> eyre::Result<Version> {
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

fn compile_output(output: Output) -> eyre::Result<Vec<u8>> {
    if output.status.success() {
        Ok(output.stdout)
    } else {
        let err = String::from_utf8_lossy(&output.stderr).to_string();
        eyre::bail!(err)
    }
}

fn version_from_output(output: Output) -> eyre::Result<Version> {
    if output.status.success() {
        let version = output
            .stdout
            .lines()
            .last()
            .ok_or_else(|| eyre::eyre!("version not found in solc output"))?
            .map_err(|err| eyre::eyre!(err))?;
        Ok(Version::from_str(version.trim_start_matches("Version: "))?)
    } else {
        let err = String::from_utf8_lossy(&output.stderr).to_string();
        eyre::bail!(err)
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
        std::env::var("SOLC_PATH")
            .map(Solc::new)
            .unwrap_or_default()
    }

    #[test]
    fn solc_version_works() {
        solc().version().unwrap();
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
        let other = solc()
            .async_compile(&serde_json::json!(input))
            .await
            .unwrap();
        assert_eq!(out, other);
    }
}
