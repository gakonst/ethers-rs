


pub struct ProjectBuilder<T: ArtifactOutput = ConfigurableArtifacts> {
    /// The layout of the
    paths: Option<ProjectPathsConfig>,
    /// Whether caching is enabled, default is true.
    cached: bool,
    /// Whether writing artifacts to disk is enabled, default is true.
    no_artifacts: bool,
    /// Whether automatic solc version detection is enabled
    auto_detect: bool,
    /// Use offline mode
    offline: bool,
    /// handles all artifacts related tasks
    artifacts: T,
    /// Which error codes to ignore
    pub ignored_error_codes: Vec<u64>,
    /// All allowed paths
    pub allowed_paths: Vec<PathBuf>,
    /// Maximum number of compiling processes to run simultaneously.
    pub processes: usize,
}

impl<T: ArtifactOutput> ProjectBuilder<T> {
    /// Create a new builder with the given artifacts handler
    pub fn new(artifacts: T) -> Self {
        Self {
            paths: None,
            cached: true,
            no_artifacts: false,
            auto_detect: true,
            offline: false,
            artifacts,
            ignored_error_codes: Vec::new(),
            allowed_paths: vec![],
            processes: None,
        }
    }

    #[must_use]
    pub fn paths(mut self, paths: ProjectPathsConfig) -> Self {
        self.paths = Some(paths);
        self
    }

    #[must_use]
    pub fn ignore_error_code(mut self, code: u64) -> Self {
        self.ignored_error_codes.push(code);
        self
    }

    #[must_use]
    pub fn ignore_error_codes(mut self, codes: impl IntoIterator<Item = u64>) -> Self {
        for code in codes {
            self = self.ignore_error_code(code);
        }
        self
    }

    /// Disables cached builds
    #[must_use]
    pub fn ephemeral(self) -> Self {
        self.set_cached(false)
    }

    /// Sets the cache status
    #[must_use]
    pub fn set_cached(mut self, cached: bool) -> Self {
        self.cached = cached;
        self
    }

    /// Activates offline mode
    ///
    /// Prevents network possible access to download/check solc installs
    #[must_use]
    pub fn offline(self) -> Self {
        self.set_offline(true)
    }

    /// Sets the offline status
    #[must_use]
    pub fn set_offline(mut self, offline: bool) -> Self {
        self.offline = offline;
        self
    }

    /// Disables writing artifacts to disk
    #[must_use]
    pub fn no_artifacts(self) -> Self {
        self.set_no_artifacts(true)
    }

    /// Sets the no artifacts status
    #[must_use]
    pub fn set_no_artifacts(mut self, artifacts: bool) -> Self {
        self.no_artifacts = artifacts;
        self
    }

    /// Sets automatic solc version detection
    #[must_use]
    pub fn set_auto_detect(mut self, auto_detect: bool) -> Self {
        self.auto_detect = auto_detect;
        self
    }

    /// Disables automatic solc version detection
    #[must_use]
    pub fn no_auto_detect(self) -> Self {
        self.set_auto_detect(false)
    }

    /// Sets the maximum number of parallel compiling processes to run simultaneously.
    ///
    /// **Panics if `count == 0`**
    pub fn set_processes(mut self, count: usize) {
        assert!(count > 0);
        self.processes = Some(count);
        self
    }

    /// Sets the number of parallel processes to `1`, no parallelization
    #[must_use]
    pub fn single_processes(self) -> Self {
        self.set_processes(1)
    }

    /// Set arbitrary `ArtifactOutputHandler`
    pub fn artifacts<A: ArtifactOutput>(self, artifacts: A) -> ProjectBuilder<A> {
        let ProjectBuilder {
            paths,
            cached,
            no_artifacts,
            auto_detect,
            ignored_error_codes,
            allowed_paths,
            processes,
            offline,
            ..
        } = self;
        ProjectBuilder {
            paths,
            cached,
            no_artifacts,
            auto_detect,
            offline,
            artifacts,
            ignored_error_codes,
            allowed_paths,
            processes,
        }
    }

    /// Adds an allowed-path to the solc executable
    #[must_use]
    pub fn allowed_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.allowed_paths.push(path.into());
        self
    }

    /// Adds multiple allowed-path to the solc executable
    #[must_use]
    pub fn allowed_paths<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<PathBuf>,
    {
        for arg in args {
            self = self.allowed_path(arg);
        }
        self
    }

    pub fn build(self) -> Result<Project<T>> {
        let Self {
            paths,
            cached,
            no_artifacts,
            auto_detect,
            artifacts,
            ignored_error_codes,
            mut allowed_paths,
            processes,
            offline,
        } = self;

        let paths = paths.map(Ok).unwrap_or_else(ProjectPathsConfig::current_hardhat)?;

        if allowed_paths.is_empty() {
            // allow every contract under root by default
            allowed_paths.push(paths.root.clone())
        }

        Ok(Project {
            paths,
            cached,
            no_artifacts,
            auto_detect,
            artifacts,
            ignored_error_codes,
            allowed_lib_paths: allowed_paths.into(),
            processes: processes.unwrap_or_else(::num_cpus::get),
            offline,
        })
    }
}

