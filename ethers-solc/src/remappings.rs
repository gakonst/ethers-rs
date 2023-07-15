use crate::utils;

use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map::Entry, HashMap},
    fmt,
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
/// `@openzeppelin/contracts-ethereum-package`:
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
/// When using `solc`, you have to specify the following:
///
/// - A `prefix`: the path that's used in your smart contract, i.e.
///   `@openzeppelin/contracts-ethereum-package`
/// - A `target`: the absolute path of the downloaded contracts on your computer
///
/// The format looks like this: `solc prefix=target ./MyContract.sol`
///
/// For example:
///
/// ```text
/// solc --bin \
///     @openzeppelin/contracts-ethereum-package=/Your/Absolute/Path/To/@openzeppelin/contracts-ethereum-package \
///     ./MyContract.sol
/// ```
///
/// You can also specify a `context` which limits the scope of the remapping to a subset of your
/// project. This allows you to apply the remapping only to imports located in a specific library or
/// a specific file. Without a context a remapping is applied to every matching import in all files.
///
/// The format is: `solc context:prefix=target ./MyContract.sol`
///
/// [Source](https://ethereum.stackexchange.com/questions/74448/what-are-remappings-and-how-do-they-work-in-solidity)
#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct Remapping {
    pub context: Option<String>,
    pub name: String,
    pub path: String,
}

impl Remapping {
    /// Convenience function for [`RelativeRemapping::new`]
    pub fn into_relative(self, root: impl AsRef<Path>) -> RelativeRemapping {
        RelativeRemapping::new(self, root)
    }

    /// Removes the `base` path from the remapping
    pub fn strip_prefix(&mut self, base: impl AsRef<Path>) -> &mut Self {
        if let Ok(stripped) = Path::new(&self.path).strip_prefix(base.as_ref()) {
            self.path = format!("{}", stripped.display());
        }
        self
    }
}

#[derive(thiserror::Error, Debug, PartialEq, Eq, PartialOrd)]
pub enum RemappingError {
    #[error("invalid remapping format, found `{0}`, expected `<key>=<value>`")]
    InvalidRemapping(String),
    #[error("remapping key can't be empty, found `{0}`, expected `<key>=<value>`")]
    EmptyRemappingKey(String),
    #[error("remapping value must be a path, found `{0}`, expected `<key>=<value>`")]
    EmptyRemappingValue(String),
}

impl FromStr for Remapping {
    type Err = RemappingError;

    fn from_str(remapping: &str) -> Result<Self, Self::Err> {
        let (name, path) = remapping
            .split_once('=')
            .ok_or_else(|| RemappingError::InvalidRemapping(remapping.to_string()))?;
        let (context, name) = name
            .split_once(':')
            .map_or((None, name), |(context, name)| (Some(context.to_string()), name));
        if name.trim().is_empty() {
            return Err(RemappingError::EmptyRemappingKey(remapping.to_string()))
        }
        if path.trim().is_empty() {
            return Err(RemappingError::EmptyRemappingValue(remapping.to_string()))
        }
        // if the remapping just starts with : (no context name), treat it as global
        let context =
            context.and_then(|c| if c.trim().is_empty() { None } else { Some(c.to_string()) });
        Ok(Remapping { context, name: name.to_string(), path: path.to_string() })
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
        let mut s = String::new();
        if let Some(context) = self.context.as_ref() {
            #[cfg(target_os = "windows")]
            {
                // ensure we have `/` slashes on windows
                use path_slash::PathExt;
                s.push_str(&std::path::Path::new(context).to_slash_lossy());
            }
            #[cfg(not(target_os = "windows"))]
            {
                s.push_str(context);
            }
            s.push(':');
        }
        s.push_str(&{
            #[cfg(target_os = "windows")]
            {
                // ensure we have `/` slashes on windows
                use path_slash::PathExt;
                format!("{}={}", self.name, std::path::Path::new(&self.path).to_slash_lossy())
            }
            #[cfg(not(target_os = "windows"))]
            {
                format!("{}={}", self.name, self.path)
            }
        });

        if !s.ends_with('/') {
            s.push('/');
        }
        f.write_str(&s)
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
    pub fn find_many(dir: impl AsRef<Path>) -> Vec<Remapping> {
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

        let dir = dir.as_ref();
        let is_inside_node_modules = dir.ends_with("node_modules");

        // iterate over all dirs that are children of the root
        for dir in walkdir::WalkDir::new(dir)
            .follow_links(true)
            .min_depth(1)
            .max_depth(1)
            .into_iter()
            .filter_entry(|e| !is_hidden(e))
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_dir())
        {
            let depth1_dir = dir.path();
            // check all remappings in this depth 1 folder
            let candidates =
                find_remapping_candidates(depth1_dir, depth1_dir, 0, is_inside_node_modules);

            for candidate in candidates {
                if let Some(name) = candidate.window_start.file_name().and_then(|s| s.to_str()) {
                    insert_prioritized(
                        &mut all_remappings,
                        format!("{name}/"),
                        candidate.source_dir,
                    );
                }
            }
        }

        all_remappings
            .into_iter()
            .map(|(name, path)| Remapping {
                context: None,
                name,
                path: format!("{}/", path.display()),
            })
            .collect()
    }

