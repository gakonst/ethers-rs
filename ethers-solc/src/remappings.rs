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
        write!(f, "{}={}", self.name, self.path)
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
            let children = scan_children(depth1_dir);

            let ancestor = if children.len() > 1 {
                crate::utils::common_ancestor_all(children.values()).unwrap()
            } else {
                depth1_dir.to_path_buf()
            };

            for path in children.into_values() {
                if let Some((name, path)) = to_remapping(path, &ancestor) {
                    insert_prioritized(&mut all_remappings, name, path);
                }
            }
        }
        all_remappings
            .into_iter()
            .map(|(name, path)| Remapping { name, path: format!("{}/", path.display()) })
            .collect()
    }
}

/// Recursively scans sub folders and checks if they contain a solidity file
fn scan_children(root: &Path) -> HashMap<String, PathBuf> {
    // this is a marker if the current root is already a remapping
    let mut remapping = false;

    // all found remappings
    let mut remappings = HashMap::new();

    for entry in walkdir::WalkDir::new(&root)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        let entry: walkdir::DirEntry = entry;

        if entry.file_type().is_file() && !remapping {
            if entry.file_name().to_str().filter(|f| f.ends_with(".sol")).is_some() {
                // found a solidity file
                // this will hold the actual root remapping if root is named `src` or `lib`
                let actual_parent = root.parent().filter(|_| {
                    root.ends_with(DAPPTOOLS_CONTRACTS_DIR) ||
                        root.ends_with(DAPPTOOLS_LIB_DIR) ||
                        root.ends_with(JS_CONTRACTS_DIR)
                });

                let parent = actual_parent.unwrap_or(root);
                if let Some(name) = parent.file_name().and_then(|f| f.to_str()) {
                    remappings.insert(name.to_string(), root.to_path_buf());
                    remapping = true;
                }
            }
        } else if entry.file_type().is_dir() {
            let path = entry.path();
            // we skip commonly used subdirs that should not be included
            if !(path.ends_with("tests") ||
                path.ends_with("test") ||
                path.ends_with("node_modules") ||
                path.ends_with("demo"))
            {
                for (name, path) in scan_children(path) {
                    if let Entry::Vacant(e) = remappings.entry(name) {
                        e.insert(path);
                    }
                }
            }
        }
    }
    remappings
}

/// Determine the remapping for a path based on the ancestor
fn to_remapping(path: PathBuf, ancestor: &Path) -> Option<(String, PathBuf)> {
    let rem = path.strip_prefix(ancestor).ok()?;
    // strip dapptools style dirs, `lib/solmate/src` -> `solmate/src`
    if let Ok((peek, barrier)) = rem
        .strip_prefix(DAPPTOOLS_CONTRACTS_DIR)
        .map(|p| (p, DAPPTOOLS_CONTRACTS_DIR))
        .or_else(|_| rem.strip_prefix("lib").map(|p| (p, "lib")))
    {
        // this is considered a dapptools style dir as it starts with `src`, `lib`
        if let Some(c) = peek.components().next() {
            // here we need to handle layouts that deviate from dapptools layout like `peek:
            // openzeppelin-contracts/contracts/tokens/contract.sol` which really should just
            // `openzeppelin-contracts`
            if peek.ends_with(DAPPTOOLS_CONTRACTS_DIR) || peek.ends_with(DAPPTOOLS_LIB_DIR) {
                last_valid_mapping(&path)
            } else {
                // simply cut off after the next barrier (src, lib, contracts)
                let name = c.as_os_str().to_str()?;
                let path = join_until_next_barrier(ancestor.join(barrier), peek);
                if path.ends_with(JS_CONTRACTS_DIR) ||
                    path.ends_with(DAPPTOOLS_CONTRACTS_DIR) ||
                    path.ends_with(DAPPTOOLS_LIB_DIR)
                {
                    Some((format!("{}/", name), path))
                } else {
                    let name = ancestor.file_name()?.to_str()?;
                    Some((format!("{}/", name), ancestor.join(barrier)))
                }
            }
        } else {
            let name = ancestor.file_name()?.to_str()?;
            Some((format!("{}/", name), path))
        }
    } else {
        // this is likely a hardhat/node_modules dir, in which case we assume the following
        // `@aave/tokens/contracts` -> `@aave`
        // `@openzeppelin/contracts` -> `@openzeppelin/contracts`
        if ancestor.ends_with(JS_CONTRACTS_DIR) {
            last_valid_mapping(ancestor)
        } else {
            let name = ancestor.file_name()?.to_str()?;
            if rem.starts_with(JS_CONTRACTS_DIR) {
                Some((format!("{}/", name), ancestor.join(JS_CONTRACTS_DIR)))
            } else {
                Some((format!("{}/", name), ancestor.to_path_buf()))
            }
        }
    }
}

// the common ancestor is a `contracts` dir, in which case we assume the name of the
// remapping should be the dir name before the first higher up barrier:
// `dep/{a,b}/contracts` -> `dep`
// while `dep/contracts` will still be `dep/contracts`
fn last_valid_mapping(ancestor: &Path) -> Option<(String, PathBuf)> {
    let mut adjusted_remapping_root = None;
    let mut p = ancestor.parent()?;
    while let Some(parent) = p.parent() {
        let name = parent.file_name()?.to_str()?;
        if [DAPPTOOLS_CONTRACTS_DIR, DAPPTOOLS_LIB_DIR, "node_modules"].contains(&name) {
            break
        }
        p = parent;
        adjusted_remapping_root = Some(p);
    }
    let name = p.file_name()?.to_str()?;
    Some((format!("{}/", name), adjusted_remapping_root.unwrap_or(ancestor).to_path_buf()))
}

/// join the `base` path and all components of the `rem` path until a component is a barrier (src,
/// lib, contracts)
fn join_until_next_barrier(mut base: PathBuf, rem: &Path) -> PathBuf {
    for c in rem.components() {
        let s = c.as_os_str();
        base = base.join(s);
        if [DAPPTOOLS_CONTRACTS_DIR, DAPPTOOLS_LIB_DIR, JS_CONTRACTS_DIR]
            .contains(&c.as_os_str().to_string_lossy().as_ref())
        {
            break
        }
    }
    base
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let tmp_dir = tempdir::TempDir::new("lib").unwrap();
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
        let tmp_dir = tempdir::TempDir::new("lib").unwrap();
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
        let tmp_dir = tempdir::TempDir::new("tmp").unwrap();
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
    fn hardhat_remappings() {
        let tmp_dir = tempdir::TempDir::new("node_modules").unwrap();
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
}