impl<T: ArtifactOutput + Default> Default for ProjectBuilder<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: ArtifactOutput> ArtifactOutput for Project<T> {
    type Artifact = T::Artifact;

    fn on_output(
        &self,
        contracts: &VersionedContracts,
        layout: &ProjectPathsConfig,
    ) -> Result<Artifacts<Self::Artifact>> {
        self.artifacts_handler().on_output(contracts, layout)
    }

    fn write_contract_extras(&self, contract: &Contract, file: &Path) -> Result<()> {
        self.artifacts_handler().write_contract_extras(contract, file)
    }

    fn write_extras(
        &self,
        contracts: &VersionedContracts,
        layout: &ProjectPathsConfig,
    ) -> Result<()> {
        self.artifacts_handler().write_extras(contracts, layout)
    }

    fn output_file_name(name: impl AsRef<str>) -> PathBuf {
        T::output_file_name(name)
    }

    fn output_file_name_versioned(name: impl AsRef<str>, version: &Version) -> PathBuf {
        T::output_file_name_versioned(name, version)
    }

    fn output_file(contract_file: impl AsRef<Path>, name: impl AsRef<str>) -> PathBuf {
        T::output_file(contract_file, name)
    }

    fn output_file_versioned(
        contract_file: impl AsRef<Path>,
        name: impl AsRef<str>,
        version: &Version,
    ) -> PathBuf {
        T::output_file_versioned(contract_file, name, version)
    }

    fn contract_name(file: impl AsRef<Path>) -> Option<String> {
        T::contract_name(file)
    }

    fn output_exists(
        contract_file: impl AsRef<Path>,
        name: impl AsRef<str>,
        root: impl AsRef<Path>,
    ) -> bool {
        T::output_exists(contract_file, name, root)
    }

    fn read_cached_artifact(path: impl AsRef<Path>) -> Result<Self::Artifact> {
        T::read_cached_artifact(path)
    }

    fn read_cached_artifacts<P, I>(files: I) -> Result<BTreeMap<PathBuf, Self::Artifact>>
    where
        I: IntoIterator<Item = P>,
        P: Into<PathBuf>,
    {
        T::read_cached_artifacts(files)
    }

    fn contract_to_artifact(&self, file: &str, name: &str, contract: Contract) -> Self::Artifact {
        self.artifacts_handler().contract_to_artifact(file, name, contract)
    }

    fn output_to_artifacts(&self, contracts: &VersionedContracts) -> Artifacts<Self::Artifact> {
        self.artifacts_handler().output_to_artifacts(contracts)
    }
}

#[cfg(test)]
#[cfg(all(feature = "svm", feature = "async"))]
mod tests {
    use crate::remappings::Remapping;

    #[test]
    fn test_build_all_versions() {
        use super::*;

        let paths = ProjectPathsConfig::builder()
            .root("./test-data/test-contract-versions")
            .sources("./test-data/test-contract-versions")
            .build()
            .unwrap();
        let project = Project::builder().paths(paths).no_artifacts().ephemeral().build().unwrap();
        let compiled = project.compile().unwrap();
        assert!(!compiled.has_compiler_errors());
        let contracts = compiled.output().contracts;
        // Contracts A to F
        assert_eq!(contracts.contracts().count(), 5);
    }

    #[test]
    fn test_build_many_libs() {
        use super::*;

        let root = utils::canonicalize("./test-data/test-contract-libs").unwrap();

        let paths = ProjectPathsConfig::builder()
            .root(&root)
            .sources(root.join("src"))
            .lib(root.join("lib1"))
            .lib(root.join("lib2"))
            .remappings(
                Remapping::find_many(&root.join("lib1"))
                    .into_iter()
                    .chain(Remapping::find_many(&root.join("lib2"))),
            )
            .build()
            .unwrap();
        let project = Project::builder()
            .paths(paths)
            .no_artifacts()
            .ephemeral()
            .no_artifacts()
            .build()
            .unwrap();
        let compiled = project.compile().unwrap();
        assert!(!compiled.has_compiler_errors());
        let contracts = compiled.output().contracts;
        assert_eq!(contracts.contracts().count(), 3);
    }

    #[test]
    fn test_build_remappings() {
        use super::*;

        let root = utils::canonicalize("./test-data/test-contract-remappings").unwrap();
        let paths = ProjectPathsConfig::builder()
            .root(&root)
            .sources(root.join("src"))
            .lib(root.join("lib"))
            .remappings(Remapping::find_many(&root.join("lib")))
            .build()
            .unwrap();
        let project = Project::builder().no_artifacts().paths(paths).ephemeral().build().unwrap();
        let compiled = project.compile().unwrap();
        assert!(!compiled.has_compiler_errors());
        let contracts = compiled.output().contracts;
        assert_eq!(contracts.contracts().count(), 2);
    }
}
