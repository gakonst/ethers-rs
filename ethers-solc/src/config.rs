use crate::{
    artifacts::{output_selection::ContractOutputSelection, Settings},
    cache::SOLIDITY_FILES_CACHE_FILENAME,
    error::{Result, SolcError, SolcIoError},
    remappings::Remapping,
    resolver::{Graph, SolImportAlias},
    utils, Source, Sources,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeSet, HashSet},
    fmt::{self, Formatter},
    fs,
    ops::{Deref, DerefMut},
    path::{Component, Path, PathBuf},
};

/// Where to find all files or where to write them
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectPathsConfig {
    /// Project root
    pub root: PathBuf,
    /// Path to the cache, if any
    pub cache: PathBuf,
    /// Where to store build artifacts
    pub artifacts: PathBuf,
    /// Where to store the build info files
    pub build_infos: PathBuf,
    /// Where to find sources
    pub sources: PathBuf,
    /// Where to find tests
    pub tests: PathBuf,
    /// Where to find scripts
    pub scripts: PathBuf,
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

    /// Returns a new [ProjectPaths] instance that contains all directories configured for this
    /// project
    pub fn paths(&self) -> ProjectPaths {
        ProjectPaths {
            artifacts: self.artifacts.clone(),
            build_infos: self.build_infos.clone(),
            sources: self.sources.clone(),
            tests: self.tests.clone(),
            scripts: self.scripts.clone(),
            libraries: self.libraries.iter().cloned().collect(),
        }
    }

    /// Same as [`paths`][ProjectPathsConfig::paths] but strips the `root` form all paths.
    ///
    /// See: [`ProjectPaths::strip_prefix_all`]
    pub fn paths_relative(&self) -> ProjectPaths {
        let mut paths = self.paths();
        paths.strip_prefix_all(&self.root);
        paths
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
        fs::create_dir_all(&self.scripts).map_err(|err| SolcIoError::new(err, &self.scripts))?;
        for lib in &self.libraries {
            fs::create_dir_all(lib).map_err(|err| SolcIoError::new(err, lib))?;
        }
        Ok(())
    }

    /// Returns all sources found under the project's configured `sources` path
    pub fn read_sources(&self) -> Result<Sources> {
        tracing::trace!("reading all sources from \"{}\"", self.sources.display());
        Ok(Source::read_all_from(&self.sources)?)
    }

    /// Returns all sources found under the project's configured `test` path
    pub fn read_tests(&self) -> Result<Sources> {
        tracing::trace!("reading all tests from \"{}\"", self.tests.display());
        Ok(Source::read_all_from(&self.tests)?)
    }

    /// Returns all sources found under the project's configured `script` path
    pub fn read_scripts(&self) -> Result<Sources> {
        tracing::trace!("reading all scripts from \"{}\"", self.scripts.display());
        Ok(Source::read_all_from(&self.scripts)?)
    }

    /// Returns true if the there is at least one solidity file in this config.
    ///
    /// See also, `Self::input_files()`
    pub fn has_input_files(&self) -> bool {
        self.input_files_iter().next().is_some()
    }

    /// Returns an iterator that yields all solidity file paths for `Self::sources`, `Self::tests`
    /// and `Self::scripts`
    pub fn input_files_iter(&self) -> impl Iterator<Item = PathBuf> + '_ {
        utils::source_files_iter(&self.sources)
            .chain(utils::source_files_iter(&self.tests))
            .chain(utils::source_files_iter(&self.scripts))
    }

    /// Returns the combined set solidity file paths for `Self::sources`, `Self::tests` and
    /// `Self::scripts`
    pub fn input_files(&self) -> Vec<PathBuf> {
        self.input_files_iter().collect()
    }

    /// Returns the combined set of `Self::read_sources` + `Self::read_tests` + `Self::read_scripts`
    pub fn read_input_files(&self) -> Result<Sources> {
        Ok(Source::read_all_files(self.input_files())?)
    }

    /// Converts all `\\` separators in _all_ paths to `/`
    pub fn slash_paths(&mut self) {
        #[cfg(windows)]
        {
            use path_slash::PathBufExt;

            let slashed = |p: &mut PathBuf| {
                *p = p.to_slash_lossy().as_ref().into();
            };
            slashed(&mut self.root);
            slashed(&mut self.cache);
            slashed(&mut self.artifacts);
            slashed(&mut self.build_infos);
            slashed(&mut self.sources);
            slashed(&mut self.tests);
            slashed(&mut self.scripts);

            self.libraries.iter_mut().for_each(slashed);
            self.remappings.iter_mut().for_each(Remapping::slash_path);
        }
    }

    /// Returns true if the `file` belongs to a `library`, See [`Self::find_library_ancestor()`]
    pub fn has_library_ancestor(&self, file: impl AsRef<Path>) -> bool {
        self.find_library_ancestor(file).is_some()
    }

    /// Returns the library the file belongs to
    ///
    /// Returns the first library that is an ancestor of the given `file`.
    ///
    /// **Note:** this does not resolve remappings [`Self::resolve_import()`], instead this merely
    /// checks if a `library` is a parent of `file`
    ///
    /// # Example
    ///
    /// ```
    ///  use std::path::Path;
    /// use ethers_solc::ProjectPathsConfig;
    /// let config = ProjectPathsConfig::builder().lib("lib").build().unwrap();
    /// assert_eq!(config.find_library_ancestor("lib/src/Greeter.sol").unwrap(), Path::new("lib"));
    /// ```
    pub fn find_library_ancestor(&self, file: impl AsRef<Path>) -> Option<&PathBuf> {
        let file = file.as_ref();

        for lib in &self.libraries {
            if lib.is_relative() &&
                file.is_absolute() &&
                file.starts_with(&self.root) &&
                file.starts_with(self.root.join(lib)) ||
                file.is_relative() &&
                    lib.is_absolute() &&
                    lib.starts_with(&self.root) &&
                    self.root.join(file).starts_with(lib)
            {
                return Some(lib)
            }
            if file.starts_with(lib) {
                return Some(lib)
            }
        }

        None
    }

    /// Attempts to resolve an `import` from the given working directory.
    ///
    /// The `cwd` path is the parent dir of the file that includes the `import`
    ///
    /// This will also populate the `include_paths` with any nested library root paths that should
    /// be provided to solc via `--include-path` because it uses absolute imports.
    pub fn resolve_import_and_include_paths(
        &self,
        cwd: &Path,
        import: &Path,
        include_paths: &mut IncludePaths,
    ) -> Result<PathBuf> {
        let component = import
            .components()
            .next()
            .ok_or_else(|| SolcError::msg(format!("Empty import path {}", import.display())))?;

        if component == Component::CurDir || component == Component::ParentDir {
            // if the import is relative we assume it's already part of the processed input
            // file set
            utils::canonicalize(cwd.join(import)).map_err(|err| {
                SolcError::msg(format!("failed to resolve relative import \"{err:?}\""))
            })
        } else {
            // resolve library file
            let resolved = self.resolve_library_import(cwd.as_ref(), import.as_ref());

            if resolved.is_none() {
                // absolute paths in solidity are a thing for example `import
                // "src/interfaces/IConfig.sol"` which could either point to `cwd +
                // src/interfaces/IConfig.sol`, or make use of a remapping (`src/=....`)
                if let Some(lib) = self.find_library_ancestor(cwd) {
                    if let Some((include_path, import)) =
                        utils::resolve_absolute_library(lib, cwd, import)
                    {
                        // track the path for this absolute import inside a nested library
                        include_paths.insert(include_path);
                        return Ok(import)
                    }
                }
                // also try to resolve absolute imports from the project paths
                for path in [&self.root, &self.sources, &self.tests, &self.scripts] {
                    if cwd.starts_with(path) {
                        if let Ok(import) = utils::canonicalize(path.join(import)) {
                            return Ok(import)
                        }
                    }
                }
            }

            resolved.ok_or_else(|| {
                SolcError::msg(format!(
                    "failed to resolve library import \"{:?}\"",
                    import.display()
                ))
            })
        }
    }

    /// Attempts to resolve an `import` from the given working directory.
    ///
    /// The `cwd` path is the parent dir of the file that includes the `import`
    pub fn resolve_import(&self, cwd: &Path, import: &Path) -> Result<PathBuf> {
        self.resolve_import_and_include_paths(cwd, import, &mut Default::default())
    }

    /// Attempts to find the path to the real solidity file that's imported via the given `import`
    /// path by applying the configured remappings and checking the library dirs
    ///
    /// # Example
    ///
    /// Following `@aave` dependency in the `lib` folder `node_modules`
    ///
    /// ```text
    /// <root>/node_modules/@aave
    /// ├── aave-token
    /// │   ├── contracts
    /// │   │   ├── open-zeppelin
    /// │   │   ├── token
    /// ├── governance-v2
    ///     ├── contracts
    ///         ├── interfaces
    /// ```
    ///
    /// has this remapping: `@aave/=@aave/` (name:path) so contracts can be imported as
    ///
    /// ```solidity
    /// import "@aave/governance-v2/contracts/governance/Executor.sol";
    /// ```
    ///
    /// So that `Executor.sol` can be found by checking each `lib` folder (`node_modules`) with
    /// applied remappings. Applying remapping works by checking if the import path of an import
    /// statement starts with the name of a remapping and replacing it with the remapping's `path`.
    ///
    /// There are some caveats though, dapptools style remappings usually include the `src` folder
    /// `ds-test/=lib/ds-test/src/` so that imports look like `import "ds-test/test.sol";` (note the
    /// missing `src` in the import path).
    ///
    /// For hardhat/npm style that's not always the case, most notably for [openzeppelin-contracts](https://github.com/OpenZeppelin/openzeppelin-contracts) if installed via npm.
    /// The remapping is detected as `'@openzeppelin/=node_modules/@openzeppelin/contracts/'`, which
    /// includes the source directory `contracts`, however it's common to see import paths like:
    ///
    /// `import "@openzeppelin/contracts/token/ERC20/IERC20.sol";`
    ///
    /// instead of
    ///
    /// `import "@openzeppelin/token/ERC20/IERC20.sol";`
    ///
    /// There is no strict rule behind this, but because [`crate::remappings::Remapping::find_many`]
    /// returns `'@openzeppelin/=node_modules/@openzeppelin/contracts/'` we should handle the
    /// case if the remapping path ends with `contracts` and the import path starts with
    /// `<remapping name>/contracts`. Otherwise we can end up with a resolved path that has a
    /// duplicate `contracts` segment:
    /// `@openzeppelin/contracts/contracts/token/ERC20/IERC20.sol` we check for this edge case
    /// here so that both styles work out of the box.
    pub fn resolve_library_import(&self, cwd: &Path, import: &Path) -> Option<PathBuf> {
        // if the import path starts with the name of the remapping then we get the resolved path by
        // removing the name and adding the remainder to the path of the remapping
        let cwd = cwd.strip_prefix(&self.root).unwrap_or(cwd);
        if let Some(path) = self
            .remappings
            .iter()
            .filter(|r| {
                // only check remappings that are either global or for `cwd`
                if let Some(ctx) = r.context.as_ref() {
                    cwd.starts_with(ctx)
                } else {
                    true
                }
            })
            .find_map(|r| {
                import.strip_prefix(&r.name).ok().map(|stripped_import| {
                    let lib_path = Path::new(&r.path).join(stripped_import);

                    // we handle the edge case where the path of a remapping ends with "contracts"
                    // (`<name>/=.../contracts`) and the stripped import also starts with
                    // `contracts`
                    if let Ok(adjusted_import) = stripped_import.strip_prefix("contracts/") {
                        if r.path.ends_with("contracts/") && !lib_path.exists() {
                            return Path::new(&r.path).join(adjusted_import)
                        }
                    }
                    lib_path
                })
            })
        {
            Some(self.root.join(path))
        } else {
            utils::resolve_library(&self.libraries, import)
        }
    }

    /// Attempts to autodetect the artifacts directory based on the given root path
    ///
    /// Dapptools layout takes precedence over hardhat style.
    /// This will return:
    ///   - `<root>/out` if it exists or `<root>/artifacts` does not exist,
    ///   - `<root>/artifacts` if it exists and `<root>/out` does not exist.
    pub fn find_artifacts_dir(root: impl AsRef<Path>) -> PathBuf {
        utils::find_fave_or_alt_path(root, "out", "artifacts")
    }

    /// Attempts to autodetect the source directory based on the given root path
    ///
    /// Dapptools layout takes precedence over hardhat style.
    /// This will return:
    ///   - `<root>/src` if it exists or `<root>/contracts` does not exist,
    ///   - `<root>/contracts` if it exists and `<root>/src` does not exist.
    pub fn find_source_dir(root: impl AsRef<Path>) -> PathBuf {
        utils::find_fave_or_alt_path(root, "src", "contracts")
    }

    /// Attempts to autodetect the lib directory based on the given root path
    ///
    /// Dapptools layout takes precedence over hardhat style.
    /// This will return:
    ///   - `<root>/lib` if it exists or `<root>/node_modules` does not exist,
    ///   - `<root>/node_modules` if it exists and `<root>/lib` does not exist.
    pub fn find_libs(root: impl AsRef<Path>) -> Vec<PathBuf> {
        vec![utils::find_fave_or_alt_path(root, "lib", "node_modules")]
    }

    /// Flattens all file imports into a single string
    pub fn flatten(&self, target: &Path) -> Result<String> {
        tracing::trace!("flattening file");
        let mut input_files = self.input_files();

        // we need to ensure that the target is part of the input set, otherwise it's not
        // part of the graph if it's not imported by any input file
        let flatten_target = target.to_path_buf();
        if !input_files.contains(&flatten_target) {
            input_files.push(flatten_target);
        }

        let sources = Source::read_all_files(input_files)?;
        let graph = Graph::resolve_sources(self, sources)?;
        self.flatten_node(target, &graph, &mut Default::default(), false, false, false).map(|x| {
            format!("{}\n", utils::RE_THREE_OR_MORE_NEWLINES.replace_all(&x, "\n\n").trim())
        })
    }

    /// Flattens a single node from the dependency graph
    fn flatten_node(
        &self,
        target: &Path,
        graph: &Graph,
        imported: &mut HashSet<usize>,
        strip_version_pragma: bool,
        strip_experimental_pragma: bool,
        strip_license: bool,
    ) -> Result<String> {
        let target_dir = target.parent().ok_or_else(|| {
            SolcError::msg(format!("failed to get parent directory for \"{:?}\"", target.display()))
        })?;
        let target_index = graph.files().get(target).ok_or_else(|| {
            SolcError::msg(format!("cannot resolve file at {:?}", target.display()))
        })?;

        if imported.contains(target_index) {
            // short circuit nodes that were already imported, if both A.sol and B.sol import C.sol
            return Ok(String::new())
        }
        imported.insert(*target_index);

        let target_node = graph.node(*target_index);

        let mut imports = target_node.imports().clone();
        imports.sort_by_key(|x| x.loc().start);

        let mut content = target_node.content().to_owned();

        for alias in imports.iter().flat_map(|i| i.data().aliases()) {
            let (alias, target) = match alias {
                SolImportAlias::Contract(alias, target) => (alias.clone(), target.clone()),
                _ => continue,
            };
            let name_regex = utils::create_contract_or_lib_name_regex(&alias);
            let target_len = target.len() as isize;
            let mut replace_offset = 0;
            for cap in name_regex.captures_iter(&content.clone()) {
                if cap.name("ignore").is_some() {
                    continue
                }
                if let Some(name_match) = ["n1", "n2", "n3"].iter().find_map(|name| cap.name(name))
                {
                    let name_match_range =
                        utils::range_by_offset(&name_match.range(), replace_offset);
                    replace_offset += target_len - (name_match_range.len() as isize);
                    content.replace_range(name_match_range, &target);
                }
            }
        }

        let mut content = content.as_bytes().to_vec();
        let mut offset = 0_isize;

        let mut statements = [
            (target_node.license(), strip_license),
            (target_node.version(), strip_version_pragma),
            (target_node.experimental(), strip_experimental_pragma),
        ]
        .iter()
        .filter_map(|(data, condition)| if *condition { data.to_owned().as_ref() } else { None })
        .collect::<Vec<_>>();
        statements.sort_by_key(|x| x.loc().start);

        let (mut imports, mut statements) =
            (imports.iter().peekable(), statements.iter().peekable());
        while imports.peek().is_some() || statements.peek().is_some() {
            let (next_import_start, next_statement_start) = (
                imports.peek().map_or(usize::max_value(), |x| x.loc().start),
                statements.peek().map_or(usize::max_value(), |x| x.loc().start),
            );
            if next_statement_start < next_import_start {
                let repl_range = statements.next().unwrap().loc_by_offset(offset);
                offset -= repl_range.len() as isize;
                content.splice(repl_range, std::iter::empty());
            } else {
                let import = imports.next().unwrap();
                let import_path = self.resolve_import(target_dir, import.data().path())?;
                let s = self.flatten_node(&import_path, graph, imported, true, true, true)?;

                let import_content = s.as_bytes();
                let import_content_len = import_content.len() as isize;
                let import_range = import.loc_by_offset(offset);
                offset += import_content_len - (import_range.len() as isize);
                content.splice(import_range, import_content.iter().copied());
            }
        }

        let result = String::from_utf8(content).map_err(|err| {
            SolcError::msg(format!("failed to convert extended bytes to string: {err}"))
        })?;

        Ok(result)
    }
}

