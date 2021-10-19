//! Utility functions

use std::path::{Path, PathBuf};

use walkdir::WalkDir;

/// Returns a list of absolute paths to all the solidity files under the root
///
/// NOTE: this does not resolve imports from other locations
///
/// # Example
///
/// ```no_run
/// use ethers_solc::utils;
/// let sources = utils::source_files("./contracts").unwrap();
/// ```
pub fn source_files(root: impl AsRef<Path>) -> walkdir::Result<Vec<PathBuf>> {
    let files = WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "sol")
                .unwrap_or_default()
        })
        .map(|e| e.path().into())
        .collect();
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::fs::{create_dir_all, File};
    use tempdir::TempDir;

    #[test]
    fn can_find_solidity_sources() {
        let tmp_dir = TempDir::new("contracts").unwrap();

        let file_a = tmp_dir.path().join("a.sol");
        let file_b = tmp_dir.path().join("a.sol");
        let nested = tmp_dir.path().join("nested");
        let file_c = nested.join("c.sol");
        let nested_deep = nested.join("deep");
        let file_d = nested_deep.join("d.sol");
        File::create(&file_a).unwrap();
        File::create(&file_b).unwrap();
        create_dir_all(nested_deep).unwrap();
        File::create(&file_c).unwrap();
        File::create(&file_d).unwrap();

        let files: HashSet<_> = source_files(tmp_dir.path()).unwrap().into_iter().collect();
        let expected: HashSet<_> = [file_a, file_b, file_c, file_d].into();
        assert_eq!(files, expected);
    }
}