    /// Converts any `\\` separators in the `path` to `/`
    pub fn slash_path(&mut self) {
        #[cfg(windows)]
        {
            use path_slash::PathExt;
            self.path = Path::new(&self.path).to_slash_lossy().to_string();
            if let Some(context) = self.context.as_mut() {
                *context = Path::new(&context).to_slash_lossy().to_string();
            }
        }
    }
}

/// A relative [`Remapping`] that's aware of the current location
///
/// See [`RelativeRemappingPathBuf`]
#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct RelativeRemapping {
    pub context: Option<String>,
    pub name: String,
    pub path: RelativeRemappingPathBuf,
}

impl RelativeRemapping {
    /// Creates a new `RelativeRemapping` starting prefixed with `root`
    pub fn new(remapping: Remapping, root: impl AsRef<Path>) -> Self {
        Self {
            context: remapping.context.map(|c| {
                RelativeRemappingPathBuf::with_root(root.as_ref(), c)
                    .path
                    .to_string_lossy()
                    .to_string()
            }),
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

    /// Converts this relative remapping into [`Remapping`] without the root path
    pub fn to_relative_remapping(mut self) -> Remapping {
        self.path.parent.take();
        self.into()
    }
}

// Remappings are printed as `prefix=target`
impl fmt::Display for RelativeRemapping {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = String::new();
        if let Some(context) = self.context.as_ref() {
            #[cfg(target_os = "windows")]
            {
                // ensure we have `/` slashes on windows
                use path_slash::PathExt;
                s.push_str(&std::path::Path::new(context).to_slash_lossy());
            }
            #[cfg(not(target_os = "windows"))]
            {
                s.push_str(context);
            }
            s.push(':');
        }
        s.push_str(&{
            #[cfg(target_os = "windows")]
            {
                // ensure we have `/` slashes on windows
                use path_slash::PathExt;
                format!("{}={}", self.name, self.path.original().to_slash_lossy())
            }
            #[cfg(not(target_os = "windows"))]
            {
                format!("{}={}", self.name, self.path.original().display())
            }
        });

        if !s.ends_with('/') {
            s.push('/');
        }
        f.write_str(&s)
    }
}

impl From<RelativeRemapping> for Remapping {
    fn from(r: RelativeRemapping) -> Self {
        let RelativeRemapping { context, mut name, path } = r;
        let mut path = format!("{}", path.relative().display());
        if !path.ends_with('/') {
            path.push('/');
        }
        if !name.ends_with('/') {
            name.push('/');
        }
        Remapping { context, name, path }
    }
}

impl From<Remapping> for RelativeRemapping {
    fn from(r: Remapping) -> Self {
        Self { context: r.context, name: r.name, path: r.path.into() }
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
    pub parent: Option<PathBuf>,
    pub path: PathBuf,
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
        Ok(RelativeRemapping {
            context: remapping.context,
            name: remapping.name,
            path: remapping.path.into(),
        })
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

impl Candidate {
    /// There are several cases where multiple candidates are detected for the same level
    ///
    /// # Example - Dapptools style
    ///
    /// Another directory next to a `src` dir:
    ///  ```text
    ///  ds-test/
    ///  ├── aux/demo.sol
    ///  └── src/test.sol
    ///  ```
    ///  which effectively ignores the `aux` dir by prioritizing source dirs and keeps
    ///  `ds-test/=ds-test/src/`
    ///
    ///
    /// # Example - node_modules / commonly onpenzeppelin related
    ///
    /// The `@openzeppelin` domain can contain several nested dirs in `node_modules/@openzeppelin`.
    /// Such as
    ///    - `node_modules/@openzeppelin/contracts`
    ///    - `node_modules/@openzeppelin/contracts-upgradeable`
    ///
    /// Which should be resolved to the top level dir `@openzeppelin`
    ///
    /// We also treat candidates with a `node_modules` parent directory differently and consider
    /// them to be `hardhat` style. In which case the trailing library barrier `contracts` will be
    /// stripped from the remapping path. This differs from dapptools style which does not include
    /// the library barrier path `src` in the solidity import statements. For example, for
    /// dapptools you could have
    ///
    /// ```text
    /// <root>/lib/<library>
    /// ├── src
    ///     ├── A.sol
    ///     ├── B.sol
    /// ```
    ///
    /// with remapping `library/=library/src/`
    ///
    /// whereas with hardhat's import resolver the import statement
    ///
    /// ```text
    /// <root>/node_modules/<library>
    /// ├── contracts
    ///     ├── A.sol
    ///     ├── B.sol
    /// ```
    /// with the simple remapping `library/=library/` because hardhat's lib resolver essentially
    /// joins the import path inside a solidity file with the `nodes_modules` folder when it tries
    /// to find an imported solidity file. For example
    ///
    /// ```solidity
    /// import "hardhat/console.sol";
    /// ```
    /// expects the file to be at: `<root>/node_modules/hardhat/console.sol`.
    ///
    /// In order to support these cases, we treat the Dapptools case as the outlier, in which case
    /// we only keep the candidate that ends with `src`
    ///
    ///   - `candidates`: list of viable remapping candidates
    ///   - `current_dir`: the directory that's currently processed, like `@openzeppelin/contracts`
    ///   - `current_level`: the number of nested library dirs encountered
    ///   - `window_start`: This contains the root directory of the current window. In other words
    ///     this will be the parent directory of the most recent library barrier, which will be
    ///     `@openzeppelin` if the `current_dir` is `@openzeppelin/contracts` See also
    ///     [`next_nested_window()`]
    ///   - `is_inside_node_modules` whether we're inside a `node_modules` lib
    fn merge_on_same_level(
        candidates: &mut Vec<Candidate>,
        current_dir: &Path,
        current_level: usize,
        window_start: PathBuf,
        is_inside_node_modules: bool,
    ) {
        // if there's only a single source dir candidate then we use this
        if let Some(pos) = candidates
            .iter()
            .enumerate()
            .fold((0, None), |(mut contracts_dir_count, mut pos), (idx, c)| {
                if c.source_dir.ends_with(DAPPTOOLS_CONTRACTS_DIR) {
                    contracts_dir_count += 1;
                    if contracts_dir_count == 1 {
                        pos = Some(idx)
                    } else {
                        pos = None;
                    }
                }

                (contracts_dir_count, pos)
            })
            .1
        {
            let c = candidates.remove(pos);
            *candidates = vec![c];
        } else {
            // merge all candidates on the current level if the current dir is itself a candidate or
            // there are multiple nested candidates on the current level like `current/{auth,
            // tokens}/contracts/c.sol`
            candidates.retain(|c| c.window_level != current_level);

            let source_dir = if is_inside_node_modules {
                window_start.clone()
            } else {
                current_dir.to_path_buf()
            };

            // if the window start and the source dir are the same directory we can end early if
            // we wrongfully detect something like: `<dep>/src/lib/`
            if current_level > 0 &&
                source_dir == window_start &&
                (is_source_dir(&source_dir) || is_lib_dir(&source_dir))
            {
                return
            }
            candidates.push(Candidate { window_start, source_dir, window_level: current_level });
        }
    }

    /// Returns `true` if the `source_dir` ends with `contracts` or `contracts/src`
    ///
    /// This is used to detect an edge case in `"@chainlink/contracts"` which layout is
    ///
    /// ```text
    /// contracts/src
    /// ├── v0.4
    ///     ├── Pointer.sol
    ///     ├── interfaces
    ///         ├── AggregatorInterface.sol
    ///     ├── tests
    ///         ├── BasicConsumer.sol
    /// ├── v0.5
    ///     ├── Chainlink.sol
    /// ├── v0.6
    ///     ├── AccessControlledAggregator.sol
    /// ```
    ///
    /// And import commonly used is
    ///
    /// ```solidity
    /// import '@chainlink/contracts/src/v0.6/interfaces/AggregatorV3Interface.sol';
    /// ```
    fn source_dir_ends_with_js_source(&self) -> bool {
        self.source_dir.ends_with(JS_CONTRACTS_DIR) || self.source_dir.ends_with("contracts/src/")
    }
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

/// Returns true if the file is _hidden_
fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry.file_name().to_str().map(|s| s.starts_with('.')).unwrap_or(false)
}

/// Finds all remappings in the directory recursively
fn find_remapping_candidates(
    current_dir: &Path,
    open: &Path,
    current_level: usize,
    is_inside_node_modules: bool,
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
        .filter_entry(|e| !is_hidden(e))
        .filter_map(Result::ok)
    {
        let entry: walkdir::DirEntry = entry;

        // found a solidity file directly the current dir
        if !is_candidate &&
            entry.file_type().is_file() &&
            entry.path().extension() == Some("sol".as_ref())
        {
            is_candidate = true;
        } else if entry.file_type().is_dir() {
            // if the dir is a symlink to a parent dir we short circuit here
            // `walkdir` will catch symlink loops, but this check prevents that we end up scanning a
            // workspace like
            // ```text
            // my-package/node_modules
            // ├── dep/node_modules
            //     ├── symlink to `my-package`
            // ```
            if entry.path_is_symlink() {
                if let Ok(target) = utils::canonicalize(entry.path()) {
                    // the symlink points to a parent dir of the current window
                    if open.components().count() > target.components().count() &&
                        utils::common_ancestor(open, &target).is_some()
                    {
                        // short-circuiting
                        return Vec::new()
                    }
                }
            }

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
                    candidates.extend(find_remapping_candidates(
                        subdir,
                        subdir,
                        current_level + 1,
                        is_inside_node_modules,
                    ));
                } else {
                    // continue scanning with the current window
                    candidates.extend(find_remapping_candidates(
                        subdir,
                        open,
                        current_level,
                        is_inside_node_modules,
                    ));
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
        Candidate::merge_on_same_level(
            &mut candidates,
            current_dir,
            current_level,
            window_start,
            is_inside_node_modules,
        );
    } else {
        // this handles the case if there is a single nested candidate
        if let Some(candidate) = candidates.iter_mut().find(|c| c.window_level == current_level) {
            // we need to determine the distance from the starting point of the window to the
            // contracts dir for cases like `current/nested/contracts/c.sol` which should point to
            // `current`
            let distance = dir_distance(&candidate.window_start, &candidate.source_dir);
            if distance > 1 && candidate.source_dir_ends_with_js_source() {
                candidate.source_dir = window_start;
            } else if !is_source_dir(&candidate.source_dir) &&
                candidate.source_dir != candidate.window_start
            {
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
    use crate::{utils::tempdir, ProjectPathsConfig};

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

        let relative = RelativeRemapping::new(remapping, "/a/b");
        assert_eq!(relative.to_relative_remapping(), Remapping::from_str("oz/=c/d/").unwrap());
    }

    #[test]
    fn remapping_errors() {
        let remapping = "oz=../b/c/d";
        let remapping = Remapping::from_str(remapping).unwrap();
        assert_eq!(remapping.name, "oz".to_string());
        assert_eq!(remapping.path, "../b/c/d".to_string());

        let err = Remapping::from_str("").unwrap_err();
        matches!(err, RemappingError::InvalidRemapping(_));

        let err = Remapping::from_str("oz=").unwrap_err();
        matches!(err, RemappingError::EmptyRemappingValue(_));
    }

    // <https://doc.rust-lang.org/rust-by-example/std_misc/fs.html>
    fn touch(path: &std::path::Path) -> std::io::Result<()> {
        match std::fs::OpenOptions::new().create(true).write(true).open(path) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    fn mkdir_or_touch(tmp: &std::path::Path, paths: &[&str]) {
        for path in paths {
            if let Some(parent) = Path::new(path).parent() {
                std::fs::create_dir_all(tmp.join(parent)).unwrap();
            }
            if path.ends_with(".sol") {
                let path = tmp.join(path);
                touch(&path).unwrap();
            } else {
                let path = tmp.join(path);
                std::fs::create_dir_all(path).unwrap();
            }
        }
    }

    // helper function for converting path bufs to remapping strings
    fn to_str(p: std::path::PathBuf) -> String {
        format!("{}/", p.display())
    }

    #[test]
    #[cfg_attr(windows, ignore = "Windows remappings #2347")]
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
        assert_eq!(remappings[0].path, format!("{path}/src/"));
    }

    #[test]
    #[cfg_attr(windows, ignore = "Windows remappings #2347")]
    fn can_resolve_contract_dir_combinations() {
        let tmp_dir = tempdir("demo").unwrap();
        let paths =
            ["lib/timeless/src/lib/A.sol", "lib/timeless/src/B.sol", "lib/timeless/src/test/C.sol"];
        mkdir_or_touch(tmp_dir.path(), &paths[..]);

        let tmp_dir_path = tmp_dir.path().join("lib");
        let remappings = Remapping::find_many(&tmp_dir_path);
        let expected = vec![Remapping {
            context: None,
            name: "timeless/".to_string(),
            path: to_str(tmp_dir_path.join("timeless/src")),
        }];
        pretty_assertions::assert_eq!(remappings, expected);
    }

    #[test]
    #[cfg_attr(windows, ignore = "Windows remappings #2347")]
    fn can_resolve_geb_remappings() {
        let tmp_dir = tempdir("geb").unwrap();
        let paths = [
            "lib/ds-token/src/test/Contract.sol",
            "lib/ds-token/lib/ds-test/src/Contract.sol",
            "lib/ds-token/lib/ds-test/aux/Contract.sol",
            "lib/ds-token/lib/ds-stop/lib/ds-test/src/Contract.sol",
            "lib/ds-token/lib/ds-stop/lib/ds-note/src/Contract.sol",
            "lib/ds-token/lib/ds-math/lib/ds-test/aux/Contract.sol",
            "lib/ds-token/lib/ds-math/src/Contract.sol",
            "lib/ds-token/lib/ds-stop/lib/ds-test/aux/Contract.sol",
            "lib/ds-token/lib/ds-stop/lib/ds-note/lib/ds-test/src/Contract.sol",
            "lib/ds-token/lib/ds-math/lib/ds-test/src/Contract.sol",
            "lib/ds-token/lib/ds-stop/lib/ds-auth/lib/ds-test/src/Contract.sol",
            "lib/ds-token/lib/ds-stop/src/Contract.sol",
            "lib/ds-token/src/Contract.sol",
            "lib/ds-token/lib/erc20/src/Contract.sol",
            "lib/ds-token/lib/ds-stop/lib/ds-auth/lib/ds-test/aux/Contract.sol",
            "lib/ds-token/lib/ds-stop/lib/ds-auth/src/Contract.sol",
            "lib/ds-token/lib/ds-stop/lib/ds-note/lib/ds-test/aux/Contract.sol",
        ];
        mkdir_or_touch(tmp_dir.path(), &paths[..]);

        let tmp_dir_path = tmp_dir.path().join("lib");
        let mut remappings = Remapping::find_many(&tmp_dir_path);
        remappings.sort_unstable();
        let mut expected = vec![
            Remapping {
                context: None,
                name: "ds-auth/".to_string(),
                path: to_str(tmp_dir_path.join("ds-token/lib/ds-stop/lib/ds-auth/src")),
            },
            Remapping {
                context: None,
                name: "ds-math/".to_string(),
                path: to_str(tmp_dir_path.join("ds-token/lib/ds-math/src")),
            },
            Remapping {
                context: None,
                name: "ds-note/".to_string(),
                path: to_str(tmp_dir_path.join("ds-token/lib/ds-stop/lib/ds-note/src")),
            },
            Remapping {
                context: None,
                name: "ds-stop/".to_string(),
                path: to_str(tmp_dir_path.join("ds-token/lib/ds-stop/src")),
            },
            Remapping {
                context: None,
                name: "ds-test/".to_string(),
                path: to_str(tmp_dir_path.join("ds-token/lib/ds-test/src")),
            },
            Remapping {
                context: None,
                name: "ds-token/".to_string(),
                path: to_str(tmp_dir_path.join("ds-token/src")),
            },
            Remapping {
                context: None,
                name: "erc20/".to_string(),
                path: to_str(tmp_dir_path.join("ds-token/lib/erc20/src")),
            },
        ];
        expected.sort_unstable();
        pretty_assertions::assert_eq!(remappings, expected);
    }

    #[test]
    fn can_resolve_nested_chainlink_remappings() {
        let tmp_dir = tempdir("root").unwrap();
        let paths = [
            "@chainlink/contracts/src/v0.6/vendor/Contract.sol",
            "@chainlink/contracts/src/v0.8/tests/Contract.sol",
            "@chainlink/contracts/src/v0.7/Contract.sol",
            "@chainlink/contracts/src/v0.6/Contract.sol",
            "@chainlink/contracts/src/v0.5/Contract.sol",
            "@chainlink/contracts/src/v0.7/tests/Contract.sol",
            "@chainlink/contracts/src/v0.7/interfaces/Contract.sol",
            "@chainlink/contracts/src/v0.4/tests/Contract.sol",
            "@chainlink/contracts/src/v0.6/tests/Contract.sol",
            "@chainlink/contracts/src/v0.5/tests/Contract.sol",
            "@chainlink/contracts/src/v0.8/vendor/Contract.sol",
            "@chainlink/contracts/src/v0.5/dev/Contract.sol",
            "@chainlink/contracts/src/v0.6/examples/Contract.sol",
            "@chainlink/contracts/src/v0.5/interfaces/Contract.sol",
            "@chainlink/contracts/src/v0.4/interfaces/Contract.sol",
            "@chainlink/contracts/src/v0.4/vendor/Contract.sol",
            "@chainlink/contracts/src/v0.6/interfaces/Contract.sol",
            "@chainlink/contracts/src/v0.7/dev/Contract.sol",
            "@chainlink/contracts/src/v0.8/dev/Contract.sol",
            "@chainlink/contracts/src/v0.5/vendor/Contract.sol",
            "@chainlink/contracts/src/v0.7/vendor/Contract.sol",
            "@chainlink/contracts/src/v0.4/Contract.sol",
            "@chainlink/contracts/src/v0.8/interfaces/Contract.sol",
            "@chainlink/contracts/src/v0.6/dev/Contract.sol",
        ];
        mkdir_or_touch(tmp_dir.path(), &paths[..]);
        let remappings = Remapping::find_many(tmp_dir.path());

        let expected = vec![Remapping {
            context: None,
            name: "@chainlink/".to_string(),
            path: to_str(tmp_dir.path().join("@chainlink")),
        }];
        pretty_assertions::assert_eq!(remappings, expected);
    }

    #[test]
    fn can_resolve_oz_upgradeable_remappings() {
        let tmp_dir = tempdir("root").unwrap();
        let paths = [
            "@openzeppelin/contracts-upgradeable/proxy/ERC1967/Contract.sol",
            "@openzeppelin/contracts-upgradeable/token/ERC1155/Contract.sol",
            "@openzeppelin/contracts/token/ERC777/Contract.sol",
            "@openzeppelin/contracts/token/ERC721/presets/Contract.sol",
            "@openzeppelin/contracts/interfaces/Contract.sol",
            "@openzeppelin/contracts-upgradeable/token/ERC777/presets/Contract.sol",
            "@openzeppelin/contracts/token/ERC1155/extensions/Contract.sol",
            "@openzeppelin/contracts/proxy/Contract.sol",
            "@openzeppelin/contracts/proxy/utils/Contract.sol",
            "@openzeppelin/contracts-upgradeable/security/Contract.sol",
            "@openzeppelin/contracts-upgradeable/utils/Contract.sol",
            "@openzeppelin/contracts/token/ERC20/Contract.sol",
            "@openzeppelin/contracts-upgradeable/utils/introspection/Contract.sol",
            "@openzeppelin/contracts/metatx/Contract.sol",
            "@openzeppelin/contracts/utils/cryptography/Contract.sol",
            "@openzeppelin/contracts/token/ERC20/utils/Contract.sol",
            "@openzeppelin/contracts-upgradeable/token/ERC20/utils/Contract.sol",
            "@openzeppelin/contracts-upgradeable/proxy/Contract.sol",
            "@openzeppelin/contracts-upgradeable/token/ERC20/presets/Contract.sol",
            "@openzeppelin/contracts-upgradeable/utils/math/Contract.sol",
            "@openzeppelin/contracts-upgradeable/utils/escrow/Contract.sol",
            "@openzeppelin/contracts/governance/extensions/Contract.sol",
            "@openzeppelin/contracts-upgradeable/interfaces/Contract.sol",
            "@openzeppelin/contracts/proxy/transparent/Contract.sol",
            "@openzeppelin/contracts/utils/structs/Contract.sol",
            "@openzeppelin/contracts-upgradeable/access/Contract.sol",
            "@openzeppelin/contracts/governance/compatibility/Contract.sol",
            "@openzeppelin/contracts/governance/Contract.sol",
            "@openzeppelin/contracts-upgradeable/governance/extensions/Contract.sol",
            "@openzeppelin/contracts/security/Contract.sol",
            "@openzeppelin/contracts-upgradeable/metatx/Contract.sol",
            "@openzeppelin/contracts-upgradeable/token/ERC721/utils/Contract.sol",
            "@openzeppelin/contracts/token/ERC721/utils/Contract.sol",
            "@openzeppelin/contracts-upgradeable/governance/compatibility/Contract.sol",
            "@openzeppelin/contracts/token/common/Contract.sol",
            "@openzeppelin/contracts/proxy/beacon/Contract.sol",
            "@openzeppelin/contracts-upgradeable/token/ERC721/Contract.sol",
            "@openzeppelin/contracts-upgradeable/proxy/beacon/Contract.sol",
            "@openzeppelin/contracts/token/ERC1155/utils/Contract.sol",
            "@openzeppelin/contracts/token/ERC777/presets/Contract.sol",
            "@openzeppelin/contracts-upgradeable/token/ERC20/Contract.sol",
            "@openzeppelin/contracts-upgradeable/utils/structs/Contract.sol",
            "@openzeppelin/contracts/utils/escrow/Contract.sol",
            "@openzeppelin/contracts/utils/Contract.sol",
            "@openzeppelin/contracts-upgradeable/token/ERC721/extensions/Contract.sol",
            "@openzeppelin/contracts/token/ERC721/extensions/Contract.sol",
            "@openzeppelin/contracts-upgradeable/token/ERC777/Contract.sol",
            "@openzeppelin/contracts/token/ERC1155/presets/Contract.sol",
            "@openzeppelin/contracts/token/ERC721/Contract.sol",
            "@openzeppelin/contracts/token/ERC1155/Contract.sol",
            "@openzeppelin/contracts-upgradeable/governance/Contract.sol",
            "@openzeppelin/contracts/token/ERC20/extensions/Contract.sol",
            "@openzeppelin/contracts-upgradeable/utils/cryptography/Contract.sol",
            "@openzeppelin/contracts-upgradeable/token/ERC1155/presets/Contract.sol",
            "@openzeppelin/contracts/access/Contract.sol",
            "@openzeppelin/contracts/governance/utils/Contract.sol",
            "@openzeppelin/contracts-upgradeable/token/ERC20/extensions/Contract.sol",
            "@openzeppelin/contracts-upgradeable/token/common/Contract.sol",
            "@openzeppelin/contracts-upgradeable/token/ERC1155/utils/Contract.sol",
            "@openzeppelin/contracts/proxy/ERC1967/Contract.sol",
            "@openzeppelin/contracts/finance/Contract.sol",
            "@openzeppelin/contracts-upgradeable/token/ERC1155/extensions/Contract.sol",
            "@openzeppelin/contracts-upgradeable/governance/utils/Contract.sol",
            "@openzeppelin/contracts-upgradeable/proxy/utils/Contract.sol",
            "@openzeppelin/contracts/token/ERC20/presets/Contract.sol",
            "@openzeppelin/contracts/utils/math/Contract.sol",
            "@openzeppelin/contracts-upgradeable/token/ERC721/presets/Contract.sol",
            "@openzeppelin/contracts-upgradeable/finance/Contract.sol",
            "@openzeppelin/contracts/utils/introspection/Contract.sol",
        ];
        mkdir_or_touch(tmp_dir.path(), &paths[..]);
        let remappings = Remapping::find_many(tmp_dir.path());

        let expected = vec![Remapping {
            context: None,
            name: "@openzeppelin/".to_string(),
            path: to_str(tmp_dir.path().join("@openzeppelin")),
        }];
        pretty_assertions::assert_eq!(remappings, expected);
    }

    #[test]
    fn can_resolve_oz_remappings() {
        let tmp_dir = tempdir("node_modules").unwrap();
        let tmp_dir_node_modules = tmp_dir.path().join("node_modules");
        let paths = [
            "node_modules/@openzeppelin/contracts/interfaces/IERC1155.sol",
            "node_modules/@openzeppelin/contracts/finance/VestingWallet.sol",
            "node_modules/@openzeppelin/contracts/proxy/Proxy.sol",
            "node_modules/@openzeppelin/contracts/token/ERC20/IERC20.sol",
        ];
        mkdir_or_touch(tmp_dir.path(), &paths[..]);
        let remappings = Remapping::find_many(tmp_dir_node_modules);
        let mut paths = ProjectPathsConfig::hardhat(tmp_dir.path()).unwrap();
        paths.remappings = remappings;

        let resolved = paths
            .resolve_library_import(
                tmp_dir.path(),
                Path::new("@openzeppelin/contracts/token/ERC20/IERC20.sol"),
            )
            .unwrap();
        assert!(resolved.exists());

        // adjust remappings
        paths.remappings[0].name = "@openzeppelin/".to_string();

        let resolved = paths
            .resolve_library_import(
                tmp_dir.path(),
                Path::new("@openzeppelin/contracts/token/ERC20/IERC20.sol"),
            )
            .unwrap();
        assert!(resolved.exists());
    }

    #[test]
    #[cfg_attr(windows, ignore = "Windows remappings #2347")]
    fn recursive_remappings() {
        let tmp_dir = tempdir("lib").unwrap();
        let tmp_dir_path = tmp_dir.path();
        let paths = [
            "repo1/src/contract.sol",
            "repo1/lib/ds-test/src/test.sol",
            "repo1/lib/ds-math/src/contract.sol",
            "repo1/lib/ds-math/lib/ds-test/src/test.sol",
            "repo1/lib/guni-lev/src/contract.sol",
            "repo1/lib/solmate/src/auth/contract.sol",
            "repo1/lib/solmate/src/tokens/contract.sol",
            "repo1/lib/solmate/lib/ds-test/src/test.sol",
            "repo1/lib/solmate/lib/ds-test/demo/demo.sol",
            "repo1/lib/openzeppelin-contracts/contracts/access/AccessControl.sol",
            "repo1/lib/ds-token/lib/ds-stop/src/contract.sol",
            "repo1/lib/ds-token/lib/ds-stop/lib/ds-note/src/contract.sol",
        ];
        mkdir_or_touch(tmp_dir_path, &paths[..]);

        let path = tmp_dir_path.display().to_string();
        let mut remappings = Remapping::find_many(path);
        remappings.sort_unstable();

        let mut expected = vec![
            Remapping {
                context: None,
                name: "repo1/".to_string(),
                path: to_str(tmp_dir_path.join("repo1").join("src")),
            },
            Remapping {
                context: None,
                name: "ds-math/".to_string(),
                path: to_str(tmp_dir_path.join("repo1").join("lib").join("ds-math").join("src")),
            },
            Remapping {
                context: None,
                name: "ds-test/".to_string(),
                path: to_str(tmp_dir_path.join("repo1").join("lib").join("ds-test").join("src")),
            },
            Remapping {
                context: None,
                name: "guni-lev/".to_string(),
                path: to_str(tmp_dir_path.join("repo1/lib/guni-lev").join("src")),
            },
            Remapping {
                context: None,
                name: "solmate/".to_string(),
                path: to_str(tmp_dir_path.join("repo1/lib/solmate").join("src")),
            },
            Remapping {
                context: None,
                name: "openzeppelin-contracts/".to_string(),
                path: to_str(tmp_dir_path.join("repo1/lib/openzeppelin-contracts/contracts")),
            },
            Remapping {
                context: None,
                name: "ds-stop/".to_string(),
                path: to_str(tmp_dir_path.join("repo1/lib/ds-token/lib/ds-stop/src")),
            },
            Remapping {
                context: None,
                name: "ds-note/".to_string(),
                path: to_str(tmp_dir_path.join("repo1/lib/ds-token/lib/ds-stop/lib/ds-note/src")),
            },
        ];
        expected.sort_unstable();
        pretty_assertions::assert_eq!(remappings, expected);
    }

    #[test]
    fn can_resolve_contexts() {
        let remapping = "context:oz=a/b/c/d";
        let remapping = Remapping::from_str(remapping).unwrap();

        assert_eq!(
            remapping,
            Remapping {
                context: Some("context".to_string()),
                name: "oz".to_string(),
                path: "a/b/c/d".to_string(),
            }
        );
        assert_eq!(remapping.to_string(), "context:oz=a/b/c/d/".to_string());

        let remapping = "context:foo=C:/bar/src/";
        let remapping = Remapping::from_str(remapping).unwrap();

        assert_eq!(
            remapping,
            Remapping {
                context: Some("context".to_string()),
                name: "foo".to_string(),
                path: "C:/bar/src/".to_string()
            }
        );
    }

    #[test]
    fn can_resolve_global_contexts() {
        let remapping = ":oz=a/b/c/d/";
        let remapping = Remapping::from_str(remapping).unwrap();

        assert_eq!(
            remapping,
            Remapping { context: None, name: "oz".to_string(), path: "a/b/c/d/".to_string() }
        );
        assert_eq!(remapping.to_string(), "oz=a/b/c/d/".to_string());
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
        let mut remappings = Remapping::find_many(path);
        remappings.sort_unstable();
        let mut expected = vec![
            Remapping {
                context: None,
                name: "src_repo/".to_string(),
                path: format!("{}/", dir1.into_os_string().into_string().unwrap()),
            },
            Remapping {
                context: None,
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
    #[cfg_attr(windows, ignore = "Windows remappings #2347")]
    fn simple_dapptools_remappings() {
        let tmp_dir = tempdir("lib").unwrap();
        let tmp_dir_path = tmp_dir.path();
        let paths = [
            "ds-test/src",
            "ds-test/demo",
            "ds-test/demo/demo.sol",
            "ds-test/src/test.sol",
            "openzeppelin/src/interfaces/c.sol",
            "openzeppelin/src/token/ERC/c.sol",
            "standards/src/interfaces/iweth.sol",
            "uniswapv2/src",
        ];
        mkdir_or_touch(tmp_dir_path, &paths[..]);

        let path = tmp_dir_path.display().to_string();
        let mut remappings = Remapping::find_many(path);
        remappings.sort_unstable();

        let mut expected = vec![
            Remapping {
                context: None,
                name: "ds-test/".to_string(),
                path: to_str(tmp_dir_path.join("ds-test/src")),
            },
            Remapping {
                context: None,
                name: "openzeppelin/".to_string(),
                path: to_str(tmp_dir_path.join("openzeppelin/src")),
            },
            Remapping {
                context: None,
                name: "standards/".to_string(),
                path: to_str(tmp_dir_path.join("standards/src")),
            },
        ];
        expected.sort_unstable();
        pretty_assertions::assert_eq!(remappings, expected);
    }

    #[test]
    #[cfg_attr(windows, ignore = "Windows remappings #2347")]
    fn hardhat_remappings() {
        let tmp_dir = tempdir("node_modules").unwrap();
        let tmp_dir_node_modules = tmp_dir.path().join("node_modules");
        let paths = [
            "node_modules/@aave/aave-token/contracts/token/AaveToken.sol",
            "node_modules/@aave/governance-v2/contracts/governance/Executor.sol",
            "node_modules/@aave/protocol-v2/contracts/protocol/lendingpool/",
            "node_modules/@aave/protocol-v2/contracts/protocol/lendingpool/LendingPool.sol",
            "node_modules/@ensdomains/ens/contracts/contract.sol",
            "node_modules/prettier-plugin-solidity/tests/format/ModifierDefinitions/",
            "node_modules/prettier-plugin-solidity/tests/format/ModifierDefinitions/
            ModifierDefinitions.sol",
            "node_modules/@openzeppelin/contracts/tokens/contract.sol",
            "node_modules/@openzeppelin/contracts/access/contract.sol",
            "node_modules/eth-gas-reporter/mock/contracts/ConvertLib.sol",
            "node_modules/eth-gas-reporter/mock/test/TestMetacoin.sol",
        ];
        mkdir_or_touch(tmp_dir.path(), &paths[..]);
        let mut remappings = Remapping::find_many(&tmp_dir_node_modules);
        remappings.sort_unstable();
        let mut expected = vec![
            Remapping {
                context: None,
                name: "@aave/".to_string(),
                path: to_str(tmp_dir_node_modules.join("@aave")),
            },
            Remapping {
                context: None,
                name: "@ensdomains/".to_string(),
                path: to_str(tmp_dir_node_modules.join("@ensdomains")),
            },
            Remapping {
                context: None,
                name: "@openzeppelin/".to_string(),
                path: to_str(tmp_dir_node_modules.join("@openzeppelin")),
            },
            Remapping {
                context: None,
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

    #[test]
    #[cfg_attr(windows, ignore = "Windows remappings #2347")]
    fn find_openzeppelin_remapping() {
        let tmp_dir = tempdir("lib").unwrap();
        let tmp_dir_path = tmp_dir.path();
        let paths = [
            "lib/ds-test/src/test.sol",
            "lib/forge-std/src/test.sol",
            "openzeppelin/contracts/interfaces/c.sol",
        ];
        mkdir_or_touch(tmp_dir_path, &paths[..]);

        let path = tmp_dir_path.display().to_string();
        let mut remappings = Remapping::find_many(path);
        remappings.sort_unstable();

        let mut expected = vec![
            Remapping {
                context: None,
                name: "ds-test/".to_string(),
                path: to_str(tmp_dir_path.join("lib/ds-test/src")),
            },
            Remapping {
                context: None,
                name: "openzeppelin/".to_string(),
                path: to_str(tmp_dir_path.join("openzeppelin/contracts")),
            },
            Remapping {
                context: None,
                name: "forge-std/".to_string(),
                path: to_str(tmp_dir_path.join("lib/forge-std/src")),
            },
        ];
        expected.sort_unstable();
        pretty_assertions::assert_eq!(remappings, expected);
    }
}
