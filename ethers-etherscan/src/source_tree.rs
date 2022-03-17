use crate::Result;
use std::{
    fs::create_dir_all,
    path::{Component, Path, PathBuf},
};

#[derive(Debug)]
pub struct SourceTreeEntry {
    pub path: PathBuf,
    pub contents: String,
}

#[derive(Debug)]
pub struct SourceTree {
    pub entries: Vec<SourceTreeEntry>,
}

impl SourceTree {
    /// Expand the source tree into the provided directory.  This method sanitizes paths to ensure
    /// that no directory traversal happens.
    pub fn write_to(&self, dir: &Path) -> Result<()> {
        create_dir_all(&dir)?;
        for entry in &self.entries {
            let sanitized_path = sanitize_path(&entry.path);
            let joined = dir.join(sanitized_path);
            if let Some(parent) = joined.parent() {
                create_dir_all(parent)?;
                std::fs::write(joined, &entry.contents)?;
            }
        }
        Ok(())
    }
}

/// Remove any components in a smart contract source path that could cause a directory traversal.
fn sanitize_path(path: &Path) -> PathBuf {
    Path::new(path)
        .components()
        .filter(|x| x.as_os_str() != Component::ParentDir.as_os_str())
        .collect::<PathBuf>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::read_dir;

    #[test]
    fn test_source_tree_write() {
        let tempdir = tempfile::tempdir().unwrap();
        let st = SourceTree {
            entries: vec![
                SourceTreeEntry { path: PathBuf::from("a/a.sol"), contents: String::from("Test") },
                SourceTreeEntry {
                    path: PathBuf::from("b/b.sol"),
                    contents: String::from("Test 2"),
                },
            ],
        };
        st.write_to(tempdir.path()).unwrap();
        let written_paths = read_dir(tempdir.path()).unwrap();
        let paths: Vec<PathBuf> =
            written_paths.into_iter().filter_map(|x| x.ok()).map(|x| x.path()).collect();
        assert_eq!(paths.len(), 2);
        assert!(paths.contains(&tempdir.path().join("a")));
        assert!(paths.contains(&tempdir.path().join("b")));
    }

    /// Ensure that the .. are ignored when writing the source tree to disk because of
    /// sanitization.
    #[test]
    fn test_malformed_source_tree_write() {
        let tempdir = tempfile::tempdir().unwrap();
        let st = SourceTree {
            entries: vec![
                SourceTreeEntry {
                    path: PathBuf::from("../a/a.sol"),
                    contents: String::from("Test"),
                },
                SourceTreeEntry {
                    path: PathBuf::from("../b/../b.sol"),
                    contents: String::from("Test 2"),
                },
            ],
        };
        st.write_to(tempdir.path()).unwrap();
        let written_paths = read_dir(tempdir.path()).unwrap();
        let paths: Vec<PathBuf> =
            written_paths.into_iter().filter_map(|x| x.ok()).map(|x| x.path()).collect();
        assert_eq!(paths.len(), 2);
        assert!(paths.contains(&tempdir.path().join("a")));
        assert!(paths.contains(&tempdir.path().join("b")));
    }
}
