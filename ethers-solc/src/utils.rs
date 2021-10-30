//! Utility functions

use std::path::{Component, Path, PathBuf};

use once_cell::sync::Lazy;
use regex::Regex;
use walkdir::WalkDir;

/// A regex that matches the import path and identifier of a solidity import
/// statement with the named groups "path", "id".
pub static RE_SOL_IMPORT: Lazy<Regex> = Lazy::new(|| {
    // Adapted from https://github.com/nomiclabs/hardhat/blob/cced766c65b25d3d0beb39ef847246ac9618bdd9/packages/hardhat-core/src/internal/solidity/parse.ts#L100
    Regex::new(r#"import\s+(?:(?:"(?P<path>[^;]*)"|'([^;]*)')(?:;|\s+as\s+(?P<id>[^;]*);)|.+from\s+(?:"(.*)"|'(.*)');)"#).unwrap()
});

/// A regex that matches the version part of a solidity pragma
/// as follows: `pragma solidity ^0.5.2;` => `^0.5.2`
/// statement with the named groups "path", "id".
// Adapted from https://github.com/nomiclabs/hardhat/blob/cced766c65b25d3d0beb39ef847246ac9618bdd9/packages/hardhat-core/src/internal/solidity/parse.ts#L119
pub static RE_SOL_PRAGMA_VERSION: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"pragma\s+solidity\s+(?P<version>.+?);").unwrap());

/// Returns all path parts from any solidity import statement in a string,
/// `import "./contracts/Contract.sol";` -> `"./contracts/Contract.sol"`.
///
/// See also https://docs.soliditylang.org/en/v0.8.9/grammar.html
pub fn find_import_paths(contract: &str) -> Vec<&str> {
    RE_SOL_IMPORT
        .captures_iter(contract)
        .filter_map(|cap| cap.name("path"))
        .map(|m| m.as_str())
        .collect()
}

/// Returns the solidity version pragma from the given input:
/// `pragma solidity ^0.5.2;` => `^0.5.2`
pub fn find_version_pragma(contract: &str) -> Option<&str> {
    RE_SOL_PRAGMA_VERSION.captures(contract)?.name("version").map(|m| m.as_str())
}

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
        .filter(|e| e.path().extension().map(|ext| ext == "sol").unwrap_or_default())
        .map(|e| e.path().into())
        .collect();
    Ok(files)
}

/// Returns the source name for the given source path, the ancestors of the root path
/// `/Users/project/sources/contract.sol` -> `sources/contracts.sol`
pub fn source_name(source: &Path, root: impl AsRef<Path>) -> &Path {
    source.strip_prefix(root.as_ref()).unwrap_or(source)
}

/// Attempts to determine if the given source is a local, relative import
pub fn is_local_source_name(libs: &[impl AsRef<Path>], source: impl AsRef<Path>) -> bool {
    resolve_library(libs, source).is_none()
}

/// Returns the path to the library if the source path is in fact determined to be a library path,
/// and it exists.
pub fn resolve_library(libs: &[impl AsRef<Path>], source: impl AsRef<Path>) -> Option<PathBuf> {
    let source = source.as_ref();
    let comp = source.components().next()?;
    match comp {
        Component::Normal(first_dir) => {
            // attempt to verify that the root component of this source exists under a library
            // folder
            for lib in libs {
                let lib = lib.as_ref();
                let contract = lib.join(source);
                if contract.exists() {
                    // contract exists in <lib>/<source>
                    return Some(contract)
                }
                // check for <lib>/<first_dir>/src/name.sol
                let contract = lib
                    .join(first_dir)
                    .join("src")
                    .join(source.strip_prefix(first_dir).expect("is first component"));
                if contract.exists() {
                    return Some(contract)
                }
            }
            None
        }
        Component::RootDir => Some(source.into()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        collections::HashSet,
        fs::{create_dir_all, File},
    };

    use tempdir::TempDir;

    #[test]
    fn can_determine_local_paths() {
        assert!(is_local_source_name(&[""], "./local/contract.sol"));
        assert!(is_local_source_name(&[""], "../local/contract.sol"));
        assert!(!is_local_source_name(&[""], "/ds-test/test.sol"));

        let tmp_dir = TempDir::new("contracts").unwrap();
        let dir = tmp_dir.path().join("ds-test");
        create_dir_all(&dir).unwrap();
        File::create(dir.join("test.sol")).unwrap();

        assert!(!is_local_source_name(&[tmp_dir.path()], "ds-test/test.sol"));
    }

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

    #[test]
    fn can_find_import_paths() {
        let s = r##"//SPDX-License-Identifier: Unlicense
pragma solidity ^0.8.0;
import "hardhat/console.sol";
import "../contract/Contract.sol";
"##;
        assert_eq!(vec!["hardhat/console.sol", "../contract/Contract.sol"], find_import_paths(s));
    }
    #[test]
    fn can_find_version() {
        let s = r##"//SPDX-License-Identifier: Unlicense
pragma solidity ^0.8.0;
"##;
        assert_eq!(Some("^0.8.0"), find_version_pragma(s));
    }
}
