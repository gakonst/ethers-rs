//! Utility functions

use cfg_if::cfg_if;
use std::{
    collections::HashSet,
    ops::Range,
    path::{Component, Path, PathBuf},
};

use crate::{error::SolcError, SolcIoError};
use once_cell::sync::Lazy;
use regex::{Match, Regex};
use semver::Version;
use serde::de::DeserializeOwned;
use tiny_keccak::{Hasher, Keccak};
use walkdir::WalkDir;

/// A regex that matches the import path and identifier of a solidity import
/// statement with the named groups "path", "id".
// Adapted from <https://github.com/nomiclabs/hardhat/blob/cced766c65b25d3d0beb39ef847246ac9618bdd9/packages/hardhat-core/src/internal/solidity/parse.ts#L100>
pub static RE_SOL_IMPORT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"import\s+(?:(?:"(?P<p1>.*)"|'(?P<p2>.*)')(?:\s+as\s+\w+)?|(?:(?:\w+(?:\s+as\s+\w+)?|\*\s+as\s+\w+|\{\s*(?:\w+(?:\s+as\s+\w+)?(?:\s*,\s*)?)+\s*\})\s+from\s+(?:"(?P<p3>.*)"|'(?P<p4>.*)')))\s*;"#).unwrap()
});

/// A regex that matches an alias within an import statement
pub static RE_SOL_IMPORT_ALIAS: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(?:(?P<target>\w+)|\*|'|")\s+as\s+(?P<alias>\w+)"#).unwrap());

/// A regex that matches the version part of a solidity pragma
/// as follows: `pragma solidity ^0.5.2;` => `^0.5.2`
/// statement with the named group "version".
// Adapted from <https://github.com/nomiclabs/hardhat/blob/cced766c65b25d3d0beb39ef847246ac9618bdd9/packages/hardhat-core/src/internal/solidity/parse.ts#L119>
pub static RE_SOL_PRAGMA_VERSION: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"pragma\s+solidity\s+(?P<version>.+?);").unwrap());

/// A regex that matches the SDPX license identifier
/// statement with the named group "license".
pub static RE_SOL_SDPX_LICENSE_IDENTIFIER: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"///?\s*SPDX-License-Identifier:\s*(?P<license>.+)").unwrap());

/// A regex used to remove extra lines in flatenned files
pub static RE_THREE_OR_MORE_NEWLINES: Lazy<Regex> = Lazy::new(|| Regex::new("\n{3,}").unwrap());

/// Create a regex that matches any library or contract name inside a file
pub fn create_contract_or_lib_name_regex(name: &str) -> Regex {
    Regex::new(&format!(r#"(?:using\s+(?P<n1>{name})\s+|is\s+(?:\w+\s*,\s*)*(?P<n2>{name})(?:\s*,\s*\w+)*|(?:(?P<ignore>(?:function|error|as)\s+|\n[^\n]*(?:"([^"\n]|\\")*|'([^'\n]|\\')*))|\W+)(?P<n3>{name})(?:\.|\(| ))"#)).unwrap()
}

/// Move a range by a specified offset
pub fn range_by_offset(range: &Range<usize>, offset: isize) -> Range<usize> {
    Range {
        start: offset.saturating_add(range.start as isize) as usize,
        end: offset.saturating_add(range.end as isize) as usize,
    }
}

/// Returns all path parts from any solidity import statement in a string,
/// `import "./contracts/Contract.sol";` -> `"./contracts/Contract.sol"`.
///
/// See also <https://docs.soliditylang.org/en/v0.8.9/grammar.html>
pub fn find_import_paths(contract: &str) -> impl Iterator<Item = Match> {
    RE_SOL_IMPORT.captures_iter(contract).filter_map(|cap| {
        cap.name("p1")
            .or_else(|| cap.name("p2"))
            .or_else(|| cap.name("p3"))
            .or_else(|| cap.name("p4"))
    })
}

/// Returns the solidity version pragma from the given input:
/// `pragma solidity ^0.5.2;` => `^0.5.2`
pub fn find_version_pragma(contract: &str) -> Option<Match> {
    RE_SOL_PRAGMA_VERSION.captures(contract)?.name("version")
}

/// Returns an iterator that yields all solidity/yul files funder under the given root path or the
/// `root` itself, if it is a sol/yul file
///
/// This also follows symlinks.
pub fn source_files_iter(root: impl AsRef<Path>) -> impl Iterator<Item = PathBuf> {
    WalkDir::new(root)
        .follow_links(true)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.path().extension().map(|ext| (ext == "sol") || (ext == "yul")).unwrap_or_default()
        })
        .map(|e| e.path().into())
}

