use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map::Entry, HashMap},
    fmt,
    fmt::Write,
    path::{Path, PathBuf},
    str::FromStr,
};

const DAPPTOOLS_CONTRACTS_DIR: &str = "src";
const DAPPTOOLS_LIB_DIR: &str = "lib";
const JS_CONTRACTS_DIR: &str = "contracts";
const JS_LIB_DIR: &str = "node_modules";

/// The solidity compiler can only reference files that exist locally on your computer.
/// So importing directly from GitHub (as an example) is not possible.
///
/// Let's imagine you want to use OpenZeppelin's amazing library of smart contracts,
/// @openzeppelin/contracts-ethereum-package:
///
/// ```ignore
/// pragma solidity 0.5.11;
///
/// import "@openzeppelin/contracts-ethereum-package/contracts/math/SafeMath.sol";
///
/// contract MyContract {
///     using SafeMath for uint256;
///     ...
/// }
/// ```
///
/// When using solc, you have to specify the following:
///
/// "prefix" = the path that's used in your smart contract, i.e.
/// "@openzeppelin/contracts-ethereum-package" "target" = the absolute path of OpenZeppelin's
/// contracts downloaded on your computer
///
/// The format looks like this:
/// `solc prefix=target ./MyContract.sol`
///
/// solc --bin
/// @openzeppelin/contracts-ethereum-package=/Your/Absolute/Path/To/@openzeppelin/
/// contracts-ethereum-package ./MyContract.sol
///
/// [Source](https://ethereum.stackexchange.com/questions/74448/what-are-remappings-and-how-do-they-work-in-solidity)
#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct Remapping {
    pub name: String,
    pub path: String,
}

#[derive(thiserror::Error, Debug, PartialEq, PartialOrd)]
pub enum RemappingError {
    #[error("no prefix found")]
    NoPrefix,
    #[error("no target found")]
    NoTarget,
}

impl FromStr for Remapping {
    type Err = RemappingError;

    fn from_str(remapping: &str) -> std::result::Result<Self, Self::Err> {
        let (name, path) = remapping.split_once('=').ok_or(RemappingError::NoPrefix)?;
        if name.trim().is_empty() {
            return Err(RemappingError::NoPrefix)
        }
        if path.trim().is_empty() {
            return Err(RemappingError::NoTarget)
        }
        Ok(Remapping { name: name.to_string(), path: path.to_string() })
    }
}

impl Serialize for Remapping {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Remapping {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let remapping = String::deserialize(deserializer)?;
        Remapping::from_str(&remapping).map_err(serde::de::Error::custom)
    }
}

// Remappings are printed as `prefix=target`
impl fmt::Display for Remapping {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}={}", self.name, self.path)?;
        if !self.path.ends_with('/') {
            f.write_char('/')?;
        }
        Ok(())
    }
}

impl Remapping {
    /// Returns all formatted remappings
    pub fn find_many_str(path: &str) -> Vec<String> {
        Self::find_many(path).into_iter().map(|r| r.to_string()).collect()
    }

