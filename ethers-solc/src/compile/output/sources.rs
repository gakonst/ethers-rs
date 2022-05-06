use crate::SourceFile;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// file -> [(source_file name  -> `SourceFile` + solc version)]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct VersionedSources(pub BTreeMap<String, Vec<VersionedSourceFile>>);

impl VersionedSources {
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
                sources.iter().position(|source| source.source_file.id == id).map(|pos| sources.remove(pos).source_file)
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
    /// # fn demo(sources: VersionedSources) {
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
    /// # fn demo(sources: VersionedSources) {
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
}

impl AsRef<BTreeMap<String, Vec<VersionedSourceFile>>> for VersionedSources {
    fn as_ref(&self) -> &BTreeMap<String, Vec<VersionedSourceFile>> {
        &self.0
    }
}

impl AsMut<BTreeMap<String, Vec<VersionedSourceFile>>> for VersionedSources {
    fn as_mut(&mut self) -> &mut BTreeMap<String, Vec<VersionedSourceFile>> {
        &mut self.0
    }
}

impl IntoIterator for VersionedSources {
    type Item = (String, Vec<VersionedSourceFile>);
    type IntoIter = std::collections::btree_map::IntoIter<String, Vec<VersionedSourceFile>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// A [SourceFile] and the compiler version used to compile it
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VersionedSourceFile {
    pub source_file: SourceFile,
    pub version: Version,
}