/// Returns a list of absolute paths to all the solidity files under the root, or the file itself,
/// if the path is a solidity file.
///
/// This also follows symlinks.
///
/// NOTE: this does not resolve imports from other locations
///
/// # Example
///
/// ```no_run
/// use ethers_solc::utils;
/// let sources = utils::source_files("./contracts");
/// ```
pub fn source_files(root: impl AsRef<Path>) -> Vec<PathBuf> {
    source_files_iter(root).collect()
}

/// Returns a list of _unique_ paths to all folders under `root` that contain at least one solidity
/// file (`*.sol`).
///
/// # Example
///
/// ```no_run
/// use ethers_solc::utils;
/// let dirs = utils::solidity_dirs("./lib");
/// ```
///
/// for following layout will return
/// `["lib/ds-token/src", "lib/ds-token/src/test", "lib/ds-token/lib/ds-math/src", ...]`
///
/// ```text
/// lib
/// └── ds-token
///     ├── lib
///     │   ├── ds-math
///     │   │   └── src/Contract.sol
///     │   ├── ds-stop
///     │   │   └── src/Contract.sol
///     │   ├── ds-test
///     │       └── src//Contract.sol
///     └── src
///         ├── base.sol
///         ├── test
///         │   ├── base.t.sol
///         └── token.sol
/// ```
pub fn solidity_dirs(root: impl AsRef<Path>) -> Vec<PathBuf> {
    let sources = source_files(root);
    sources
        .iter()
        .filter_map(|p| p.parent())
        .collect::<HashSet<_>>()
        .into_iter()
        .map(|p| p.to_path_buf())
        .collect()
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

/// Canonicalize the path, platform-agnostic
///
/// On windows this will ensure the path only consists of `/` separators
pub fn canonicalize(path: impl AsRef<Path>) -> Result<PathBuf, SolcIoError> {
    let path = path.as_ref();
    cfg_if! {
        if #[cfg(windows)] {
            let res = dunce::canonicalize(path).map(|p| {
                use path_slash::PathBufExt;
                PathBuf::from(p.to_slash_lossy().as_ref())
            });
        } else {
         let res = dunce::canonicalize(path);
        }
    };

    res.map_err(|err| SolcIoError::new(err, path))
}

/// Returns the same path config but with canonicalized paths.
///
/// This will take care of potential symbolic linked directories.
/// For example, the tempdir library is creating directories hosted under `/var/`, which in OS X
/// is a symbolic link to `/private/var/`. So if when we try to resolve imports and a path is
/// rooted in a symbolic directory we might end up with different paths for the same file, like
/// `private/var/.../Dapp.sol` and `/var/.../Dapp.sol`
///
/// This canonicalizes all the paths but does not treat non existing dirs as an error
pub fn canonicalized(path: impl Into<PathBuf>) -> PathBuf {
    let path = path.into();
    canonicalize(&path).unwrap_or(path)
}

/// Returns the path to the library if the source path is in fact determined to be a library path,
/// and it exists.
/// Note: this does not handle relative imports or remappings.
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

/// Tries to find an absolute import like `src/interfaces/IConfig.sol` in `cwd`, moving up the path
/// until the `root` is reached.
///
/// If an existing file under `root` is found, this returns the path up to the `import` path and the
/// canonicalized `import` path itself:
///
/// For example for following layout:
///
/// ```text
/// <root>/mydependency/
/// ├── src (`cwd`)
/// │   ├── interfaces
/// │   │   ├── IConfig.sol
/// ```
/// and `import` as `src/interfaces/IConfig.sol` and `cwd` as `src` this will return
/// (`<root>/mydependency/`, `<root>/mydependency/src/interfaces/IConfig.sol`)
pub fn resolve_absolute_library(
    root: &Path,
    cwd: &Path,
    import: &Path,
) -> Option<(PathBuf, PathBuf)> {
    let mut parent = cwd.parent()?;
    while parent != root {
        if let Ok(import) = canonicalize(parent.join(import)) {
            return Some((parent.to_path_buf(), import))
        }
        parent = parent.parent()?;
    }
    None
}

