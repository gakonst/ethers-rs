//! Utilities for mocking project workspaces
use crate::{
    artifacts::Settings,
    config::ProjectPathsConfigBuilder,
    error::{bail, Result, SolcError},
    hh::HardhatArtifacts,
    project_util::mock::{MockProjectGenerator, MockProjectSettings},
    utils::tempdir,
    ArtifactOutput, Artifacts, ConfigurableArtifacts, PathStyle, Project, ProjectCompileOutput,
    ProjectPathsConfig, SolFilesCache, SolcIoError,
};
use fs_extra::{dir, file};
use std::{
    fmt,
    path::{Path, PathBuf},
};
use tempfile::TempDir;

pub mod mock;

/// A [`Project`] wrapper that lives in a new temporary directory
///
/// Once `TempProject` is dropped, the temp dir is automatically removed, see [`TempDir::drop()`]
pub struct TempProject<T: ArtifactOutput = ConfigurableArtifacts> {
    /// temporary workspace root
    _root: TempDir,
    /// actual project workspace with the `root` tempdir as its root
    inner: Project<T>,
}

impl<T: ArtifactOutput> TempProject<T> {
    /// Makes sure all resources are created
    pub fn create_new(root: TempDir, inner: Project<T>) -> std::result::Result<Self, SolcIoError> {
        let project = Self { _root: root, inner };
        project.paths().create_all()?;
        Ok(project)
    }

    /// Creates a new temp project using the provided paths and artifacts handler.
    /// sets the project root to a temp dir
    pub fn with_artifacts(paths: ProjectPathsConfigBuilder, artifacts: T) -> Result<Self> {
        Self::prefixed_with_artifacts("temp-project", paths, artifacts)
    }

    /// Creates a new temp project inside a tempdir with a prefixed directory and the given
    /// artifacts handler
    pub fn prefixed_with_artifacts(
        prefix: &str,
        paths: ProjectPathsConfigBuilder,
        artifacts: T,
    ) -> Result<Self> {
        let tmp_dir = tempdir(prefix)?;
        let paths = paths.build_with_root(tmp_dir.path());
        let inner = Project::builder().artifacts(artifacts).paths(paths).build()?;
        Ok(Self::create_new(tmp_dir, inner)?)
    }

    /// Overwrites the settings to pass to `solc`
    pub fn with_settings(mut self, settings: impl Into<Settings>) -> Self {
        self.inner.solc_config.settings = settings.into();
        self
    }

    pub fn project(&self) -> &Project<T> {
        &self.inner
    }

    pub fn compile(&self) -> Result<ProjectCompileOutput<T>> {
        self.project().compile()
    }

    pub fn flatten(&self, target: &Path) -> Result<String> {
        self.project().flatten(target)
    }

    pub fn project_mut(&mut self) -> &mut Project<T> {
        &mut self.inner
    }

    /// The configured paths of the project
    pub fn paths(&self) -> &ProjectPathsConfig {
        &self.project().paths
    }

    /// The configured paths of the project
    pub fn paths_mut(&mut self) -> &mut ProjectPathsConfig {
        &mut self.project_mut().paths
    }

    /// Returns the path to the artifacts directory
    pub fn artifacts_path(&self) -> &PathBuf {
        &self.paths().artifacts
    }

    /// Returns the path to the sources directory
    pub fn sources_path(&self) -> &PathBuf {
        &self.paths().sources
    }

    /// Returns the path to the cache file
    pub fn cache_path(&self) -> &PathBuf {
        &self.paths().cache
    }

    /// The root path of the temporary workspace
    pub fn root(&self) -> &Path {
        self.project().paths.root.as_path()
    }

    /// Copies a single file into the projects source
    pub fn copy_source(&self, source: impl AsRef<Path>) -> Result<()> {
        copy_file(source, &self.paths().sources)
    }

