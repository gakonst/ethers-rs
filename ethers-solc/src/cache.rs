//! Support for compiling contracts
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

/// Hardhat format version
const HH_FORMAT_VERSION: &str = "hh-sol-cache-2";

/// A hardhat compatible cache representation
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SolFilesCache {
    #[serde(rename = "_format")]
    pub format: String,
    pub files: BTreeMap<PathBuf, CachEntry>,
}

impl SolFilesCache {
    fn new(format: impl Into<String>) -> Self {
        Self {
            format: format.into(),
            files: Default::default(),
        }
    }

    /// Reads the cache json file from the given path
    pub fn read(path: impl AsRef<Path>) -> eyre::Result<Self> {
        let file = fs::File::open(path.as_ref())?;
        Ok(serde_json::from_reader(file)?)
    }

    /// Write the cache to json file
    pub fn write(&self, path: impl AsRef<Path>) -> eyre::Result<()> {
        let file = fs::File::create(path.as_ref())?;
        Ok(serde_json::to_writer_pretty(file, self)?)
    }

    pub fn remove_missing_files(&mut self) {
        self.files.retain(|file, _| Path::new(file).exists())
    }

    /// Returns true if the given content hash or config differs from the file's
    /// or the file does not exist
    pub fn has_changed(
        &self,
        file: impl AsRef<Path>,
        hash: impl AsRef<[u8]>,
        config: Option<SolcConfig>,
    ) -> bool {
        if let Some(entry) = self.files.get(file.as_ref()) {
            if entry.content_hash.as_bytes() != hash.as_ref() {
                return true;
            }

            if let Some(config) = config {
                if config != entry.solc_config {
                    return true;
                }
            }
            false
        } else {
            true
        }
    }
}

impl Default for SolFilesCache {
    fn default() -> Self {
        Self::new(HH_FORMAT_VERSION)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CachEntry {
    /// the last modification time of this file
    pub last_modification_date: u64,
    pub content_hash: String,
    pub source_name: String,
    pub solc_config: SolcConfig,
    pub imports: Vec<String>,
    pub version_pragmas: Vec<String>,
    pub artifacts: Vec<String>,
}

impl CachEntry {
    /// Returns the time
    pub fn last_modified(&self) -> Duration {
        Duration::from_millis(self.last_modification_date)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]

pub struct SolcConfig {
    pub version: String,
    pub settings: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse_solidity_files_cache() {
        let input = include_str!("../test-data/solidity-files-cache.json");
        let _ = serde_json::from_str::<SolFilesCache>(input).unwrap();
    }
}
