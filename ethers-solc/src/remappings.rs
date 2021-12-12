use crate::{error::SolcError, Result};
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

const DAPPTOOLS_CONTRACTS_DIR: &str = "src";
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
        let mut split = remapping.split('=');
        let name = split.next().ok_or(RemappingError::NoPrefix)?.to_string();
        if name.is_empty() {
            return Err(RemappingError::NoPrefix)
        }
        let path = split.next().ok_or(RemappingError::NoTarget)?.to_string();
        if path.is_empty() {
            return Err(RemappingError::NoTarget)
        }
        Ok(Remapping { name, path })
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
    /// Detects a remapping prioritizing Dapptools-style remappings over `contracts/`-style ones.
    fn find(root: &str) -> Result<Self> {
        Self::find_with_type(root, DAPPTOOLS_CONTRACTS_DIR)
            .or_else(|_| Self::find_with_type(root, JS_CONTRACTS_DIR))
    }

    /// Given a path and the style of contracts dir, it proceeds to find
    /// a `Remapping` for it.
    fn find_with_type(name: &str, source: &str) -> Result<Self> {
        let pattern = if name.contains(source) {
            format!("{}/**/*.sol", name)
        } else {
            format!("{}/{}/**/*.sol", name, source)
        };
        let mut dapptools_contracts = glob::glob(&pattern)?;
        let next = dapptools_contracts.next();
        if next.is_some() {
            let path = format!("{}/{}/", name, source);
            let mut name = name.split('/').last().unwrap().to_string();
            name.push('/');
            Ok(Remapping { name, path })
        } else {
            Err(SolcError::NoContracts(source.to_string()))
        }
    }

    pub fn find_many_str(path: &str) -> Result<Vec<String>> {
        let remappings = Self::find_many(path)?;
        Ok(remappings.iter().map(|mapping| format!("{}={}", mapping.name, mapping.path)).collect())
    }

    /// Gets all the remappings detected
    pub fn find_many(path: impl AsRef<std::path::Path>) -> Result<Vec<Self>> {
        let path = path.as_ref();
        if !path.exists() {
            // nothing to find
            return Ok(Vec::new())
        }
        let mut paths = std::fs::read_dir(path)
            .map_err(|err| SolcError::io(err, path))?
            .into_iter()
            .collect::<Vec<_>>();

        let mut remappings = Vec::new();
        while let Some(p) = paths.pop() {
            let path = p.map_err(|err| SolcError::io(err, path))?.path();

            // get all the directories inside a file if it's a valid dir
            if let Ok(dir) = std::fs::read_dir(&path) {
                for inner in dir {
                    let inner = inner.map_err(|err| SolcError::io(err, &path))?;
                    let path = inner.path().display().to_string();
                    let path = path.rsplit('/').next().unwrap().to_string();
                    if path != DAPPTOOLS_CONTRACTS_DIR && path != JS_CONTRACTS_DIR {
                        paths.push(Ok(inner));
                    }
                }
            }

            let remapping = Self::find(&path.display().to_string());
            if let Ok(remapping) = remapping {
                // skip remappings that exist already
                if let Some(ref mut found) =
                    remappings.iter_mut().find(|x: &&mut Remapping| x.name == remapping.name)
                {
                    // always replace with the shortest length path
                    fn depth(path: &str, delim: char) -> usize {
                        path.matches(delim).count()
                    }
                    // if the one which exists is larger, we should replace it
                    // if not, ignore it
                    if depth(&found.path, '/') > depth(&remapping.path, '/') {
                        **found = remapping;
                    }
                } else {
                    remappings.push(remapping);
                }
            }
        }

        Ok(remappings)
    }
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
        Remapping::find_with_type(&path, JS_CONTRACTS_DIR).unwrap_err();
        let remapping = Remapping::find_with_type(&path, DAPPTOOLS_CONTRACTS_DIR).unwrap();

        // repo1/=lib/repo1/src
        assert_eq!(remapping.name, "repo1/");
        assert_eq!(remapping.path, format!("{}/src/", path));
    }

    #[test]
    fn recursive_remappings() {
        //let tmp_dir_path = PathBuf::from("."); // tempdir::TempDir::new("lib").unwrap();
        let tmp_dir = tempdir::TempDir::new("lib").unwrap();
        let tmp_dir_path = tmp_dir.path();
        let paths = [
            "repo1/src/",
            "repo1/src/contract.sol",
            "repo1/lib/",
            "repo1/lib/ds-math/src/",
            "repo1/lib/ds-math/src/contract.sol",
            "repo1/lib/ds-math/lib/ds-test/src/",
            "repo1/lib/ds-math/lib/ds-test/src/test.sol",
        ];
        mkdir_or_touch(tmp_dir_path, &paths[..]);

        let path = tmp_dir_path.display().to_string();
        let mut remappings = Remapping::find_many(&path).unwrap();
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
                path: to_str(
                    tmp_dir_path
                        .join("repo1")
                        .join("lib")
                        .join("ds-math")
                        .join("lib")
                        .join("ds-test")
                        .join("src"),
                ),
            },
        ];
        expected.sort_unstable();
        assert_eq!(remappings, expected);
    }

    #[test]
    fn remappings() {
        let tmp_dir = tempdir::TempDir::new("lib").unwrap();
        let repo1 = tmp_dir.path().join("src_repo");
        let repo2 = tmp_dir.path().join("contracts_repo");

        let dir1 = repo1.join("src");
        std::fs::create_dir_all(&dir1).unwrap();

        let dir2 = repo2.join("contracts");
        std::fs::create_dir_all(&dir2).unwrap();

        let contract1 = dir1.join("contract.sol");
        touch(&contract1).unwrap();

        let contract2 = dir2.join("contract.sol");
        touch(&contract2).unwrap();

        let path = tmp_dir.path().display().to_string();
        let mut remappings = Remapping::find_many(&path).unwrap();
        remappings.sort_unstable();
        let mut expected = vec![
            Remapping {
                name: "src_repo/".to_string(),
                path: format!("{}/", dir1.into_os_string().into_string().unwrap()),
            },
            Remapping {
                name: "contracts_repo/".to_string(),
                path: format!("{}/", dir2.into_os_string().into_string().unwrap()),
            },
        ];
        expected.sort_unstable();
        assert_eq!(remappings, expected);
    }
}