    /// Attempts to autodetect all remappings given a certain root path.
    ///
    /// This will recursively scan all subdirectories of the root path, if a subdirectory contains a
    /// solidity file then this a candidate for a remapping. The name of the remapping will be the
    /// folder name.
    ///
    /// However, there are additional rules/assumptions when it comes to determining if a candidate
    /// should in fact be a remapping:
    ///
    /// All names and paths end with a trailing "/"
    ///
    /// The name of the remapping will be the parent folder of a solidity file, unless the folder is
    /// named `src`, `lib` or `contracts` in which case the name of the remapping will be the parent
    /// folder's name of `src`, `lib`, `contracts`: The remapping of `repo1/src/contract.sol` is
    /// `name: "repo1/", path: "repo1/src/"`
    ///
    /// Nested remappings need to be separated by `src`, `lib` or `contracts`, The remapping of
    /// `repo1/lib/ds-math/src/contract.sol` is `name: "ds-math/", "repo1/lib/ds-math/src/"`
    ///
    /// Remapping detection is primarily designed for dapptool's rules for lib folders, however, we
    /// attempt to detect and optimize various folder structures commonly used in `node_modules`
    /// dependencies. For those the same rules apply. In addition, we try to unify all
    /// remappings discovered according to the rules mentioned above, so that layouts like,
    //   @aave/
    //   ├─ governance/
    //   │  ├─ contracts/
    //   ├─ protocol-v2/
    //   │  ├─ contracts/
    ///
    /// which would be multiple rededications according to our rules ("governance", "protocol-v2"),
    /// are unified into `@aave` by looking at their common ancestor, the root of this subdirectory
    /// (`@aave`)
    pub fn find_many(root: impl AsRef<Path>) -> Vec<Remapping> {
        /// prioritize
        ///   - ("a", "1/2") over ("a", "1/2/3")
        ///   - if a path ends with `src`
        fn insert_prioritized(mappings: &mut HashMap<String, PathBuf>, key: String, path: PathBuf) {
            match mappings.entry(key) {
                Entry::Occupied(mut e) => {
                    if e.get().components().count() > path.components().count() ||
                        (path.ends_with(DAPPTOOLS_CONTRACTS_DIR) &&
                            !e.get().ends_with(DAPPTOOLS_CONTRACTS_DIR))
                    {
                        e.insert(path);
                    }
                }
                Entry::Vacant(e) => {
                    e.insert(path);
                }
            }
        }

        // all combined remappings from all subdirs
        let mut all_remappings = HashMap::new();

        // iterate over all dirs that are children of the root
        for dir in walkdir::WalkDir::new(root)
            .follow_links(true)
            .min_depth(1)
            .max_depth(1)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|e| e.file_type().is_dir())
        {
            let depth1_dir = dir.path();

            // check all remappings in this depth 1 folder
            let candidates = find_remapping_candidates(depth1_dir, depth1_dir, 0);

            for candidate in candidates {
                if let Some(name) = candidate.window_start.file_name().and_then(|s| s.to_str()) {
                    insert_prioritized(
                        &mut all_remappings,
                        format!("{}/", name),
                        candidate.source_dir,
                    );
                }
            }
        }

        all_remappings
            .into_iter()
            .map(|(name, path)| Remapping { name, path: format!("{}/", path.display()) })
            .collect()
    }
}

/// A relative [`Remapping`] that's aware of the current location
///
/// See [`RelativeRemappingPathBuf`]
#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct RelativeRemapping {
    pub name: String,
    pub path: RelativeRemappingPathBuf,
}

impl RelativeRemapping {
    /// Creates a new `RelativeRemapping` starting prefixed with `root`
    pub fn new(remapping: Remapping, root: impl AsRef<Path>) -> Self {
        Self {
            name: remapping.name,
            path: RelativeRemappingPathBuf::with_root(root, remapping.path),
        }
    }

    /// Converts this relative remapping into an absolute remapping
    ///
    /// This sets to root of the remapping to the given `root` path
    pub fn to_remapping(mut self, root: PathBuf) -> Remapping {
        self.path.parent = Some(root);
        self.into()
    }
}

// Remappings are printed as `prefix=target`
impl fmt::Display for RelativeRemapping {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = format!("{}={}", self.name, self.path.original().display());
        if !s.ends_with('/') {
            s.push('/');
        }
        f.write_str(&s)
    }
}

impl From<RelativeRemapping> for Remapping {
    fn from(r: RelativeRemapping) -> Self {
        let RelativeRemapping { mut name, path } = r;
        let mut path = format!("{}", path.relative().display());
        if !path.ends_with('/') {
            path.push('/');
        }
        if !name.ends_with('/') {
            name.push('/');
        }
        Remapping { name, path }
    }
}

impl From<Remapping> for RelativeRemapping {
    fn from(r: Remapping) -> Self {
        Self { name: r.name, path: r.path.into() }
    }
}

