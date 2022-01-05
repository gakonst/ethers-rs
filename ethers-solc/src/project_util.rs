//! Utilities for mocking project workspaces
use crate::{
    config::ProjectPathsConfigBuilder,
    error::{Result, SolcError},
    hh::HardhatArtifacts,
    ArtifactOutput, MinimalCombinedArtifacts, Project, ProjectCompileOutput, ProjectPathsConfig,
    SolcIoError,
};
use fs_extra::{dir, file};
use std::path::{Path, PathBuf};
use tempdir::TempDir;

pub struct TempProject<T: ArtifactOutput> {
    /// temporary workspace root
    _root: TempDir,
    /// actual project workspace with the `root` tempdir as its root
    inner: Project<T>,
}

impl<T: ArtifactOutput> TempProject<T> {
    /// Makes sure all resources are created
    fn create_new(root: TempDir, inner: Project<T>) -> std::result::Result<Self, SolcIoError> {
        let project = Self { _root: root, inner };
        project.paths().create_all()?;
        Ok(project)
    }

    pub fn new(paths: ProjectPathsConfigBuilder) -> Result<Self> {
        let tmp_dir = TempDir::new("root").map_err(|err| SolcError::io(err, "root"))?;
        let paths = paths.build_with_root(tmp_dir.path());
        let inner = Project::builder().artifacts().paths(paths).build()?;
        Ok(Self::create_new(tmp_dir, inner)?)
    }

    pub fn project(&self) -> &Project<T> {
        &self.inner
    }

    pub fn compile(&self) -> Result<ProjectCompileOutput<T>> {
        self.project().compile()
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

    /// Adds a new source file
    pub fn add_source(&self, name: impl AsRef<str>, content: impl AsRef<str>) -> Result<PathBuf> {
        let name = contract_file_name(name);
        let source = self.paths().sources.join(name);
        create_contract_file(source, content)
    }
}

fn create_contract_file(path: PathBuf, content: impl AsRef<str>) -> Result<PathBuf> {
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
        let tmp_dir = TempDir::new("tmp_hh").map_err(|err| SolcError::io(err, "tmp_hh"))?;

        let paths = ProjectPathsConfig::hardhat(tmp_dir.path())?;

        let inner = Project::builder().artifacts().paths(paths).build()?;
        Ok(Self::create_new(tmp_dir, inner)?)
    }
}

impl TempProject<MinimalCombinedArtifacts> {
    /// Creates an empty new dapptools style workspace in a new temporary dir
    pub fn dapptools() -> Result<Self> {
        let tmp_dir = TempDir::new("tmp_dapp").map_err(|err| SolcError::io(err, "temp_dapp"))?;
        let paths = ProjectPathsConfig::dapptools(tmp_dir.path())?;

        let inner = Project::builder().artifacts().paths(paths).build()?;
        Ok(Self::create_new(tmp_dir, inner)?)
    }
}

impl<T: ArtifactOutput> AsRef<Project<T>> for TempProject<T> {
    fn as_ref(&self) -> &Project<T> {
        self.project()
    }
}

fn dir_copy_options() -> dir::CopyOptions {
    dir::CopyOptions {
        overwrite: true,
        skip_exist: false,
        buffer_size: 64000, //64kb
        copy_inside: true,
        content_only: false,
        depth: 0,
    }
}

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