    pub fn copy_sources<I, S>(&self, sources: I) -> Result<()>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<Path>,
    {
        for path in sources {
            self.copy_source(path)?;
        }
        Ok(())
    }

    fn get_lib(&self) -> Result<PathBuf> {
        self.paths()
            .libraries
            .get(0)
            .cloned()
            .ok_or_else(|| SolcError::msg("No libraries folders configured"))
    }

    /// Copies a single file into the project's main library directory
    pub fn copy_lib(&self, lib: impl AsRef<Path>) -> Result<()> {
        let lib_dir = self.get_lib()?;
        copy_file(lib, lib_dir)
    }

    /// Copy a series of files into the main library dir
    pub fn copy_libs<I, S>(&self, libs: I) -> Result<()>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<Path>,
    {
        for path in libs {
            self.copy_lib(path)?;
        }
        Ok(())
    }

    /// Adds a new library file
    pub fn add_lib(&self, name: impl AsRef<str>, content: impl AsRef<str>) -> Result<PathBuf> {
        let name = contract_file_name(name);
        let lib_dir = self.get_lib()?;
        let lib = lib_dir.join(name);
        create_contract_file(lib, content)
    }

    /// Adds a basic lib contract `contract <name> {}` as a new file
    pub fn add_basic_lib(
        &self,
        name: impl AsRef<str>,
        version: impl AsRef<str>,
    ) -> Result<PathBuf> {
        let name = name.as_ref();
        self.add_lib(
            name,
            format!(
                r#"
// SPDX-License-Identifier: UNLICENSED
pragma solidity {};
contract {} {{}}
            "#,
                name,
                version.as_ref()
            ),
        )
    }

    /// Adds a new source file inside the project's source dir
    pub fn add_source(&self, name: impl AsRef<str>, content: impl AsRef<str>) -> Result<PathBuf> {
        let name = contract_file_name(name);
        let source = self.paths().sources.join(name);
        create_contract_file(source, content)
    }

    /// Adds a basic source contract `contract <name> {}` as a new file
    pub fn add_basic_source(
        &self,
        name: impl AsRef<str>,
        version: impl AsRef<str>,
    ) -> Result<PathBuf> {
        let name = name.as_ref();
        self.add_source(
            name,
            format!(
                r#"
// SPDX-License-Identifier: UNLICENSED
pragma solidity {};
contract {} {{}}
            "#,
                name,
                version.as_ref()
            ),
        )
    }

    /// Adds a solidity contract in the project's root dir.
    /// This will also create all intermediary dirs.
    pub fn add_contract(&self, name: impl AsRef<str>, content: impl AsRef<str>) -> Result<PathBuf> {
        let name = contract_file_name(name);
        let source = self.root().join(name);
        create_contract_file(source, content)
    }

    /// Returns a snapshot of all cached artifacts
    pub fn artifacts_snapshot(&self) -> Result<ArtifactsSnapshot> {
        let cache = self.project().read_cache_file()?;
        let artifacts = cache.read_artifacts::<ConfigurableArtifacts>()?;
        Ok(ArtifactsSnapshot { cache, artifacts })
    }

    /// Populate the project with mock files
    pub fn mock(&self, gen: &MockProjectGenerator, version: impl AsRef<str>) -> Result<()> {
        gen.write_to(self.paths(), version)
    }

    /// Compiles the project and ensures that the output does not contain errors
    pub fn ensure_no_errors(&self) -> Result<&Self> {
        let compiled = self.compile().unwrap();
        if compiled.has_compiler_errors() {
            bail!("Compiled with errors {}", compiled)
        }
        Ok(self)
    }

    /// Compiles the project and ensures that the output is __unchanged__
    pub fn ensure_unchanged(&self) -> Result<&Self> {
        let compiled = self.compile().unwrap();
        if !compiled.is_unchanged() {
            bail!("Compiled with detected changes {}", compiled)
        }
        Ok(self)
    }