/// The path part of the [`Remapping`] that knows the path of the file it was configured in, if any.
///
/// A [`Remapping`] is intended to be absolute, but paths in configuration files are often desired
/// to be relative to the configuration file itself. For example, a path of
/// `weird-erc20/=lib/weird-erc20/src/` configured in a file `/var/foundry.toml` might be desired to
/// resolve as a `weird-erc20/=/var/lib/weird-erc20/src/` remapping.
#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct RelativeRemappingPathBuf {
    parent: Option<PathBuf>,
    path: PathBuf,
}

impl RelativeRemappingPathBuf {
    /// Creates a new `RelativeRemappingPathBuf` that checks if the `path` is a child path of
    /// `parent`.
    pub fn with_root(parent: impl AsRef<Path>, path: impl AsRef<Path>) -> Self {
        let parent = parent.as_ref();
        let path = path.as_ref();
        if let Ok(path) = path.strip_prefix(parent) {
            Self { parent: Some(parent.to_path_buf()), path: path.to_path_buf() }
        } else if path.has_root() {
            Self { parent: None, path: path.to_path_buf() }
        } else {
            Self { parent: Some(parent.to_path_buf()), path: path.to_path_buf() }
        }
    }

    /// Returns the path as it was declared, without modification.
    pub fn original(&self) -> &Path {
        &self.path
    }

    /// Returns this path relative to the file it was delcared in, if any.
    /// Returns the original if this path was not declared in a file or if the
    /// path has a root.
    pub fn relative(&self) -> PathBuf {
        if self.original().has_root() {
            return self.original().into()
        }
        self.parent
            .as_ref()
            .map(|p| p.join(self.original()))
            .unwrap_or_else(|| self.original().into())
    }
}

impl<P: AsRef<Path>> From<P> for RelativeRemappingPathBuf {
    fn from(path: P) -> RelativeRemappingPathBuf {
        Self { parent: None, path: path.as_ref().to_path_buf() }
    }
}

impl Serialize for RelativeRemapping {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for RelativeRemapping {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let remapping = String::deserialize(deserializer)?;
        let remapping = Remapping::from_str(&remapping).map_err(serde::de::Error::custom)?;
        Ok(RelativeRemapping { name: remapping.name, path: remapping.path.into() })
    }
}

#[derive(Debug, Clone)]
struct Candidate {
    /// dir that opened the window
    window_start: PathBuf,
    /// dir that contains the solidity file
    source_dir: PathBuf,
    /// number of the current nested dependency
    window_level: usize,
}

fn is_source_dir(dir: &Path) -> bool {
    dir.file_name()
        .and_then(|p| p.to_str())
        .map(|name| [DAPPTOOLS_CONTRACTS_DIR, JS_CONTRACTS_DIR].contains(&name))
        .unwrap_or_default()
}

fn is_lib_dir(dir: &Path) -> bool {
    dir.file_name()
        .and_then(|p| p.to_str())
        .map(|name| [DAPPTOOLS_LIB_DIR, JS_LIB_DIR].contains(&name))
        .unwrap_or_default()
}