/// Reads the list of Solc versions that have been installed in the machine. The version list is
/// sorted in ascending order.
/// Checks for installed solc versions under the given path as
/// `<root>/<major.minor.path>`, (e.g.: `~/.svm/0.8.10`)
/// and returns them sorted in ascending order
pub fn installed_versions(root: impl AsRef<Path>) -> Result<Vec<Version>, SolcError> {
    let mut versions: Vec<_> = walkdir::WalkDir::new(root)
        .max_depth(1)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.file_type().is_dir())
        .filter_map(|e: walkdir::DirEntry| {
            e.path().file_name().and_then(|v| Version::parse(v.to_string_lossy().as_ref()).ok())
        })
        .collect();
    versions.sort();
    Ok(versions)
}

/// Returns the 36 char (deprecated) fully qualified name placeholder
///
/// If the name is longer than 36 char, then the name gets truncated,
/// If the name is shorter than 36 char, then the name is filled with trailing `_`
pub fn library_fully_qualified_placeholder(name: impl AsRef<str>) -> String {
    name.as_ref().chars().chain(std::iter::repeat('_')).take(36).collect()
}

/// Returns the library hash placeholder as `$hex(library_hash(name))$`
pub fn library_hash_placeholder(name: impl AsRef<[u8]>) -> String {
    let hash = library_hash(name);
    let placeholder = hex::encode(hash);
    format!("${placeholder}$")
}

/// Returns the library placeholder for the given name
/// The placeholder is a 34 character prefix of the hex encoding of the keccak256 hash of the fully
/// qualified library name.
///
/// See also <https://docs.soliditylang.org/en/develop/using-the-compiler.html#library-linking>
pub fn library_hash(name: impl AsRef<[u8]>) -> [u8; 17] {
    let mut output = [0u8; 17];
    let mut hasher = Keccak::v256();
    hasher.update(name.as_ref());
    hasher.finalize(&mut output);
    output
}

/// Find the common ancestor, if any, between the given paths
///
/// # Example
///
/// ```rust
/// use std::path::{PathBuf, Path};
///
/// # fn main() {
/// use ethers_solc::utils::common_ancestor_all;
/// let baz = Path::new("/foo/bar/baz");
/// let bar = Path::new("/foo/bar/bar");
/// let foo = Path::new("/foo/bar/foo");
/// let common = common_ancestor_all(vec![baz, bar, foo]).unwrap();
/// assert_eq!(common, Path::new("/foo/bar").to_path_buf());
/// # }
/// ```
pub fn common_ancestor_all<I, P>(paths: I) -> Option<PathBuf>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    let mut iter = paths.into_iter();
    let mut ret = iter.next()?.as_ref().to_path_buf();
    for path in iter {
        if let Some(r) = common_ancestor(ret, path.as_ref()) {
            ret = r;
        } else {
            return None
        }
    }
    Some(ret)
}

/// Finds the common ancestor of both paths
///
/// # Example
///
/// ```rust
/// use std::path::{PathBuf, Path};
///
/// # fn main() {
/// use ethers_solc::utils::common_ancestor;
/// let foo = Path::new("/foo/bar/foo");
/// let bar = Path::new("/foo/bar/bar");
/// let ancestor = common_ancestor(foo, bar).unwrap();
/// assert_eq!(ancestor, Path::new("/foo/bar").to_path_buf());
/// # }
/// ```
pub fn common_ancestor(a: impl AsRef<Path>, b: impl AsRef<Path>) -> Option<PathBuf> {
    let a = a.as_ref().components();
    let b = b.as_ref().components();
    let mut ret = PathBuf::new();
    let mut found = false;
    for (c1, c2) in a.zip(b) {
        if c1 == c2 {
            ret.push(c1);
            found = true;
        } else {
            break
        }
    }
    if found {
        Some(ret)
    } else {
        None
    }
}

/// Returns the right subpath in a dir
///
/// Returns `<root>/<fave>` if it exists or `<root>/<alt>` does not exist,
/// Returns `<root>/<alt>` if it exists and `<root>/<fave>` does not exist.
pub(crate) fn find_fave_or_alt_path(root: impl AsRef<Path>, fave: &str, alt: &str) -> PathBuf {
    let root = root.as_ref();
    let p = root.join(fave);
    if !p.exists() {
        let alt = root.join(alt);
        if alt.exists() {
            return alt
        }
    }
    p
}