impl fmt::Display for ProjectPathsConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "root: {}", self.root.display())?;
        writeln!(f, "contracts: {}", self.sources.display())?;
        writeln!(f, "artifacts: {}", self.artifacts.display())?;
        writeln!(f, "tests: {}", self.tests.display())?;
        writeln!(f, "scripts: {}", self.scripts.display())?;
        writeln!(f, "libs:")?;
        for lib in &self.libraries {
            writeln!(f, "    {}", lib.display())?;
        }
        writeln!(f, "remappings:")?;
        for remapping in &self.remappings {
            writeln!(f, "    {remapping}")?;
        }
        Ok(())
    }
}

/// This is a subset of [ProjectPathsConfig] that contains all relevant folders in the project
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProjectPaths {
    pub artifacts: PathBuf,
    pub build_infos: PathBuf,
    pub sources: PathBuf,
    pub tests: PathBuf,
    pub scripts: PathBuf,
    pub libraries: BTreeSet<PathBuf>,
}

impl ProjectPaths {
    /// Joins the folders' location with `root`
    pub fn join_all(&mut self, root: impl AsRef<Path>) -> &mut Self {
        let root = root.as_ref();
        self.artifacts = root.join(&self.artifacts);
        self.build_infos = root.join(&self.build_infos);
        self.sources = root.join(&self.sources);
        self.tests = root.join(&self.tests);
        self.scripts = root.join(&self.scripts);
        let libraries = std::mem::take(&mut self.libraries);
        self.libraries.extend(libraries.into_iter().map(|p| root.join(p)));
        self
    }