/// Finds all remappings in the directory recursively
fn find_remapping_candidates(
    current_dir: &Path,
    open: &Path,
    current_level: usize,
) -> Vec<Candidate> {
    // this is a marker if the current root is a candidate for a remapping
    let mut is_candidate = false;

    // all found candidates
    let mut candidates = Vec::new();

    // scan all entries in the current dir
    for entry in walkdir::WalkDir::new(current_dir)
        .follow_links(true)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| !entry.file_name().to_str().map(|s| s.starts_with('.')).unwrap_or(false))
    {
        let entry: walkdir::DirEntry = entry;

        // found a solidity file directly the current dir
        if !is_candidate &&
            entry.file_type().is_file() &&
            entry.path().extension() == Some("sol".as_ref())
        {
            is_candidate = true;
        } else if entry.file_type().is_dir() {
            let subdir = entry.path();
            // we skip commonly used subdirs that should not be searched for recursively
            if !(subdir.ends_with("tests") || subdir.ends_with("test") || subdir.ends_with("demo"))
            {
                // scan the subdirectory for remappings, but we need a way to identify nested
                // dependencies like `ds-token/lib/ds-stop/lib/ds-note/src/contract.sol`, or
                // `oz/{tokens,auth}/{contracts, interfaces}/contract.sol` to assign
                // the remappings to their root, we use a window that lies between two barriers. If
                // we find a solidity file within a window, it belongs to the dir that opened the
                // window.

                // check if the subdir is a lib barrier, in which case we open a new window
                if is_lib_dir(subdir) {
                    candidates.extend(find_remapping_candidates(subdir, subdir, current_level + 1));
                } else {
                    // continue scanning with the current window
                    candidates.extend(find_remapping_candidates(subdir, open, current_level));
                }
            }
        }
    }

    // need to find the actual next window in the event `open` is a lib dir
    let window_start = next_nested_window(open, current_dir);
    // finally, we need to merge, adjust candidates from the same level and opening window
    if is_candidate ||
        candidates
            .iter()
            .filter(|c| c.window_level == current_level && c.window_start == window_start)
            .count() >
            1
    {
        // merge all candidates on the current level if the current dir is itself a candidate or
        // there are multiple nested candidates on the current level like `current/{auth,
        // tokens}/contracts/c.sol`
        candidates.retain(|c| c.window_level != current_level);
        candidates.push(Candidate {
            window_start,
            source_dir: current_dir.to_path_buf(),
            window_level: current_level,
        });
    } else {
        // this handles the case if there is a single nested candidate
        if let Some(candidate) = candidates.iter_mut().find(|c| c.window_level == current_level) {
            // we need to determine the distance from the starting point of the window to the
            // contracts dir for cases like `current/nested/contracts/c.sol` which should point to
            // `current`
            let distance = dir_distance(&candidate.window_start, &candidate.source_dir);
            if distance > 1 && candidate.source_dir.ends_with(JS_CONTRACTS_DIR) {
                candidate.source_dir = window_start;
            } else if !is_source_dir(&candidate.source_dir) {
                candidate.source_dir = last_nested_source_dir(open, &candidate.source_dir);
            }
        }
    }
    candidates
}

/// Counts the number of components between `root` and `current`
/// `dir_distance("root/a", "root/a/b/c") == 2`
fn dir_distance(root: &Path, current: &Path) -> usize {
    if root == current {
        return 0
    }
    if let Ok(rem) = current.strip_prefix(root) {
        rem.components().count()
    } else {
        0
    }
}

/// This finds the next window between `root` and `current`
/// If `root` ends with a `lib` component then start join components from `current` until no valid
/// window opener is found
fn next_nested_window(root: &Path, current: &Path) -> PathBuf {
    if !is_lib_dir(root) || root == current {
        return root.to_path_buf()
    }
    if let Ok(rem) = current.strip_prefix(root) {
        let mut p = root.to_path_buf();
        for c in rem.components() {
            let next = p.join(c);
            if !is_lib_dir(&next) || !next.ends_with(JS_CONTRACTS_DIR) {
                return next
            }
            p = next
        }
    }
    root.to_path_buf()
}

