use semver::Version;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use crate::{
    compile::{compile_output, version_from_output, CompilerTrait},
    error::{CompilerError, Result},
    CompilerInput, CompilerOutput, Source,
};

/// The name of the `solc` binary on the system
pub const VYPER: &str = "vyper";

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Vyper {
    /// Path to the `solc` executable
    pub vyper: PathBuf,
    /// Additional arguments passed to the `solc` exectuable
    pub args: Vec<String>,
}

impl Default for Vyper {
    fn default() -> Self {
        if let Ok(vyper) = std::env::var("VYPER_PATH") {
            return Vyper::new(vyper)
        }

        Vyper::new(VYPER)
    }
}

impl CompilerTrait for Vyper {
    fn path(&self) -> PathBuf {
        self.vyper.clone()
    }

    fn arg(&mut self, arg: String) {
        self.args.push(arg);
    }

    fn args(&mut self, args: Vec<String>) {
        for arg in args {
            self.arg(arg);
        }
    }

    fn get_args(&self) -> Vec<String> {
        self.args.clone()
    }

    fn version(&self) -> Version {
        version_from_output(
            Command::new(&self.vyper)
                .arg("--version")
                .stdin(Stdio::piped())
                .stderr(Stdio::piped())
                .stdout(Stdio::piped())
                .output()
                .map_err(|err| CompilerError::io(err, &self.vyper))
                .expect("version"),
        )
        .expect("version")
    }

    fn language(&self) -> String {
        Vyper::compiler_language()
    }

    fn compile_exact(&self, input: &CompilerInput) -> Result<CompilerOutput> {
        let mut out = self.compile(input)?;
        out.retain_files(input.sources.keys().filter_map(|p| p.to_str()));
        Ok(out)
    }

    fn compile(&self, input: &CompilerInput) -> Result<CompilerOutput> {
        self.compile_as(input)
    }
}

impl Vyper {
    /// A new instance which points to `vyper`
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Vyper { vyper: path.into(), args: Vec::new() }
    }

    pub fn compiler_language() -> String {
        "Vyper".to_string()
    }

    pub fn compile_source(&self, path: impl AsRef<Path>) -> Result<CompilerOutput> {
        let path = path.as_ref();
        self.compile(&CompilerInput::new(path)?)
    }

    pub fn compile_as<T: Serialize, D: DeserializeOwned>(&self, input: &T) -> Result<D> {
        let output = self.compile_output(input)?;
        Ok(serde_json::from_slice(&output)?)
    }

    pub fn compile_output<T: Serialize>(&self, input: &T) -> Result<Vec<u8>> {
        let mut cmd = Command::new(&self.vyper);

        // Filter out solc arguments
        let mut args = vec![];
        let mut skip = false;
        for arg in &self.args {
            if skip {
                continue
            }
            if arg == "--allow-paths" {
                skip = true;
            } else {
                skip = false;
                args.push(arg);
            }
        }

        let mut child = cmd
            .args(&args)
            .arg("--standard-json")
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|err| CompilerError::io(err, &self.vyper))?; // todo error
        let stdin = child.stdin.take().expect("Stdin exists.");
        serde_json::to_writer(stdin, input)?;
        compile_output(
            child.wait_with_output().map_err(|err| CompilerError::Message(err.to_string()))?,
        )
    }

    pub fn version_short(&self) -> Result<Version> {
        let version = self.version()?;
        Ok(Version::new(version.major, version.minor, version.patch))
    }

    pub fn version(&self) -> Result<Version> {
        version_from_output(
            Command::new(&self.vyper)
                .arg("--version")
                .stdin(Stdio::piped())
                .stderr(Stdio::piped())
                .stdout(Stdio::piped())
                .output()
                .map_err(|err| CompilerError::io(err, &self.vyper))?,
        )
    }

    #[cfg(feature = "async")]
    /// Convenience function for compiling all sources under the given path
    pub async fn async_compile_source<T: Serialize>(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<CompilerOutput> {
        self.async_compile(&CompilerInput::with_sources(Source::async_read_all_from(path).await?))
            .await
    }

    #[cfg(feature = "async")]
    /// Run `solc --stand-json` and return the `solc`'s output as
    /// `CompilerOutput`
    pub async fn async_compile<T: Serialize + std::marker::Sync>(
        &self,
        input: &T,
    ) -> Result<CompilerOutput> {
        self.async_compile_as(input).await
    }

    #[cfg(feature = "async")]
    /// Run `solc --stand-json` and return the `solc`'s output as the given json
    /// output
    pub async fn async_compile_as<T: Serialize + std::marker::Sync, D: DeserializeOwned>(
        &self,
        input: &T,
    ) -> Result<D> {
        let output = self.async_compile_output(input).await?;
        Ok(serde_json::from_slice(&output)?)
    }

    #[cfg(feature = "async")]
    pub async fn async_compile_output<T: Serialize + std::marker::Sync>(
        &self,
        input: &T,
    ) -> Result<Vec<u8>> {
        use tokio::io::AsyncWriteExt;
        let content = serde_json::to_vec(input)?;
        let mut child = tokio::process::Command::new(&self.vyper)
            .args(&self.args)
            .arg("--standard-json")
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|err| CompilerError::io(err, &self.vyper))?;
        let stdin = child.stdin.as_mut().unwrap();
        stdin.write_all(&content).await.map_err(|err| CompilerError::io(err, &self.vyper))?;
        stdin.flush().await.map_err(|err| CompilerError::io(err, &self.vyper))?;
        compile_output(
            child.wait_with_output().await.map_err(|err| CompilerError::io(err, &self.vyper))?,
        )
    }

    #[cfg(feature = "async")]
    pub async fn async_version(&self) -> Result<Version> {
        version_from_output(
            tokio::process::Command::new(&self.vyper)
                .arg("--version")
                .stdin(Stdio::piped())
                .stderr(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .map_err(|err| CompilerError::io(err, &self.vyper))?
                .wait_with_output()
                .await
                .map_err(|err| CompilerError::io(err, &self.vyper))?,
        )
    }
}