    /// Compiles the project and ensures that the output has __changed__
    pub fn ensure_changed(&self) -> Result<&Self> {
        let compiled = self.compile().unwrap();
        if compiled.is_unchanged() {
            bail!("Compiled without detecting changes {}", compiled)
        }
        Ok(self)
    }

    /// Compiles the project and ensures that the output does not contain errors and no changes
    /// exists on recompiled.
    ///
    /// This is a convenience function for
    ///
    /// ```no_run
    /// use ethers_solc::project_util::TempProject;
    /// let project = TempProject::dapptools().unwrap();
    //  project.ensure_no_errors().unwrap();
    //  project.ensure_unchanged().unwrap();
    /// ```
    pub fn ensure_no_errors_recompile_unchanged(&self) -> Result<&Self> {
        self.ensure_no_errors()?.ensure_unchanged()
    }

    /// Compiles the project and asserts that the output does not contain errors and no changes
    /// exists on recompiled.
    ///
    /// This is a convenience function for
    ///
    /// ```no_run
    /// use ethers_solc::project_util::TempProject;
    /// let project = TempProject::dapptools().unwrap();
    //  project.assert_no_errors();
    //  project.assert_unchanged();
    /// ```
    pub fn assert_no_errors_recompile_unchanged(&self) -> &Self {
        self.assert_no_errors().assert_unchanged()
    }

    /// Compiles the project and asserts that the output does not contain errors
    pub fn assert_no_errors(&self) -> &Self {
        let compiled = self.compile().unwrap();
        assert!(!compiled.has_compiler_errors());
        self
    }

    /// Compiles the project and asserts that the output is unchanged
    pub fn assert_unchanged(&self) -> &Self {
        let compiled = self.compile().unwrap();
        assert!(compiled.is_unchanged());
        self
    }

    /// Compiles the project and asserts that the output is _changed_
    pub fn assert_changed(&self) -> &Self {
        let compiled = self.compile().unwrap();
        assert!(!compiled.is_unchanged());
        self
    }
}

impl<T: ArtifactOutput + Default> TempProject<T> {
    /// Creates a new temp project inside a tempdir with a prefixed directory
    pub fn prefixed(prefix: &str, paths: ProjectPathsConfigBuilder) -> Result<Self> {
        Self::prefixed_with_artifacts(prefix, paths, T::default())
    }

    /// Creates a new temp project for the given `PathStyle`
    pub fn with_style(prefix: &str, style: PathStyle) -> Result<Self> {
        let tmp_dir = tempdir(prefix)?;
        let paths = style.paths(tmp_dir.path())?;
        let inner = Project::builder().artifacts(T::default()).paths(paths).build()?;
        Ok(Self::create_new(tmp_dir, inner)?)
    }

    /// Creates a new temp project using the provided paths and setting the project root to a temp
    /// dir
    pub fn new(paths: ProjectPathsConfigBuilder) -> Result<Self> {
        Self::prefixed("temp-project", paths)
    }
}

impl<T: ArtifactOutput> fmt::Debug for TempProject<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TempProject").field("paths", self.paths()).finish()
    }
}

pub(crate) fn create_contract_file(path: PathBuf, content: impl AsRef<str>) -> Result<PathBuf> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|err| SolcIoError::new(err, parent.to_path_buf()))?;
    }
    std::fs::write(&path, content.as_ref()).map_err(|err| SolcIoError::new(err, path.clone()))?;
    Ok(path)
}

fn contract_file_name(name: impl AsRef<str>) -> String {
    let name = name.as_ref();
    if name.ends_with(".sol") {
        name.to_string()
    } else {
        format!("{}.sol", name)
    }
}

impl TempProject<HardhatArtifacts> {
    /// Creates an empty new hardhat style workspace in a new temporary dir
    pub fn hardhat() -> Result<Self> {
        let tmp_dir = tempdir("tmp_hh")?;

        let paths = ProjectPathsConfig::hardhat(tmp_dir.path())?;

        let inner =
            Project::builder().artifacts(HardhatArtifacts::default()).paths(paths).build()?;
        Ok(Self::create_new(tmp_dir, inner)?)
    }
}