/// Finds the last valid source directory in the window (root -> dir)
fn last_nested_source_dir(root: &Path, dir: &Path) -> PathBuf {
    if is_source_dir(dir) {
        return dir.to_path_buf()
    }
    let mut p = dir;
    while let Some(parent) = p.parent() {
        if parent == root {
            return root.to_path_buf()
        }
        if is_source_dir(parent) {
            return parent.to_path_buf()
        }
        p = parent;
    }
    root.to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::tempdir;

    #[test]
    fn relative_remapping() {
        let remapping = "oz=a/b/c/d";
        let remapping = Remapping::from_str(remapping).unwrap();

        let relative = RelativeRemapping::new(remapping.clone(), "a/b/c");
        assert_eq!(relative.path.relative(), Path::new(&remapping.path));
        assert_eq!(relative.path.original(), Path::new("d"));

        let relative = RelativeRemapping::new(remapping.clone(), "x/y");
        assert_eq!(relative.path.relative(), Path::new("x/y/a/b/c/d"));
        assert_eq!(relative.path.original(), Path::new(&remapping.path));

        let remapping = "oz=/a/b/c/d";
        let remapping = Remapping::from_str(remapping).unwrap();
        let relative = RelativeRemapping::new(remapping.clone(), "a/b");
        assert_eq!(relative.path.relative(), Path::new(&remapping.path));
        assert_eq!(relative.path.original(), Path::new(&remapping.path));
        assert!(relative.path.parent.is_none());
    }

    #[test]
    fn serde() {
        let remapping = "oz=../b/c/d";
        let remapping = Remapping::from_str(remapping).unwrap();
        assert_eq!(remapping.name, "oz".to_string());
        assert_eq!(remapping.path, "../b/c/d".to_string());

        let err = Remapping::from_str("").unwrap_err();
        assert_eq!(err, RemappingError::NoPrefix);

        let err = Remapping::from_str("oz=").unwrap_err();
        assert_eq!(err, RemappingError::NoTarget);
    }

    // https://doc.rust-lang.org/rust-by-example/std_misc/fs.html
    fn touch(path: &std::path::Path) -> std::io::Result<()> {
        match std::fs::OpenOptions::new().create(true).write(true).open(path) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    fn mkdir_or_touch(tmp: &std::path::Path, paths: &[&str]) {
        for path in paths {
            if path.ends_with(".sol") {
                let path = tmp.join(path);
                touch(&path).unwrap();
            } else {
                let path = tmp.join(path);
                std::fs::create_dir_all(&path).unwrap();
            }
        }
    }

    // helper function for converting path bufs to remapping strings
    fn to_str(p: std::path::PathBuf) -> String {
        format!("{}/", p.display())
    }

    #[test]
    fn find_remapping_dapptools() {
        let tmp_dir = tempdir("lib").unwrap();
        let tmp_dir_path = tmp_dir.path();
        let paths = ["repo1/src/", "repo1/src/contract.sol"];
        mkdir_or_touch(tmp_dir_path, &paths[..]);

        let path = tmp_dir_path.join("repo1").display().to_string();
        let remappings = Remapping::find_many(tmp_dir_path);
        // repo1/=lib/repo1/src
        assert_eq!(remappings.len(), 1);

        assert_eq!(remappings[0].name, "repo1/");
        assert_eq!(remappings[0].path, format!("{}/src/", path));
    }

    #[test]
    fn recursive_remappings() {
        let tmp_dir = tempdir("lib").unwrap();
        let tmp_dir_path = tmp_dir.path();
        let paths = [
            "repo1/src/",
            "repo1/src/contract.sol",
            "repo1/lib/",
            "repo1/lib/ds-test/src/",
            "repo1/lib/ds-test/src/test.sol",
            "repo1/lib/ds-math/src/",
            "repo1/lib/ds-math/src/contract.sol",
            "repo1/lib/ds-math/lib/ds-test/src/",
            "repo1/lib/ds-math/lib/ds-test/src/test.sol",
            "repo1/lib/guni-lev/src",
            "repo1/lib/guni-lev/src/contract.sol",
            "repo1/lib/solmate/src/auth",
            "repo1/lib/solmate/src/auth/contract.sol",
            "repo1/lib/solmate/src/tokens",
            "repo1/lib/solmate/src/tokens/contract.sol",
            "repo1/lib/solmate/lib/ds-test/src/",
            "repo1/lib/solmate/lib/ds-test/src/test.sol",
            "repo1/lib/solmate/lib/ds-test/demo/",
            "repo1/lib/solmate/lib/ds-test/demo/demo.sol",
            "repo1/lib/openzeppelin-contracts/contracts/access",
            "repo1/lib/openzeppelin-contracts/contracts/access/AccessControl.sol",
            "repo1/lib/ds-token/lib/ds-stop/src",
            "repo1/lib/ds-token/lib/ds-stop/src/contract.sol",
            "repo1/lib/ds-token/lib/ds-stop/lib/ds-note/src",
            "repo1/lib/ds-token/lib/ds-stop/lib/ds-note/src/contract.sol",
        ];
        mkdir_or_touch(tmp_dir_path, &paths[..]);

        let path = tmp_dir_path.display().to_string();
        let mut remappings = Remapping::find_many(&path);
        remappings.sort_unstable();

        let mut expected = vec![
            Remapping {
                name: "repo1/".to_string(),
                path: to_str(tmp_dir_path.join("repo1").join("src")),
            },
            Remapping {
                name: "ds-math/".to_string(),
                path: to_str(tmp_dir_path.join("repo1").join("lib").join("ds-math").join("src")),
            },
            Remapping {
                name: "ds-test/".to_string(),
                path: to_str(tmp_dir_path.join("repo1").join("lib").join("ds-test").join("src")),
            },
            Remapping {
                name: "guni-lev/".to_string(),
                path: to_str(tmp_dir_path.join("repo1/lib/guni-lev").join("src")),
            },
            Remapping {
                name: "solmate/".to_string(),
                path: to_str(tmp_dir_path.join("repo1/lib/solmate").join("src")),
            },
            Remapping {
                name: "openzeppelin-contracts/".to_string(),
                path: to_str(tmp_dir_path.join("repo1/lib/openzeppelin-contracts/contracts")),
            },
            Remapping {
                name: "ds-stop/".to_string(),
                path: to_str(tmp_dir_path.join("repo1/lib/ds-token/lib/ds-stop/src")),
            },
            Remapping {
                name: "ds-note/".to_string(),
                path: to_str(tmp_dir_path.join("repo1/lib/ds-token/lib/ds-stop/lib/ds-note/src")),
            },
        ];
        expected.sort_unstable();
        pretty_assertions::assert_eq!(remappings, expected);
    }

    #[test]
    fn remappings() {
        let tmp_dir = tempdir("tmp").unwrap();
        let tmp_dir_path = tmp_dir.path().join("lib");
        let repo1 = tmp_dir_path.join("src_repo");
        let repo2 = tmp_dir_path.join("contracts_repo");

        let dir1 = repo1.join("src");
        std::fs::create_dir_all(&dir1).unwrap();

        let dir2 = repo2.join("contracts");
        std::fs::create_dir_all(&dir2).unwrap();

        let contract1 = dir1.join("contract.sol");
        touch(&contract1).unwrap();

        let contract2 = dir2.join("contract.sol");
        touch(&contract2).unwrap();

        let path = tmp_dir_path.display().to_string();
        let mut remappings = Remapping::find_many(&path);
        remappings.sort_unstable();
        let mut expected = vec![
            Remapping {
                name: "src_repo/".to_string(),
                path: format!("{}/", dir1.into_os_string().into_string().unwrap()),
            },
            Remapping {
                name: "contracts_repo/".to_string(),
                path: format!(
                    "{}/",
                    repo2.join("contracts").into_os_string().into_string().unwrap()
                ),
            },
        ];
        expected.sort_unstable();
        pretty_assertions::assert_eq!(remappings, expected);
    }

    #[test]
    fn simple_dapptools_remappings() {
        let tmp_dir = tempdir("lib").unwrap();
        let tmp_dir_path = tmp_dir.path();
        let paths = [
            "ds-test/src",
            "ds-test/demo",
            "ds-test/demo/demo.sol",
            "ds-test/src/test.sol",
            "openzeppelin/src",
            "openzeppelin/src/interfaces",
            "openzeppelin/src/interfaces/c.sol",
            "openzeppelin/src/token/ERC/",
            "openzeppelin/src/token/ERC/c.sol",
            "standards/src/interfaces",
            "standards/src/interfaces/iweth.sol",
            "uniswapv2/src",
        ];
        mkdir_or_touch(tmp_dir_path, &paths[..]);

        let path = tmp_dir_path.display().to_string();
        let mut remappings = Remapping::find_many(&path);
        remappings.sort_unstable();

        let mut expected = vec![
            Remapping {
                name: "ds-test/".to_string(),
                path: to_str(tmp_dir_path.join("ds-test/src")),
            },
            Remapping {
                name: "openzeppelin/".to_string(),
                path: to_str(tmp_dir_path.join("openzeppelin/src")),
            },
            Remapping {
                name: "standards/".to_string(),
                path: to_str(tmp_dir_path.join("standards/src")),
            },
        ];
        expected.sort_unstable();
        pretty_assertions::assert_eq!(remappings, expected);
    }

    #[test]
    fn hardhat_remappings() {
        let tmp_dir = tempdir("node_modules").unwrap();
        let tmp_dir_node_modules = tmp_dir.path().join("node_modules");
        let paths = [
            "node_modules/@aave/aave-token/contracts/token/",
            "node_modules/@aave/aave-token/contracts/token/AaveToken.sol",
            "node_modules/@aave/governance-v2/contracts/governance/",
            "node_modules/@aave/governance-v2/contracts/governance/Executor.sol",
            "node_modules/@aave/protocol-v2/contracts/protocol/lendingpool/",
            "node_modules/@aave/protocol-v2/contracts/protocol/lendingpool/LendingPool.sol",
            "node_modules/@ensdomains/ens/contracts/",
            "node_modules/@ensdomains/ens/contracts/contract.sol",
            "node_modules/prettier-plugin-solidity/tests/format/ModifierDefinitions/",
            "node_modules/prettier-plugin-solidity/tests/format/ModifierDefinitions/
            ModifierDefinitions.sol",
            "node_modules/@openzeppelin/contracts/tokens",
            "node_modules/@openzeppelin/contracts/tokens/contract.sol",
            "node_modules/@openzeppelin/contracts/access",
            "node_modules/@openzeppelin/contracts/access/contract.sol",
            "node_modules/eth-gas-reporter/mock/contracts",
            "node_modules/eth-gas-reporter/mock/contracts/ConvertLib.sol",
            "node_modules/eth-gas-reporter/mock/test/",
            "node_modules/eth-gas-reporter/mock/test/TestMetacoin.sol",
        ];
        mkdir_or_touch(tmp_dir.path(), &paths[..]);
        let mut remappings = Remapping::find_many(&tmp_dir_node_modules);
        remappings.sort_unstable();
        let mut expected = vec![
            Remapping {
                name: "@aave/".to_string(),
                path: to_str(tmp_dir_node_modules.join("@aave")),
            },
            Remapping {
                name: "@ensdomains/".to_string(),
                path: to_str(tmp_dir_node_modules.join("@ensdomains")),
            },
            Remapping {
                name: "@openzeppelin/".to_string(),
                path: to_str(tmp_dir_node_modules.join("@openzeppelin/contracts")),
            },
            Remapping {
                name: "eth-gas-reporter/".to_string(),
                path: to_str(tmp_dir_node_modules.join("eth-gas-reporter")),
            },
        ];
        expected.sort_unstable();
        pretty_assertions::assert_eq!(remappings, expected);
    }

    #[test]
    fn can_determine_nested_window() {
        let a = Path::new(
            "/var/folders/l5/lprhf87s6xv8djgd017f0b2h0000gn/T/lib.Z6ODLZJQeJQa/repo1/lib",
        );
        let b = Path::new(
            "/var/folders/l5/lprhf87s6xv8djgd017f0b2h0000gn/T/lib.Z6ODLZJQeJQa/repo1/lib/ds-test/src"
        );
        assert_eq!(next_nested_window(a, b),Path::new(
            "/var/folders/l5/lprhf87s6xv8djgd017f0b2h0000gn/T/lib.Z6ODLZJQeJQa/repo1/lib/ds-test"
        ));
    }
}
