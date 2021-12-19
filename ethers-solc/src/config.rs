use crate::{
    artifacts::{CompactContract, CompactContractRef, Contract, Settings},
    cache::SOLIDITY_FILES_CACHE_FILENAME,
    error::{Result, SolcError, SolcIoError},
    hh::HardhatArtifact,
    remappings::Remapping,
    CompilerOutput,
};
use ethers_core::{abi::Abi, types::Bytes};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
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
        Self::hardhat(std::env::current_dir().map_err(|err| SolcError::io(err, "."))?)
    }

    /// Creates a new config with the current directory as the root
    pub fn current_dapptools() -> Result<Self> {
        Self::dapptools(std::env::current_dir().map_err(|err| SolcError::io(err, "."))?)
    }

    /// Creates all configured dirs and files
    pub fn create_all(&self) -> std::result::Result<(), SolcIoError> {
        if let Some(parent) = self.cache.parent() {
            fs::create_dir_all(parent).map_err(|err| SolcIoError::new(err, parent))?;
        }
        fs::create_dir_all(&self.artifacts)
            .map_err(|err| SolcIoError::new(err, &self.artifacts))?;
        fs::create_dir_all(&self.sources).map_err(|err| SolcIoError::new(err, &self.sources))?;
        fs::create_dir_all(&self.tests).map_err(|err| SolcIoError::new(err, &self.tests))?;
        for lib in &self.libraries {
            fs::create_dir_all(lib).map_err(|err| SolcIoError::new(err, lib))?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PathStyle {
    HardHat,
    Dapptools,
}

impl PathStyle {
    pub fn paths(&self, root: impl AsRef<Path>) -> Result<ProjectPathsConfig> {
        let root = root.as_ref();
        let root = std::fs::canonicalize(root).map_err(|err| SolcError::io(err, root))?;

        Ok(match self {
            PathStyle::Dapptools => ProjectPathsConfig::builder()
                .sources(root.join("src"))
                .artifacts(root.join("out"))
                .lib(root.join("lib"))
                .remappings(Remapping::find_many(&root.join("lib")))
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

    pub fn build_with_root(self, root: impl Into<PathBuf>) -> ProjectPathsConfig {
        let root = root.into();
        ProjectPathsConfig {
            cache: self
                .cache
                .unwrap_or_else(|| root.join("cache").join(SOLIDITY_FILES_CACHE_FILENAME)),
            artifacts: self.artifacts.unwrap_or_else(|| root.join("artifacts")),
            sources: self.sources.unwrap_or_else(|| root.join("contracts")),
            tests: self.tests.unwrap_or_else(|| root.join("tests")),
            libraries: self.libraries.unwrap_or_default(),
            remappings: self.remappings.unwrap_or_default(),
            root,
        }
    }

    pub fn build(self) -> std::result::Result<ProjectPathsConfig, SolcIoError> {
        let root = self
            .root
            .clone()
            .map(Ok)
            .unwrap_or_else(std::env::current_dir)
            .map_err(|err| SolcIoError::new(err, "."))?;
        let root = std::fs::canonicalize(&root).map_err(|err| SolcIoError::new(err, &root))?;
        Ok(self.build_with_root(root))
    }
}

/// The config to use when compiling the contracts
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SolcConfig {
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
    settings: Option<Settings>,
}

impl SolcConfigBuilder {
    pub fn settings(mut self, settings: Settings) -> Self {
        self.settings = Some(settings);
        self
    }

    /// Creates the solc config
    ///
    /// If no solc version is configured then it will be determined by calling `solc --version`.
    pub fn build(self) -> Result<SolcConfig> {
        let Self { settings } = self;
        Ok(SolcConfig { settings: settings.unwrap_or_default() })
    }
}

pub type Artifacts<T> = BTreeMap<String, BTreeMap<String, T>>;

pub trait Artifact {
    /// Returns the artifact's `Abi` and bytecode
    fn into_inner(self) -> (Option<Abi>, Option<Bytes>);

    /// Turns the artifact into a container type for abi, bytecode and deployed bytecode
    fn into_compact_contract(self) -> CompactContract;

    /// Returns the contents of this type as a single tuple of abi, bytecode and deployed bytecode
    fn into_parts(self) -> (Option<Abi>, Option<Bytes>, Option<Bytes>);
}

impl<T: Into<CompactContract>> Artifact for T {
    fn into_inner(self) -> (Option<Abi>, Option<Bytes>) {
        let artifact = self.into_compact_contract();
        (artifact.abi, artifact.bin.and_then(|bin| bin.into_bytes()))
    }

    fn into_compact_contract(self) -> CompactContract {
        self.into()
    }

    fn into_parts(self) -> (Option<Abi>, Option<Bytes>, Option<Bytes>) {
        self.into_compact_contract().into_parts()
    }
}

pub trait ArtifactOutput {
    /// How Artifacts are stored
    type Artifact: Artifact + DeserializeOwned;

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

    fn read_cached_artifact(path: impl AsRef<Path>) -> Result<Self::Artifact> {
        let path = path.as_ref();
        let file = fs::File::open(path).map_err(|err| SolcError::io(err, path))?;
        let file = io::BufReader::new(file);
        Ok(serde_json::from_reader(file)?)
    }

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
    fn contract_to_artifact(file: &str, name: &str, contract: Contract) -> Self::Artifact;

    /// Convert the compiler output into a set of artifacts
    fn output_to_artifacts(output: CompilerOutput) -> Artifacts<Self::Artifact> {
        output
            .contracts
            .into_iter()
            .map(|(file, contracts)| {
                let contracts = contracts
                    .into_iter()
                    .map(|(name, c)| {
                        let contract = Self::contract_to_artifact(&file, &name, c);
                        (name, contract)
                    })
                    .collect();
                (file, contracts)
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
        fs::create_dir_all(&layout.artifacts)
            .map_err(|err| SolcError::msg(format!("Failed to create artifacts dir: {}", err)))?;
        for (file, contracts) in output.contracts.iter() {
            for (name, contract) in contracts {
                let artifact = Self::output_file(file, name);
                let file = layout.artifacts.join(artifact);
                if let Some(parent) = file.parent() {
                    fs::create_dir_all(parent).map_err(|err| {
                        SolcError::msg(format!(
                            "Failed to create artifact parent folder \"{}\": {}",
                            parent.display(),
                            err
                        ))
                    })?;
                }
                let min = CompactContractRef::from(contract);
                fs::write(&file, serde_json::to_vec_pretty(&min)?)
                    .map_err(|err| SolcError::io(err, file))?
            }
        }
        Ok(())
    }

    fn contract_to_artifact(_file: &str, _name: &str, contract: Contract) -> Self::Artifact {
        Self::Artifact::from(contract)
    }
}

/// An Artifacts handler implementation that works the same as `MinimalCombinedArtifacts` but also
/// supports reading hardhat artifacts if an initial attempt to deserialize an artifact failed
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct MinimalCombinedArtifactsHardhatFallback;

impl ArtifactOutput for MinimalCombinedArtifactsHardhatFallback {
    type Artifact = CompactContract;

    fn on_output(output: &CompilerOutput, layout: &ProjectPathsConfig) -> Result<()> {
        MinimalCombinedArtifacts::on_output(output, layout)
    }

    fn read_cached_artifact(path: impl AsRef<Path>) -> Result<Self::Artifact> {
        let path = path.as_ref();
        let content = fs::read_to_string(path).map_err(|err| SolcError::io(err, path))?;
        if let Ok(a) = serde_json::from_str(&content) {
            Ok(a)
        } else {
            tracing::error!("Failed to deserialize compact artifact");
            tracing::trace!("Fallback to hardhat artifact deserialization");
            let artifact = serde_json::from_str::<HardhatArtifact>(&content)?;
            tracing::trace!("successfully deserialized hardhat artifact");
            Ok(artifact.into_compact_contract())
        }
    }

    fn contract_to_artifact(file: &str, name: &str, contract: Contract) -> Self::Artifact {
        MinimalCombinedArtifacts::contract_to_artifact(file, name, contract)
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
    type Error = SolcIoError;

    fn try_from(libs: Vec<T>) -> std::result::Result<Self, Self::Error> {
        let libs = libs
            .into_iter()
            .map(|lib| {
                let path: PathBuf = lib.into();
                let lib =
                    std::fs::canonicalize(&path).map_err(|err| SolcIoError::new(err, path))?;
                Ok(lib)
            })
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(AllowedLibPaths(libs))
    }
}