impl TempProject<ConfigurableArtifacts> {
    /// Creates an empty new dapptools style workspace in a new temporary dir
    pub fn dapptools() -> Result<Self> {
        let tmp_dir = tempdir("tmp_dapp")?;
        let paths = ProjectPathsConfig::dapptools(tmp_dir.path())?;

        let inner = Project::builder().paths(paths).build()?;
        Ok(Self::create_new(tmp_dir, inner)?)
    }

    /// Create a new temporary project and populate it with mock files
    ///
    /// ```no_run
    /// use ethers_solc::project_util::mock::MockProjectSettings;
    /// use ethers_solc::project_util::TempProject;
    /// let tmp = TempProject::mocked(&MockProjectSettings::default(), "^0.8.10").unwrap();
    /// ```
    pub fn mocked(settings: &MockProjectSettings, version: impl AsRef<str>) -> Result<Self> {
        let mut tmp = Self::dapptools()?;
        let gen = MockProjectGenerator::new(settings);
        tmp.mock(&gen, version)?;
        let remappings = gen.remappings_at(tmp.root());
        tmp.paths_mut().remappings.extend(remappings);
        Ok(tmp)
    }

    /// Create a new temporary project and populate it with a random layout
    ///
    /// ```no_run
    /// use ethers_solc::project_util::TempProject;
    /// let tmp = TempProject::mocked_random("^0.8.10").unwrap();
    /// ```
    ///
    /// This is a convenience function for:
    ///
    /// ```no_run
    /// use ethers_solc::project_util::mock::MockProjectSettings;
    /// use ethers_solc::project_util::TempProject;
    /// let tmp = TempProject::mocked(&MockProjectSettings::random(), "^0.8.10").unwrap();
    /// ```
    pub fn mocked_random(version: impl AsRef<str>) -> Result<Self> {
        Self::mocked(&MockProjectSettings::random(), version)
    }
}

impl<T: ArtifactOutput> AsRef<Project<T>> for TempProject<T> {
    fn as_ref(&self) -> &Project<T> {
        self.project()
    }
}

/// The cache file and all the artifacts it references
#[derive(Debug, Clone)]
pub struct ArtifactsSnapshot {
    pub cache: SolFilesCache,
    pub artifacts: Artifacts<ConfigurableArtifacts>,
}

/// commonly used options for copying entire folders
fn dir_copy_options() -> dir::CopyOptions {
    dir::CopyOptions {
        overwrite: true,
        skip_exist: false,
        buffer_size: 64000, //64kb
        copy_inside: true,
        content_only: true,
        depth: 0,
    }
}

/// commonly used options for copying files
fn file_copy_options() -> file::CopyOptions {
    file::CopyOptions {
        overwrite: true,
        skip_exist: false,
        buffer_size: 64000, //64kb
    }
}

/// Copies a single file into the given dir
pub fn copy_file(source: impl AsRef<Path>, target_dir: impl AsRef<Path>) -> Result<()> {
    let source = source.as_ref();
    let target = target_dir.as_ref().join(
        source
            .file_name()
            .ok_or_else(|| SolcError::msg(format!("No file name for {}", source.display())))?,
    );

    fs_extra::file::copy(source, target, &file_copy_options())?;
    Ok(())
}

/// Copies all content of the source dir into the target dir
pub fn copy_dir(source: impl AsRef<Path>, target_dir: impl AsRef<Path>) -> Result<()> {
    fs_extra::dir::copy(source, target_dir, &dir_copy_options())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_mock_project() {
        let _prj = TempProject::mocked(&Default::default(), "^0.8.11").unwrap();
        let _prj = TempProject::mocked_random("^0.8.11").unwrap();
    }
}
