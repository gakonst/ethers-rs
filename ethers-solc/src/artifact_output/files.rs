use crate::contracts::VersionedContract;
use std::{
    collections::HashMap,
    hash::Hash,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
};

/// Container type for all contracts mapped to their output file
pub struct MappedArtifactFiles<'a> {
    /// Represents the determined output artifact file and the contract(s) that target it
    ///
    /// This is guaranteed to be `len >= 1`
    ///
    /// If there is more than 1 contract in the map, this means we have a naming conflict where
    /// different contracts target the same output file. This happens if the solidity file and
    /// contract name match, but they are in different folders.
    pub files: HashMap<MappedArtifactFile, Vec<MappedContract<'a>>>,
}

impl<'a> MappedArtifactFiles<'a> {
    pub fn with_capacity(len: usize) -> Self {
        Self { files: HashMap::with_capacity(len) }
    }
}

impl<'a> Deref for MappedArtifactFiles<'a> {
    type Target = HashMap<MappedArtifactFile, Vec<MappedContract<'a>>>;

    fn deref(&self) -> &Self::Target {
        &self.files
    }
}

impl<'a> DerefMut for MappedArtifactFiles<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.files
    }
}

/// Represents the targeted path of a contract or multiple contracts
///
/// To account for case-sensitivity we identify it via lowercase path
#[derive(Debug, Hash, PartialEq, Eq)]
pub struct MappedArtifactFile {
    lower_case_path: String,
}

impl MappedArtifactFile {
    pub fn new(path: &Path) -> Self {
        Self { lower_case_path: path.to_string_lossy().to_lowercase() }
    }
}

pub struct MappedContract<'a> {
    pub file: &'a str,
    pub name: &'a str,
    pub contract: &'a VersionedContract,
    pub artifact_path: PathBuf,
}
