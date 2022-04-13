use crate::{utils, Solc};
use regex::Match;
use semver::VersionReq;
use solang_parser::pt::{
    ContractPart, ContractTy, FunctionAttribute, FunctionDefinition, Import, Loc, SourceUnitPart,
    Visibility,
};
use std::path::{Path, PathBuf};

/// Represents various information about a solidity file parsed via [solang_parser]
#[derive(Debug)]
#[allow(unused)]
pub struct SolData {
    pub license: Option<SolDataUnit<String>>,
    pub version: Option<SolDataUnit<String>>,
    pub experimental: Option<SolDataUnit<String>>,
    pub imports: Vec<SolDataUnit<PathBuf>>,
    pub version_req: Option<VersionReq>,
    pub libraries: Vec<SolLibrary>,
    pub contracts: Vec<SolContract>,
}

impl SolData {
    #[allow(unused)]
    pub fn fmt_version<W: std::fmt::Write>(
        &self,
        f: &mut W,
    ) -> std::result::Result<(), std::fmt::Error> {
        if let Some(ref version) = self.version {
            write!(f, "({})", version.data)?;
        }
        Ok(())
    }

    /// Extracts the useful data from a solidity source
    ///
    /// This will attempt to parse the solidity AST and extract the imports and version pragma. If
    /// parsing fails, we'll fall back to extract that info via regex
    pub fn parse(content: &str, file: &Path) -> Self {
        let mut version = None;
        let mut experimental = None;
        let mut imports = Vec::<SolDataUnit<PathBuf>>::new();
        let mut libraries = Vec::new();
        let mut contracts = Vec::new();

        match solang_parser::parse(content, 0) {
            Ok((units, _)) => {
                for unit in units.0 {
                    match unit {
                        SourceUnitPart::PragmaDirective(loc, _, pragma, value) => {
                            if pragma.name == "solidity" {
                                // we're only interested in the solidity version pragma
                                version = Some(SolDataUnit::new(value.string.clone(), loc.into()));
                            }

                            if pragma.name == "experimental" {
                                experimental =
                                    Some(SolDataUnit::new(value.string.clone(), loc.into()));
                            }
                        }
                        SourceUnitPart::ImportDirective(_, import) => {
                            let (import, loc) = match import {
                                Import::Plain(s, l) => (s, l),
                                Import::GlobalSymbol(s, _, l) => (s, l),
                                Import::Rename(s, _, l) => (s, l),
                            };
                            imports
                                .push(SolDataUnit::new(PathBuf::from(import.string), loc.into()));
                        }
                        SourceUnitPart::ContractDefinition(def) => {
                            let functions = def
                                .parts
                                .into_iter()
                                .filter_map(|part| match part {
                                    ContractPart::FunctionDefinition(f) => Some(*f),
                                    _ => None,
                                })
                                .collect();
                            let name = def.name.name;
                            match def.ty {
                                ContractTy::Contract(_) => {
                                    contracts.push(SolContract { name, functions });
                                }
                                ContractTy::Library(_) => {
                                    libraries.push(SolLibrary { name, functions });
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
            }
            Err(err) => {
                tracing::trace!(
                    "failed to parse \"{}\" ast: \"{:?}\". Falling back to regex to extract data",
                    file.display(),
                    err
                );
                version =
                    capture_outer_and_inner(content, &utils::RE_SOL_PRAGMA_VERSION, &["version"])
                        .first()
                        .map(|(cap, name)| {
                            SolDataUnit::new(name.as_str().to_owned(), cap.to_owned().into())
                        });
                imports = capture_imports(content);
            }
        };
        let license = content.lines().next().and_then(|line| {
            capture_outer_and_inner(line, &utils::RE_SOL_SDPX_LICENSE_IDENTIFIER, &["license"])
                .first()
                .map(|(cap, l)| SolDataUnit::new(l.as_str().to_owned(), cap.to_owned().into()))
        });
        let version_req = version.as_ref().and_then(|v| Solc::version_req(v.data()).ok());

        Self { version_req, version, experimental, imports, license, libraries, contracts }
    }

    /// Returns `true` if the solidity file associated with this type contains a solidity library
    /// that won't be inlined
    pub fn has_link_references(&self) -> bool {
        self.libraries.iter().any(|lib| !lib.is_inlined())
    }
}

/// Minimal representation of a contract inside a solidity file
#[derive(Debug)]
pub struct SolContract {
    pub name: String,
    pub functions: Vec<FunctionDefinition>,
}

/// Minimal representation of a contract inside a solidity file
#[derive(Debug)]
pub struct SolLibrary {
    pub name: String,
    pub functions: Vec<FunctionDefinition>,
}

impl SolLibrary {
    /// Returns `true` if all functions of this library will be inlined.
    ///
    /// This checks if all functions are either internal or private, because internal functions can
    /// only be accessed from within the current contract or contracts deriving from it. They cannot
    /// be accessed externally. Since they are not exposed to the outside through the contract’s
    /// ABI, they can take parameters of internal types like mappings or storage references.
    ///
    /// See also <https://docs.soliditylang.org/en/latest/contracts.html#libraries>
    pub fn is_inlined(&self) -> bool {
        for f in self.functions.iter() {
            for attr in f.attributes.iter() {
                if let FunctionAttribute::Visibility(vis) = attr {
                    match vis {
                        Visibility::External(_) | Visibility::Public(_) => return false,
                        _ => {}
                    }
                }
            }
        }
        true
    }
}

/// Represents an item in a solidity file with its location in the file
#[derive(Debug, Clone)]
pub struct SolDataUnit<T> {
    loc: Location,
    data: T,
}

/// Location in a text file buffer
#[derive(Debug, Clone)]
pub struct Location {
    pub start: usize,
    pub end: usize,
}

/// Solidity Data Unit decorated with its location within the file
impl<T> SolDataUnit<T> {
    pub fn new(data: T, loc: Location) -> Self {
        Self { data, loc }
    }

    /// Returns the underlying data for the unit
    pub fn data(&self) -> &T {
        &self.data
    }

    /// Returns the location of the given data unit
    pub fn loc(&self) -> (usize, usize) {
        (self.loc.start, self.loc.end)
    }

    /// Returns the location of the given data unit adjusted by an offset.
    /// Used to determine new position of the unit within the file after
    /// content manipulation.
    pub fn loc_by_offset(&self, offset: isize) -> (usize, usize) {
        (
            offset.saturating_add(self.loc.start as isize) as usize,
            // make the end location exclusive
            offset.saturating_add(self.loc.end as isize + 1) as usize,
        )
    }
}

impl From<Match<'_>> for Location {
    fn from(src: Match) -> Self {
        Location { start: src.start(), end: src.end() }
    }
}

impl From<Loc> for Location {
    fn from(src: Loc) -> Self {
        match src {
            Loc::File(_, start, end) => Location { start, end },
            _ => Location { start: 0, end: 0 },
        }
    }
}

/// Given the regex and the target string, find all occurrences
/// of named groups within the string. This method returns
/// the tuple of matches `(a, b)` where `a` is the match for the
/// entire regex and `b` is the match for the first named group.
///
/// NOTE: This method will return the match for the first named
/// group, so the order of passed named groups matters.
fn capture_outer_and_inner<'a>(
    content: &'a str,
    regex: &regex::Regex,
    names: &[&str],
) -> Vec<(regex::Match<'a>, regex::Match<'a>)> {
    regex
        .captures_iter(content)
        .filter_map(|cap| {
            let cap_match = names.iter().find_map(|name| cap.name(name));
            cap_match.and_then(|m| cap.get(0).map(|outer| (outer.to_owned(), m)))
        })
        .collect()
}

pub fn capture_imports(content: &str) -> Vec<SolDataUnit<PathBuf>> {
    capture_outer_and_inner(content, &utils::RE_SOL_IMPORT, &["p1", "p2", "p3", "p4"])
        .iter()
        .map(|(cap, m)| SolDataUnit::new(PathBuf::from(m.as_str()), cap.to_owned().into()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_capture_curly_imports() {
        let content = r#"
import { T } from "../Test.sol";
import {ReentrancyGuard} from "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import {DsTest} from "ds-test/test.sol";
"#;

        let captured_imports =
            capture_imports(content).into_iter().map(|s| s.data).collect::<Vec<_>>();

        let expected =
            utils::find_import_paths(content).map(|m| m.as_str().into()).collect::<Vec<PathBuf>>();

        assert_eq!(captured_imports, expected);

        assert_eq!(
            captured_imports,
            vec![
                PathBuf::from("../Test.sol"),
                "@openzeppelin/contracts/utils/ReentrancyGuard.sol".into(),
                "ds-test/test.sol".into(),
            ]
        );
    }
}
