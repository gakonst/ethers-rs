use crate::SourceFile;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};

/// (source_file path  -> `SourceFile` + solc version)
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct VersionedSourceFiles(pub BTreeMap<String, Vec<VersionedSourceFile>>);

impl VersionedSourceFiles {
    /// Converts all `\\` separators in _all_ paths to `/`
    pub fn slash_paths(&mut self) {
        #[cfg(windows)]
        {
            use path_slash::PathExt;
            self.0 = std::mem::take(&mut self.0)
                .into_iter()
                .map(|(path, files)| (Path::new(&path).to_slash_lossy().to_string(), files))
                .collect()
        }
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns an iterator over all files
    pub fn files(&self) -> impl Iterator<Item = &String> + '_ {
        self.0.keys()
    }

    /// Returns an iterator over the source files' ids and path
    ///
    /// ```
    /// use std::collections::BTreeMap;
    /// use ethers_solc::sources::VersionedSourceFiles;
    /// # fn demo(files: VersionedSourceFiles) {
    /// let sources: BTreeMap<u32,String> = files.into_ids().collect();
    /// # }
    /// ```
    pub fn into_ids(self) -> impl Iterator<Item = (u32, String)> {
        self.into_sources().map(|(path, source)| (source.id, path))
    }

    /// Returns an iterator over the source files' paths and ids
    ///
    /// ```
    /// use std::collections::BTreeMap;
    /// use ethers_solc::artifacts::SourceFiles;
    /// # fn demo(files: SourceFiles) {
    /// let sources :BTreeMap<String, u32> = files.into_paths().collect();
    /// # }
    /// ```
    pub fn into_paths(self) -> impl Iterator<Item = (String, u32)> {
        self.into_ids().map(|(id, path)| (path, id))
    }

    /// Returns an iterator over the source files' ids and path
    ///
    /// ```
    /// use std::collections::BTreeMap;
    /// use semver::Version;
    /// use ethers_solc::sources::VersionedSourceFiles;
    /// # fn demo(files: VersionedSourceFiles) {
    /// let sources: BTreeMap<(u32, Version) ,String> = files.into_ids_with_version().map(|(id, source, version)|((id, version), source)).collect();
    /// # }
    /// ```
    pub fn into_ids_with_version(self) -> impl Iterator<Item = (u32, String, Version)> {
        self.into_sources_with_version().map(|(path, source, version)| (source.id, path, version))
    }

    /// Finds the _first_ source file with the given path
    ///
    /// # Example
    ///
    /// ```
    /// use ethers_solc::Project;
    /// use ethers_solc::artifacts::*;
    /// # fn demo(project: Project) {
    /// let output = project.compile().unwrap().output();
    /// let source_file = output.sources.find_file("src/Greeter.sol").unwrap();
    /// # }
    /// ```
    pub fn find_file(&self, source_file: impl AsRef<str>) -> Option<&SourceFile> {
        let source_file_name = source_file.as_ref();
        self.sources().find_map(
            |(path, source_file)| {
                if path == source_file_name {
                    Some(source_file)
                } else {
                    None
                }
            },
        )
    }

    /// Same as [Self::find_file] but also checks for version
    pub fn find_file_and_version(&self, path: &str, version: &Version) -> Option<&SourceFile> {
        self.0.get(path).and_then(|contracts| {
            contracts.iter().find_map(|source| {
                if source.version == *version {
                    Some(&source.source_file)
                } else {
                    None
                }
            })
        })
    }

    /// Finds the _first_ source file with the given id
    ///
    /// # Example
    ///
    /// ```
    /// use ethers_solc::Project;
    /// use ethers_solc::artifacts::*;
    /// # fn demo(project: Project) {
    /// let output = project.compile().unwrap().output();
    /// let source_file = output.sources.find_id(0).unwrap();
    /// # }
    /// ```
    pub fn find_id(&self, id: u32) -> Option<&SourceFile> {
        self.sources().filter(|(_, source)| source.id == id).map(|(_, source)| source).next()
    }

    /// Same as [Self::find_id] but also checks for version
    pub fn find_id_and_version(&self, id: u32, version: &Version) -> Option<&SourceFile> {
        self.sources_with_version()
            .filter(|(_, source, v)| source.id == id && *v == version)
            .map(|(_, source, _)| source)
            .next()
    }

    /// Removes the _first_ source_file with the given path from the set
    ///
    /// # Example
    ///
    /// ```
    /// use ethers_solc::Project;
    /// use ethers_solc::artifacts::*;
    /// # fn demo(project: Project) {
    /// let (mut sources, _) = project.compile().unwrap().output().split();
    /// let source_file = sources.remove_by_path("src/Greeter.sol").unwrap();
    /// # }
    /// ```
    pub fn remove_by_path(&mut self, source_file: impl AsRef<str>) -> Option<SourceFile> {
        let source_file_path = source_file.as_ref();
        self.0.get_mut(source_file_path).and_then(|all_sources| {
            if !all_sources.is_empty() {
                Some(all_sources.remove(0).source_file)
            } else {
                None
            }
        })
    }

