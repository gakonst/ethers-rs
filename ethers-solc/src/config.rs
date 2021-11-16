use crate::{
    artifacts::{CompactContract, CompactContractRef, Contract, Settings},
    cache::SOLIDITY_FILES_CACHE_FILENAME,
    error::Result,
    remappings::Remapping,
    CompilerOutput, Solc,
};
use ethers_core::{abi::Abi, types::Bytes};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    convert::TryFrom,
    fmt, fs, io,
    path::{Path, PathBuf},
};

/// Where to find all files or where to write them
#[derive(Debug, Clone)]
pub struct ProjectPathsConfig {
    /// Project root
    pub root: PathBuf,
    /// Path to the cache, if any
    pub cache: PathBuf,
    /// Where to store build artifacts
    pub artifacts: PathBuf,
    /// Where to find sources
    pub sources: PathBuf,
    /// Where to find tests
    pub tests: PathBuf,
    /// Where to look for libraries
    pub libraries: Vec<PathBuf>,
    /// The compiler remappings
    pub remappings: Vec<Remapping>,
}

impl ProjectPathsConfig {
    pub fn builder() -> ProjectPathsConfigBuilder {
        ProjectPathsConfigBuilder::default()
    }

    /// Creates a new hardhat style config instance which points to the canonicalized root path
    pub fn hardhat(root: impl AsRef<Path>) -> Result<Self> {
        PathStyle::HardHat.paths(root)
    }

    /// Creates a new dapptools style config instance which points to the canonicalized root path
    pub fn dapptools(root: impl AsRef<Path>) -> Result<Self> {
        PathStyle::Dapptools.paths(root)
    }

    /// Creates a new config with the current directory as the root
    pub fn current_hardhat() -> Result<Self> {
        Self::hardhat(std::env::current_dir()?)
    }