    /// Removes `base` from all folders
    pub fn strip_prefix_all(&mut self, base: impl AsRef<Path>) -> &mut Self {
        let base = base.as_ref();

        if let Ok(prefix) = self.artifacts.strip_prefix(base) {
            self.artifacts = prefix.to_path_buf();
        }
        if let Ok(prefix) = self.build_infos.strip_prefix(base) {
            self.build_infos = prefix.to_path_buf();
        }
        if let Ok(prefix) = self.sources.strip_prefix(base) {
            self.sources = prefix.to_path_buf();
        }
        if let Ok(prefix) = self.tests.strip_prefix(base) {
            self.tests = prefix.to_path_buf();
        }
        if let Ok(prefix) = self.scripts.strip_prefix(base) {
            self.scripts = prefix.to_path_buf();
        }
        let libraries = std::mem::take(&mut self.libraries);
        self.libraries.extend(
            libraries
                .into_iter()
                .map(|p| p.strip_prefix(base).map(|p| p.to_path_buf()).unwrap_or(p)),
        );
        self
    }
}

impl Default for ProjectPaths {
    fn default() -> Self {
        Self {
            artifacts: "out".into(),
            build_infos: ["out", "build-info"].iter().collect::<PathBuf>(),
            sources: "src".into(),
            tests: "test".into(),
            scripts: "script".into(),
            libraries: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PathStyle {
    HardHat,
    Dapptools,
}

impl PathStyle {
    /// Convert into a `ProjectPathsConfig` given the root path and based on the styled
    pub fn paths(&self, root: impl AsRef<Path>) -> Result<ProjectPathsConfig> {
        let root = root.as_ref();
        let root = utils::canonicalize(root)?;

        Ok(match self {
            PathStyle::Dapptools => ProjectPathsConfig::builder()
                .sources(root.join("src"))
                .artifacts(root.join("out"))
                .build_infos(root.join("out").join("build-info"))
                .lib(root.join("lib"))
                .remappings(Remapping::find_many(root.join("lib")))
                .root(root)
                .build()?,
            PathStyle::HardHat => ProjectPathsConfig::builder()
                .sources(root.join("contracts"))
                .artifacts(root.join("artifacts"))
                .build_infos(root.join("artifacts").join("build-info"))
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
    build_infos: Option<PathBuf>,
    sources: Option<PathBuf>,
    tests: Option<PathBuf>,
    scripts: Option<PathBuf>,
    libraries: Option<Vec<PathBuf>>,
    remappings: Option<Vec<Remapping>>,
}

impl ProjectPathsConfigBuilder {
    pub fn root(mut self, root: impl Into<PathBuf>) -> Self {
        self.root = Some(utils::canonicalized(root));
        self
    }

    pub fn cache(mut self, cache: impl Into<PathBuf>) -> Self {
        self.cache = Some(utils::canonicalized(cache));
        self
    }

    pub fn artifacts(mut self, artifacts: impl Into<PathBuf>) -> Self {
        self.artifacts = Some(utils::canonicalized(artifacts));
        self
    }

    pub fn build_infos(mut self, build_infos: impl Into<PathBuf>) -> Self {
        self.build_infos = Some(utils::canonicalized(build_infos));
        self
    }

    pub fn sources(mut self, sources: impl Into<PathBuf>) -> Self {
        self.sources = Some(utils::canonicalized(sources));
        self
    }

    pub fn tests(mut self, tests: impl Into<PathBuf>) -> Self {
        self.tests = Some(utils::canonicalized(tests));
        self
    }

    pub fn scripts(mut self, scripts: impl Into<PathBuf>) -> Self {
        self.scripts = Some(utils::canonicalized(scripts));
        self
    }

    /// Specifically disallow additional libraries
    pub fn no_libs(mut self) -> Self {
        self.libraries = Some(Vec::new());
        self
    }

    pub fn lib(mut self, lib: impl Into<PathBuf>) -> Self {
        self.libraries.get_or_insert_with(Vec::new).push(utils::canonicalized(lib));
        self
    }

    pub fn libs(mut self, libs: impl IntoIterator<Item = impl Into<PathBuf>>) -> Self {
        let libraries = self.libraries.get_or_insert_with(Vec::new);
        for lib in libs.into_iter() {
            libraries.push(utils::canonicalized(lib));
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
        let root = utils::canonicalized(root);

        let libraries = self.libraries.unwrap_or_else(|| ProjectPathsConfig::find_libs(&root));
        let artifacts =
            self.artifacts.unwrap_or_else(|| ProjectPathsConfig::find_artifacts_dir(&root));

        ProjectPathsConfig {
            cache: self
                .cache
                .unwrap_or_else(|| root.join("cache").join(SOLIDITY_FILES_CACHE_FILENAME)),
            build_infos: self.build_infos.unwrap_or_else(|| artifacts.join("build-info")),
            artifacts,
            sources: self.sources.unwrap_or_else(|| ProjectPathsConfig::find_source_dir(&root)),
            tests: self.tests.unwrap_or_else(|| root.join("test")),
            scripts: self.scripts.unwrap_or_else(|| root.join("script")),
            remappings: self
                .remappings
                .unwrap_or_else(|| libraries.iter().flat_map(Remapping::find_many).collect()),
            libraries,
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
    /// let config = SolcConfig::builder().build();
    /// ```
    pub fn builder() -> SolcConfigBuilder {
        SolcConfigBuilder::default()
    }
}

impl From<SolcConfig> for Settings {
    fn from(config: SolcConfig) -> Self {
        config.settings
    }
}

#[derive(Default)]
pub struct SolcConfigBuilder {
    settings: Option<Settings>,

    /// additionally selected outputs that should be included in the `Contract` that `solc´ creates
    output_selection: Vec<ContractOutputSelection>,
}

impl SolcConfigBuilder {
    pub fn settings(mut self, settings: Settings) -> Self {
        self.settings = Some(settings);
        self
    }

    /// Adds another `ContractOutputSelection` to the set
    #[must_use]
    pub fn additional_output(mut self, output: impl Into<ContractOutputSelection>) -> Self {
        self.output_selection.push(output.into());
        self
    }

    /// Adds multiple `ContractOutputSelection` to the set
    #[must_use]
    pub fn additional_outputs<I, S>(mut self, outputs: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<ContractOutputSelection>,
    {
        for out in outputs {
            self = self.additional_output(out);
        }
        self
    }

    /// Creates the solc config
    ///
    /// If no solc version is configured then it will be determined by calling `solc --version`.
    pub fn build(self) -> SolcConfig {
        let Self { settings, output_selection } = self;
        let mut settings = settings.unwrap_or_default();
        settings.push_all(output_selection);
        SolcConfig { settings }
    }
}

/// Container for all `--include-path` arguments for Solc, see also
/// [Solc docs](https://docs.soliditylang.org/en/v0.8.9/using-the-compiler.html#base-path-and-import-remapping).
///
/// The `--include--path` flag:
/// > Makes an additional source directory available to the default import callback. Use this option
/// > if you want to import contracts whose location is not fixed in relation to your main source
/// > tree, e.g. third-party libraries installed using a package manager. Can be used multiple
/// > times. Can only be used if base path has a non-empty value.
///
/// In contrast to `--allow-paths` [`AllowedLibPaths`], which takes multiple arguments,
/// `--include-path` only takes a single path argument.
#[derive(Clone, Debug, Default)]
pub struct IncludePaths(pub(crate) BTreeSet<PathBuf>);

// === impl IncludePaths ===

impl IncludePaths {
    /// Returns the [Command](std::process::Command) arguments for this type
    ///
    /// For each entry in the set, it will return `--include-path` + `<entry>`
    pub fn args(&self) -> impl Iterator<Item = String> + '_ {
        self.paths().flat_map(|path| ["--include-path".to_string(), format!("{}", path.display())])
    }

    /// Returns all paths that exist
    pub fn paths(&self) -> impl Iterator<Item = &PathBuf> + '_ {
        self.0.iter().filter(|path| path.exists())
    }
}

impl Deref for IncludePaths {
    type Target = BTreeSet<PathBuf>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for IncludePaths {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
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
pub struct AllowedLibPaths(pub(crate) BTreeSet<PathBuf>);

// === impl AllowedLibPaths ===

impl AllowedLibPaths {
    /// Returns the [Command](std::process::Command) arguments for this type
    ///
    /// `--allow-paths` takes a single value: all comma separated paths
    pub fn args(&self) -> Option<[String; 2]> {
        let args = self.to_string();
        if args.is_empty() {
            return None
        }
        Some(["--allow-paths".to_string(), args])
    }

    /// Returns all paths that exist
    pub fn paths(&self) -> impl Iterator<Item = &PathBuf> + '_ {
        self.0.iter().filter(|path| path.exists())
    }
}

impl Deref for AllowedLibPaths {
    type Target = BTreeSet<PathBuf>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AllowedLibPaths {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl fmt::Display for AllowedLibPaths {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let lib_paths =
            self.paths().map(|path| format!("{}", path.display())).collect::<Vec<_>>().join(",");
        write!(f, "{lib_paths}")
    }
}

impl<T: Into<PathBuf>> From<Vec<T>> for AllowedLibPaths {
    fn from(libs: Vec<T>) -> Self {
        let libs = libs.into_iter().map(utils::canonicalized).collect();
        AllowedLibPaths(libs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_autodetect_dirs() {
        let root = utils::tempdir("root").unwrap();
        let out = root.path().join("out");
        let artifacts = root.path().join("artifacts");
        let build_infos = artifacts.join("build-info");
        let contracts = root.path().join("contracts");
        let src = root.path().join("src");
        let lib = root.path().join("lib");
        let node_modules = root.path().join("node_modules");

        let root = root.path();
        assert_eq!(ProjectPathsConfig::find_source_dir(root), src,);
        std::fs::create_dir_all(&contracts).unwrap();
        assert_eq!(ProjectPathsConfig::find_source_dir(root), contracts,);
        assert_eq!(
            ProjectPathsConfig::builder().build_with_root(root).sources,
            utils::canonicalized(contracts),
        );
        std::fs::create_dir_all(&src).unwrap();
        assert_eq!(ProjectPathsConfig::find_source_dir(root), src,);
        assert_eq!(
            ProjectPathsConfig::builder().build_with_root(root).sources,
            utils::canonicalized(src),
        );

        assert_eq!(ProjectPathsConfig::find_artifacts_dir(root), out,);
        std::fs::create_dir_all(&artifacts).unwrap();
        assert_eq!(ProjectPathsConfig::find_artifacts_dir(root), artifacts,);
        assert_eq!(
            ProjectPathsConfig::builder().build_with_root(root).artifacts,
            utils::canonicalized(artifacts),
        );
        std::fs::create_dir_all(&build_infos).unwrap();
        assert_eq!(
            ProjectPathsConfig::builder().build_with_root(root).build_infos,
            utils::canonicalized(build_infos)
        );

        std::fs::create_dir_all(&out).unwrap();
        assert_eq!(ProjectPathsConfig::find_artifacts_dir(root), out,);
        assert_eq!(
            ProjectPathsConfig::builder().build_with_root(root).artifacts,
            utils::canonicalized(out),
        );

        assert_eq!(ProjectPathsConfig::find_libs(root), vec![lib.clone()],);
        std::fs::create_dir_all(&node_modules).unwrap();
        assert_eq!(ProjectPathsConfig::find_libs(root), vec![node_modules.clone()],);
        assert_eq!(
            ProjectPathsConfig::builder().build_with_root(root).libraries,
            vec![utils::canonicalized(node_modules)],
        );
        std::fs::create_dir_all(&lib).unwrap();
        assert_eq!(ProjectPathsConfig::find_libs(root), vec![lib.clone()],);
        assert_eq!(
            ProjectPathsConfig::builder().build_with_root(root).libraries,
            vec![utils::canonicalized(lib)],
        );
    }

    #[test]
    fn can_have_sane_build_info_default() {
        let root = utils::tempdir("root").unwrap();
        let root = root.path();
        let artifacts = root.join("forge-artifacts");

        // Set the artifacts directory without setting the
        // build info directory
        let project = ProjectPathsConfig::builder().artifacts(&artifacts).build_with_root(root);

        // The artifacts should be set correctly based on the configured value
        assert_eq!(project.artifacts, utils::canonicalized(artifacts));

        // The build infos should by default in the artifacts directory
        assert_eq!(project.build_infos, utils::canonicalized(project.artifacts.join("build-info")));
    }

    #[test]
    #[cfg_attr(windows, ignore = "Windows remappings #2347")]
    fn can_find_library_ancestor() {
        let mut config = ProjectPathsConfig::builder().lib("lib").build().unwrap();
        config.root = "/root/".into();

        assert_eq!(config.find_library_ancestor("lib/src/Greeter.sol").unwrap(), Path::new("lib"));

        assert_eq!(
            config.find_library_ancestor("/root/lib/src/Greeter.sol").unwrap(),
            Path::new("lib")
        );

        config.libraries.push("/root/test/".into());

        assert_eq!(
            config.find_library_ancestor("test/src/Greeter.sol").unwrap(),
            Path::new("/root/test/")
        );

        assert_eq!(
            config.find_library_ancestor("/root/test/src/Greeter.sol").unwrap(),
            Path::new("/root/test/")
        );
    }
}