    /// Removes the _first_ source_file with the given id from the set
    ///
    /// # Example
    ///
    /// ```
    /// use ethers_solc::Project;
    /// use ethers_solc::artifacts::*;
    /// # fn demo(project: Project) {
    /// let (mut sources, _) = project.compile().unwrap().output().split();
    /// let source_file = sources.remove_by_id(0).unwrap();
    /// # }
    /// ```
    pub fn remove_by_id(&mut self, id: u32) -> Option<SourceFile> {
        self.0
            .values_mut()
            .filter_map(|sources| {
                sources
                    .iter()
                    .position(|source| source.source_file.id == id)
                    .map(|pos| sources.remove(pos).source_file)
            })
            .next()
    }

    /// Iterate over all contracts and their names
    pub fn sources(&self) -> impl Iterator<Item = (&String, &SourceFile)> {
        self.0.iter().flat_map(|(path, sources)| {
            sources.iter().map(move |source| (path, &source.source_file))
        })
    }

    /// Returns an iterator over (`file`,  `SourceFile`, `Version`)
    pub fn sources_with_version(&self) -> impl Iterator<Item = (&String, &SourceFile, &Version)> {
        self.0.iter().flat_map(|(file, sources)| {
            sources.iter().map(move |c| (file, &c.source_file, &c.version))
        })
    }

    /// Returns an iterator over all contracts and their source names.
    ///
    /// ```
    /// use std::collections::BTreeMap;
    /// use ethers_solc::{ artifacts::* };
    /// use ethers_solc::sources::VersionedSourceFiles;
    /// # fn demo(sources: VersionedSourceFiles) {
    /// let sources: BTreeMap<String, SourceFile> = sources
    ///     .into_sources()
    ///     .collect();
    /// # }
    /// ```
    pub fn into_sources(self) -> impl Iterator<Item = (String, SourceFile)> {
        self.0.into_iter().flat_map(|(path, sources)| {
            sources.into_iter().map(move |source| (path.clone(), source.source_file))
        })
    }

    /// Returns an iterator over all contracts and their source names.
    ///
    /// ```
    /// use std::collections::BTreeMap;
    /// use semver::Version;
    /// use ethers_solc::{ artifacts::* };
    /// use ethers_solc::sources::VersionedSourceFiles;
    /// # fn demo(sources: VersionedSourceFiles) {
    /// let sources: BTreeMap<(String,Version), SourceFile> = sources
    ///     .into_sources_with_version().map(|(path, source, version)|((path,version), source))
    ///     .collect();
    /// # }
    /// ```
    pub fn into_sources_with_version(self) -> impl Iterator<Item = (String, SourceFile, Version)> {
        self.0.into_iter().flat_map(|(path, sources)| {
            sources
                .into_iter()
                .map(move |source| (path.clone(), source.source_file, source.version))
        })
    }

    /// Sets the sources' file paths to `root` adjoined to `self.file`.
    pub fn join_all(&mut self, root: impl AsRef<Path>) -> &mut Self {
        let root = root.as_ref();
        self.0 = std::mem::take(&mut self.0)
            .into_iter()
            .map(|(file_path, sources)| {
                (root.join(file_path).to_string_lossy().to_string(), sources)
            })
            .collect();
        self
    }

    /// Removes `base` from all source file paths
    pub fn strip_prefix_all(&mut self, base: impl AsRef<Path>) -> &mut Self {
        let base = base.as_ref();
        self.0 = std::mem::take(&mut self.0)
            .into_iter()
            .map(|(file_path, sources)| {
                let p = Path::new(&file_path);
                (
                    p.strip_prefix(base)
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or(file_path),
                    sources,
                )
            })
            .collect();
        self
    }
}

impl AsRef<BTreeMap<String, Vec<VersionedSourceFile>>> for VersionedSourceFiles {
    fn as_ref(&self) -> &BTreeMap<String, Vec<VersionedSourceFile>> {
        &self.0
    }
}

impl AsMut<BTreeMap<String, Vec<VersionedSourceFile>>> for VersionedSourceFiles {
    fn as_mut(&mut self) -> &mut BTreeMap<String, Vec<VersionedSourceFile>> {
        &mut self.0
    }
}

impl IntoIterator for VersionedSourceFiles {
    type Item = (String, Vec<VersionedSourceFile>);
    type IntoIter = std::collections::btree_map::IntoIter<String, Vec<VersionedSourceFile>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// A [SourceFile] and the compiler version used to compile it
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionedSourceFile {
    pub source_file: SourceFile,
    pub version: Version,
}
