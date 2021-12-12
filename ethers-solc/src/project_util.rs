//! Utilities for mocking project workspaces
use crate::{
    cache::SOLIDITY_FILES_CACHE_FILENAME, config::ProjectPathsConfigBuilder, hh::HardhatArtifacts,
    ArtifactOutput, MinimalCombinedArtifacts, Project, ProjectCompileOutput, ProjectPathsConfig,
};

use crate::remappings::Remapping;
use fs_extra::{dir, file};
use std::{io, path::Path};
use tempdir::TempDir;

pub struct TempProject<T: ArtifactOutput> {
    /// temporary workspace root
    root: TempDir,
    /// actual project workspace with the `root` tempdir as its root
    inner: Project<T>,
}

impl<T: ArtifactOutput> TempProject<T> {
    /// Makes sure all resources are created
    fn create_new(root: TempDir, inner: Project<T>) -> io::Result<Self> {
        let project = Self { root, inner };
        project.paths().create_all()?;
        Ok(project)
    }

    pub fn new(paths: ProjectPathsConfigBuilder) -> eyre::Result<Self> {
        let tmp_dir = TempDir::new("root")?;
        let paths = paths.build_with_root(tmp_dir.path());
        let inner = Project::builder().artifacts().paths(paths).build()?;
        Ok(Self::create_new(tmp_dir, inner)?)
    }

    pub fn project(&self) -> &Project<T> {
        &self.inner
    }

    pub fn compile(&self) -> eyre::Result<ProjectCompileOutput<T>> {
        Ok(self.project().compile()?)
    }

    pub fn project_mut(&mut self) -> &mut Project<T> {
        &mut self.inner
    }

    /// The configured paths of the project
    pub fn paths(&self) -> &ProjectPathsConfig {
        &self.project().paths
    }

    /// The root path of the temporary workspace
    pub fn root(&self) -> &Path {
        self.project().paths.root.as_path()
    }

    /// Returns the handle to the tempdir and the project
    ///
    /// NOTE: the `TempDir` object deletes its directory on drop, also removing all the project's
    /// content
    pub fn split(self) -> (Project<T>, TempDir) {
        (self.inner, self.root)
    }

    /// Copies a single file into the given dir
    pub fn copy_file(
        &self,
        source: impl AsRef<Path>,
        target_dir: impl AsRef<Path>,
    ) -> eyre::Result<()> {
        let source = source.as_ref();
        let target = target_dir.as_ref().join(
            source
                .file_name()
                .ok_or_else(|| eyre::eyre!("No file name for {}", source.display()))?,
        );

        fs_extra::file::copy(source, target, &file_copy_options())?;
        Ok(())
    }

    /// Copies all content of the source dir into the target dir
    pub fn copy_dir(
        &self,
        source: impl AsRef<Path>,
        target_dir: impl AsRef<Path>,
    ) -> eyre::Result<()> {
        fs_extra::dir::copy(source, target_dir, &dir_copy_options())?;
        Ok(())
    }

    /// Copies a single file into the projects source
    pub fn copy_source(&self, source: impl AsRef<Path>) -> eyre::Result<()> {
        self.copy_file(source, &self.paths().sources)
    }

    pub fn copy_sources<I, S>(&self, sources: I) -> eyre::Result<()>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<Path>,
    {
        for path in sources {
            self.copy_source(path)?;
        }
        Ok(())
    }

    /// Copies a single file into the project's main library directory
    pub fn copy_lib(&self, lib: impl AsRef<Path>) -> eyre::Result<()> {
        let lib_dir = self
            .paths()
            .libraries
            .get(0)
            .ok_or_else(|| eyre::eyre!("No libraries folders configured"))?;
        self.copy_file(lib, lib_dir)
    }

    /// Copy a series of files into the main library dir
    pub fn copy_libs<I, S>(&self, libs: I) -> eyre::Result<()>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<Path>,
    {
        for path in libs {
            self.copy_lib(path)?;
        }
        Ok(())
    }
}

impl TempProject<HardhatArtifacts> {
    /// Creates an empty new hardhat style workspace in a new temporary dir
    pub fn hardhat() -> eyre::Result<Self> {
        let tmp_dir = TempDir::new("tmp_hh")?;
        let root = tmp_dir.path().to_path_buf();
        let cache = tmp_dir.path().join("cache");
        let cache = cache.join(SOLIDITY_FILES_CACHE_FILENAME);

        let paths = ProjectPathsConfig::builder()
            .cache(cache)
            .sources(root.join("contracts"))
            .artifacts(root.join("artifacts"))
            .lib(root.join("node_modules"))
            .root(root)
            .build()?;

        let inner = Project::builder().artifacts().paths(paths).build()?;
        Ok(Self::create_new(tmp_dir, inner)?)
    }
}

impl TempProject<MinimalCombinedArtifacts> {
    /// Creates an empty new dapptools style workspace in a new temporary dir
    pub fn dapptools() -> eyre::Result<Self> {
        let tmp_dir = TempDir::new("tmp_dapp")?;
        let root = tmp_dir.path().to_path_buf();
        let cache = tmp_dir.path().join("cache");
        let cache = cache.join(SOLIDITY_FILES_CACHE_FILENAME);

        let paths = ProjectPathsConfig::builder()
            .cache(cache)
            .sources(root.join("src"))
            .artifacts(root.join("out"))
            .lib(root.join("lib"))
            .remappings(Remapping::find_many(&root.join("lib"))?)
            .root(root)
            .build()?;

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