/// Attempts to find a file with different case that exists next to the `non_existing` file
pub(crate) fn find_case_sensitive_existing_file(non_existing: &Path) -> Option<PathBuf> {
    let non_existing_file_name = non_existing.file_name()?;
    let parent = non_existing.parent()?;
    WalkDir::new(parent)
        .max_depth(1)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .find_map(|e| {
            let existing_file_name = e.path().file_name()?;
            if existing_file_name.eq_ignore_ascii_case(non_existing_file_name) &&
                existing_file_name != non_existing_file_name
            {
                return Some(e.path().to_path_buf())
            }
            None
        })
}

#[cfg(not(target_arch = "wasm32"))]
use tokio::runtime::{Handle, Runtime};

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug)]
pub enum RuntimeOrHandle {
    Runtime(Runtime),
    Handle(Handle),
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for RuntimeOrHandle {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl RuntimeOrHandle {
    pub fn new() -> RuntimeOrHandle {
        match Handle::try_current() {
            Ok(handle) => RuntimeOrHandle::Handle(handle),
            Err(_) => RuntimeOrHandle::Runtime(Runtime::new().expect("Failed to start runtime")),
        }
    }

    pub fn block_on<F: std::future::Future>(&self, f: F) -> F::Output {
        match &self {
            RuntimeOrHandle::Runtime(runtime) => runtime.block_on(f),
            RuntimeOrHandle::Handle(handle) => tokio::task::block_in_place(|| handle.block_on(f)),
        }
    }
}

/// Creates a new named tempdir
#[cfg(any(test, feature = "project-util"))]
pub(crate) fn tempdir(name: &str) -> Result<tempfile::TempDir, SolcIoError> {
    tempfile::Builder::new().prefix(name).tempdir().map_err(|err| SolcIoError::new(err, name))
}

/// Reads the json file and deserialize it into the provided type
pub fn read_json_file<T: DeserializeOwned>(path: impl AsRef<Path>) -> Result<T, SolcError> {
    let path = path.as_ref();
    let file = std::fs::File::open(path).map_err(|err| SolcError::io(err, path))?;
    let file = std::io::BufReader::new(file);
    let val: T = serde_json::from_reader(file)?;
    Ok(val)
}

/// Creates the parent directory of the `file` and all its ancestors if it does not exist
/// See [`std::fs::create_dir_all()`]
pub fn create_parent_dir_all(file: impl AsRef<Path>) -> Result<(), SolcError> {
    let file = file.as_ref();
    if let Some(parent) = file.parent() {
        std::fs::create_dir_all(parent).map_err(|err| {
            SolcError::msg(format!(
                "Failed to create artifact parent folder \"{}\": {}",
                parent.display(),
                err
            ))
        })?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use solang_parser::pt::SourceUnitPart;
    use std::{
        collections::HashSet,
        fs::{create_dir_all, File},
    };
    use tempdir;

    #[test]
    fn can_find_different_case() {
        let tmp_dir = tempdir("out").unwrap();
        let path = tmp_dir.path().join("forge-std");
        create_dir_all(&path).unwrap();
        let existing = path.join("Test.sol");
        let non_existing = path.join("test.sol");
        std::fs::write(&existing, b"").unwrap();

        #[cfg(target_os = "linux")]
        assert!(!non_existing.exists());

        let found = find_case_sensitive_existing_file(&non_existing).unwrap();
        assert_eq!(found, existing);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn can_read_different_case() {
        let tmp_dir = tempdir("out").unwrap();
        let path = tmp_dir.path().join("forge-std");
        create_dir_all(&path).unwrap();
        let existing = path.join("Test.sol");
        let non_existing = path.join("test.sol");
        std::fs::write(
            existing,
            r#"
pragma solidity ^0.8.10;
contract A {}
        "#,
        )
        .unwrap();

        assert!(!non_existing.exists());

        let found = crate::resolver::Node::read(&non_existing).unwrap_err();
        matches!(found, SolcError::ResolveCaseSensitiveFileName { .. });
    }

    #[test]
    fn can_create_parent_dirs_with_ext() {
        let tmp_dir = tempdir("out").unwrap();
        let path = tmp_dir.path().join("IsolationModeMagic.sol/IsolationModeMagic.json");
        create_parent_dir_all(&path).unwrap();
        assert!(path.parent().unwrap().is_dir());
    }

    #[test]
    fn can_create_parent_dirs_versioned() {
        let tmp_dir = tempdir("out").unwrap();
        let path = tmp_dir.path().join("IVersioned.sol/IVersioned.0.8.16.json");
        create_parent_dir_all(&path).unwrap();
        assert!(path.parent().unwrap().is_dir());
        let path = tmp_dir.path().join("IVersioned.sol/IVersioned.json");
        create_parent_dir_all(&path).unwrap();
        assert!(path.parent().unwrap().is_dir());
    }

    #[test]
    fn can_determine_local_paths() {
        assert!(is_local_source_name(&[""], "./local/contract.sol"));
        assert!(is_local_source_name(&[""], "../local/contract.sol"));
        assert!(!is_local_source_name(&[""], "/ds-test/test.sol"));

        let tmp_dir = tempdir("contracts").unwrap();
        let dir = tmp_dir.path().join("ds-test");
        create_dir_all(&dir).unwrap();
        File::create(dir.join("test.sol")).unwrap();

        assert!(!is_local_source_name(&[tmp_dir.path()], "ds-test/test.sol"));
    }

    #[test]
    fn can_find_solidity_sources() {
        let tmp_dir = tempdir("contracts").unwrap();

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

        let files: HashSet<_> = source_files(tmp_dir.path()).into_iter().collect();
        let expected: HashSet<_> = [file_a, file_b, file_c, file_d].into();
        assert_eq!(files, expected);
    }

    #[test]
    fn can_parse_curly_bracket_imports() {
        let s =
            r#"import {ReentrancyGuard} from "@openzeppelin/contracts/utils/ReentrancyGuard.sol";"#;

        let (unit, _) = solang_parser::parse(s, 0).unwrap();
        assert_eq!(unit.0.len(), 1);
        match unit.0[0] {
            SourceUnitPart::ImportDirective(_) => {}
            _ => unreachable!("failed to parse import"),
        }
        let imports: Vec<_> = find_import_paths(s).map(|m| m.as_str()).collect();

        assert_eq!(imports, vec!["@openzeppelin/contracts/utils/ReentrancyGuard.sol"])
    }

    #[test]
    fn can_find_single_quote_imports() {
        let content = r#"
// SPDX-License-Identifier: MIT
pragma solidity 0.8.6;

import '@openzeppelin/contracts/access/Ownable.sol';
import '@openzeppelin/contracts/utils/Address.sol';

import './../interfaces/IJBDirectory.sol';
import './../libraries/JBTokens.sol';
        "#;
        let imports: Vec<_> = find_import_paths(content).map(|m| m.as_str()).collect();

        assert_eq!(
            imports,
            vec![
                "@openzeppelin/contracts/access/Ownable.sol",
                "@openzeppelin/contracts/utils/Address.sol",
                "./../interfaces/IJBDirectory.sol",
                "./../libraries/JBTokens.sol",
            ]
        );
    }

    #[test]
    fn can_find_import_paths() {
        let s = r#"//SPDX-License-Identifier: Unlicense
pragma solidity ^0.8.0;
import "hardhat/console.sol";
import "../contract/Contract.sol";
import { T } from "../Test.sol";
import { T } from '../Test2.sol';
"#;
        assert_eq!(
            vec!["hardhat/console.sol", "../contract/Contract.sol", "../Test.sol", "../Test2.sol"],
            find_import_paths(s).map(|m| m.as_str()).collect::<Vec<&str>>()
        );
    }
    #[test]
    fn can_find_version() {
        let s = r##"//SPDX-License-Identifier: Unlicense
pragma solidity ^0.8.0;
"##;
        assert_eq!(Some("^0.8.0"), find_version_pragma(s).map(|s| s.as_str()));
    }

    #[test]
    fn can_find_ancestor() {
        let a = Path::new("/foo/bar/bar/test.txt");
        let b = Path::new("/foo/bar/foo/example/constract.sol");
        let expected = Path::new("/foo/bar");
        assert_eq!(common_ancestor(a, b).unwrap(), expected.to_path_buf())
    }

    #[test]
    fn no_common_ancestor_path() {
        let a = Path::new("/foo/bar");
        let b = Path::new("./bar/foo");
        assert!(common_ancestor(a, b).is_none());
    }

    #[test]
    fn can_find_all_ancestor() {
        let a = Path::new("/foo/bar/foo/example.txt");
        let b = Path::new("/foo/bar/foo/test.txt");
        let c = Path::new("/foo/bar/bar/foo/bar");
        let expected = Path::new("/foo/bar");
        let paths = vec![a, b, c];
        assert_eq!(common_ancestor_all(paths).unwrap(), expected.to_path_buf())
    }
}