    /// Creates a new config with the current directory as the root
    pub fn current_dapptools() -> Result<Self> {
        Self::dapptools(std::env::current_dir()?)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PathStyle {
    HardHat,
    Dapptools,
}

impl PathStyle {
    pub fn paths(&self, root: impl AsRef<Path>) -> Result<ProjectPathsConfig> {
        let root = std::fs::canonicalize(root)?;

        Ok(match self {
            PathStyle::Dapptools => ProjectPathsConfig::builder()
                .sources(root.join("src"))
                .artifacts(root.join("out"))
                .lib(root.join("lib"))
                .remappings(Remapping::find_many(&root.join("lib"))?)
                .root(root)
                .build()?,
            PathStyle::HardHat => ProjectPathsConfig::builder()
                .sources(root.join("contracts"))
                .artifacts(root.join("artifacts"))
                .lib(root.join("node_modules"))
                .root(root)
                .build()?,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct ProjectPathsConfigBuilder {
    root: Option<PathBuf>,
    cache: Option<PathBuf>,
    artifacts: Option<PathBuf>,
    sources: Option<PathBuf>,
    tests: Option<PathBuf>,
    libraries: Option<Vec<PathBuf>>,
    remappings: Option<Vec<Remapping>>,
}

impl ProjectPathsConfigBuilder {
    pub fn root(mut self, root: impl Into<PathBuf>) -> Self {
        self.root = Some(root.into());
        self
    }

    pub fn cache(mut self, cache: impl Into<PathBuf>) -> Self {
        self.cache = Some(cache.into());
        self
    }

    pub fn artifacts(mut self, artifacts: impl Into<PathBuf>) -> Self {
        self.artifacts = Some(artifacts.into());
        self
    }

    pub fn sources(mut self, sources: impl Into<PathBuf>) -> Self {
        self.sources = Some(sources.into());
        self
    }

    pub fn tests(mut self, tests: impl Into<PathBuf>) -> Self {
        self.tests = Some(tests.into());
        self
    }

    /// Specifically disallow additional libraries
    pub fn no_libs(mut self) -> Self {
        self.libraries = Some(Vec::new());
        self
    }

    pub fn lib(mut self, lib: impl Into<PathBuf>) -> Self {
        self.libraries.get_or_insert_with(Vec::new).push(lib.into());
        self
    }

    pub fn libs(mut self, libs: impl IntoIterator<Item = impl Into<PathBuf>>) -> Self {
        let libraries = self.libraries.get_or_insert_with(Vec::new);
        for lib in libs.into_iter() {
            libraries.push(lib.into());
        }
        self
    }

    pub fn remapping(mut self, remapping: Remapping) -> Self {
        self.remappings.get_or_insert_with(Vec::new).push(remapping);
        self
    }

    pub fn remappings(mut self, remappings: impl IntoIterator<Item = Remapping>) -> Self {
        let our_remappings = self.remappings.get_or_insert_with(Vec::new);
        for remapping in remappings.into_iter() {
            our_remappings.push(remapping);
        }
        self
    }

    pub fn build(self) -> io::Result<ProjectPathsConfig> {
        let root = self.root.map(Ok).unwrap_or_else(std::env::current_dir)?;
        let root = std::fs::canonicalize(root)?;

        Ok(ProjectPathsConfig {
            cache: self
                .cache
                .unwrap_or_else(|| root.join("cache").join(SOLIDITY_FILES_CACHE_FILENAME)),
            artifacts: self.artifacts.unwrap_or_else(|| root.join("artifacts")),
            sources: self.sources.unwrap_or_else(|| root.join("contracts")),
            tests: self.tests.unwrap_or_else(|| root.join("tests")),
            libraries: self.libraries.unwrap_or_default(),
            remappings: self.remappings.unwrap_or_default(),
            root,
        })
    }
}

/// The config to use when compiling the contracts
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SolcConfig {
    /// Configured solc version
    pub version: String,
    /// How the file was compiled
    pub settings: Settings,
}

impl SolcConfig {
    /// # Example
    ///
    /// Autodetect solc version and default settings
    ///
    /// ```rust
    /// use ethers_solc::SolcConfig;
    /// let config = SolcConfig::builder().build().unwrap();
    /// ```
    pub fn builder() -> SolcConfigBuilder {
        SolcConfigBuilder::default()
    }
}

#[derive(Default)]
pub struct SolcConfigBuilder {
    version: Option<String>,
    settings: Option<Settings>,
}

impl SolcConfigBuilder {
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    pub fn settings(mut self, settings: Settings) -> Self {
        self.settings = Some(settings);
        self
    }

    /// Creates the solc config
    ///
    /// If no solc version is configured then it will be determined by calling `solc --version`.
    pub fn build(self) -> Result<SolcConfig> {
        let Self { version, settings } = self;
        let version =
            version.map(Ok).unwrap_or_else(|| Solc::default().version().map(|s| s.to_string()))?;
        let settings = settings.unwrap_or_default();
        Ok(SolcConfig { version, settings })
    }
}

pub type Artifacts<T> = BTreeMap<String, BTreeMap<String, T>>;

pub trait Artifact {
    fn into_inner(self) -> (Option<Abi>, Option<Bytes>);
}

impl Artifact for CompactContract {
    fn into_inner(self) -> (Option<Abi>, Option<Bytes>) {
        (self.abi, self.bin)
    }
}

impl Artifact for serde_json::Value {
    fn into_inner(self) -> (Option<Abi>, Option<Bytes>) {
        let abi = self.get("abi").map(|abi| {
            serde_json::from_value::<Abi>(abi.clone()).expect("could not get artifact abi")
        });
        let bytecode = self.get("bin").map(|bin| {
            serde_json::from_value::<Bytes>(bin.clone()).expect("could not get artifact bytecode")
        });

        (abi, bytecode)
    }
}

pub trait ArtifactOutput {
    /// How Artifacts are stored
    type Artifact: Artifact;

    /// Handle the compiler output.
    fn on_output(output: &CompilerOutput, layout: &ProjectPathsConfig) -> Result<()>;

    /// Returns the file name for the contract's artifact
    fn output_file_name(name: impl AsRef<str>) -> PathBuf {
        format!("{}.json", name.as_ref()).into()
    }

    /// Returns the path to the contract's artifact location based on the contract's file and name
    ///
    /// This returns `contract.sol/contract.json` by default
    fn output_file(contract_file: impl AsRef<Path>, name: impl AsRef<str>) -> PathBuf {
        let name = name.as_ref();
        contract_file
            .as_ref()
            .file_name()
            .map(Path::new)
            .map(|p| p.join(Self::output_file_name(name)))
            .unwrap_or_else(|| Self::output_file_name(name))
    }

    /// The inverse of `contract_file_name`
    ///
    /// Expected to return the solidity contract's name derived from the file path
    /// `sources/Greeter.sol` -> `Greeter`
    fn contract_name(file: impl AsRef<Path>) -> Option<String> {
        file.as_ref().file_stem().and_then(|s| s.to_str().map(|s| s.to_string()))
    }

    /// Whether the corresponding artifact of the given contract file and name exists
    fn output_exists(
        contract_file: impl AsRef<Path>,
        name: impl AsRef<str>,
        root: impl AsRef<Path>,
    ) -> bool {
        root.as_ref().join(Self::output_file(contract_file, name)).exists()
    }

    fn read_cached_artifact(path: impl AsRef<Path>) -> Result<Self::Artifact>;

    /// Read the cached artifacts from disk
    fn read_cached_artifacts<T, I>(files: I) -> Result<BTreeMap<PathBuf, Self::Artifact>>
    where
        I: IntoIterator<Item = T>,
        T: Into<PathBuf>,
    {
        let mut artifacts = BTreeMap::default();
        for path in files.into_iter() {
            let path = path.into();
            let artifact = Self::read_cached_artifact(&path)?;
            artifacts.insert(path, artifact);
        }
        Ok(artifacts)
    }

    /// Convert a contract to the artifact type
    fn contract_to_artifact(contract: Contract) -> Self::Artifact;

    /// Convert the compiler output into a set of artifacts
    fn output_to_artifacts(output: CompilerOutput) -> Artifacts<Self::Artifact> {
        output
            .contracts
            .into_iter()
            .map(|(s, contracts)| {
                (
                    s,
                    contracts
                        .into_iter()
                        .map(|(s, c)| (s, Self::contract_to_artifact(c)))
                        .collect(),
                )
            })
            .collect()
    }
}

/// An Artifacts implementation that uses a compact representation
///
/// Creates a single json artifact with
/// ```json
///  {
///    "abi": [],
///    "bin": "...",
///    "runtime-bin": "..."
///  }
/// ```
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct MinimalCombinedArtifacts;

impl ArtifactOutput for MinimalCombinedArtifacts {
    type Artifact = CompactContract;

    fn on_output(output: &CompilerOutput, layout: &ProjectPathsConfig) -> Result<()> {
        fs::create_dir_all(&layout.artifacts)?;
        for (file, contracts) in output.contracts.iter() {
            for (name, contract) in contracts {
                let artifact = Self::output_file(file, name);
                let file = layout.artifacts.join(artifact);
                if let Some(parent) = file.parent() {
                    fs::create_dir_all(parent)?;
                }
                let min = CompactContractRef::from(contract);
                fs::write(file, serde_json::to_vec_pretty(&min)?)?
            }
        }
        Ok(())
    }

    fn read_cached_artifact(path: impl AsRef<Path>) -> Result<Self::Artifact> {
        let file = fs::File::open(path.as_ref())?;
        Ok(serde_json::from_reader(file)?)
    }

    fn contract_to_artifact(contract: Contract) -> Self::Artifact {
        CompactContract::from(contract)
    }
}

/// Hardhat style artifacts
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct HardhatArtifacts;

impl ArtifactOutput for HardhatArtifacts {
    type Artifact = serde_json::Value;

    fn on_output(_output: &CompilerOutput, _layout: &ProjectPathsConfig) -> Result<()> {
        todo!("Hardhat style artifacts not yet implemented")
    }

    fn read_cached_artifact(_path: impl AsRef<Path>) -> Result<Self::Artifact> {
        todo!("Hardhat style artifacts not yet implemented")
    }

    fn contract_to_artifact(_contract: Contract) -> Self::Artifact {
        todo!("Hardhat style artifacts not yet implemented")
    }
}

/// Helper struct for serializing `--allow-paths` arguments to Solc
///
/// From the [Solc docs](https://docs.soliditylang.org/en/v0.8.9/using-the-compiler.html#base-path-and-import-remapping):
/// For security reasons the compiler has restrictions on what directories it can access.
/// Directories of source files specified on the command line and target paths of
/// remappings are automatically allowed to be accessed by the file reader,
/// but everything else is rejected by default. Additional paths (and their subdirectories)
/// can be allowed via the --allow-paths /sample/path,/another/sample/path switch.
/// Everything inside the path specified via --base-path is always allowed.
#[derive(Clone, Debug, Default)]
pub struct AllowedLibPaths(pub(crate) Vec<PathBuf>);

impl fmt::Display for AllowedLibPaths {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let lib_paths = self
            .0
            .iter()
            .filter(|path| path.exists())
            .map(|path| format!("{}", path.display()))
            .collect::<Vec<_>>()
            .join(",");
        write!(f, "{}", lib_paths)
    }
}

impl<T: Into<PathBuf>> TryFrom<Vec<T>> for AllowedLibPaths {
    type Error = std::io::Error;

    fn try_from(libs: Vec<T>) -> std::result::Result<Self, Self::Error> {
        let libs = libs
            .into_iter()
            .map(|lib| {
                let path: PathBuf = lib.into();
                let lib = std::fs::canonicalize(path)?;
                Ok(lib)
            })
            .collect::<std::result::Result<Vec<_>, std::io::Error>>()?;
        Ok(AllowedLibPaths(libs))
    }
}
