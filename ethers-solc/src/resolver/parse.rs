use crate::{utils, Solc};
use semver::VersionReq;
use solang_parser::pt::{
    ContractPart, ContractTy, FunctionAttribute, FunctionDefinition, Import, Loc, SourceUnitPart,
    Visibility,
};
use std::{
    ops::Range,
    path::{Path, PathBuf},
};

/// Represents various information about a solidity file parsed via [solang_parser]
#[derive(Debug)]
#[allow(unused)]
pub struct SolData {
    pub license: Option<SolDataUnit<String>>,
    pub version: Option<SolDataUnit<String>>,
    pub experimental: Option<SolDataUnit<String>>,
    pub imports: Vec<SolDataUnit<SolImport>>,
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
        let mut imports = Vec::<SolDataUnit<SolImport>>::new();
        let mut libraries = Vec::new();
        let mut contracts = Vec::new();

        match solang_parser::parse(content, 0) {
            Ok((units, _)) => {
                for unit in units.0 {
                    match unit {
                        SourceUnitPart::PragmaDirective(loc, Some(pragma), Some(value)) => {
                            if pragma.name == "solidity" {
                                // we're only interested in the solidity version pragma
                                version = Some(SolDataUnit::from_loc(value.string.clone(), loc));
                            }

                            if pragma.name == "experimental" {
                                experimental = Some(SolDataUnit::from_loc(value.string, loc));
                            }
                        }
                        SourceUnitPart::ImportDirective(import) => {
                            let (import, ids, loc) = match import {
                                Import::Plain(s, l) => (s, vec![], l),
                                Import::GlobalSymbol(s, i, l) => (s, vec![(i, None)], l),
                                Import::Rename(s, i, l) => (s, i, l),
                            };
                            let sol_import = SolImport::new(PathBuf::from(import.string))
                                .set_aliases(
                                    ids.into_iter()
                                        .map(|(id, alias)| match alias {
                                            Some(al) => SolImportAlias::Contract(al.name, id.name),
                                            None => SolImportAlias::File(id.name),
                                        })
                                        .collect(),
                                );
                            imports.push(SolDataUnit::from_loc(sol_import, loc));
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
                            if let Some(name) = def.name {
                                match def.ty {
                                    ContractTy::Contract(_) => {
                                        contracts.push(SolContract { name: name.name, functions });
                                    }
                                    ContractTy::Library(_) => {
                                        libraries.push(SolLibrary { name: name.name, functions });
                                    }
                                    _ => {}
                                }
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
                        .map(|(cap, name)| SolDataUnit::new(name.as_str().to_owned(), cap.range()));
                imports = capture_imports(content);
            }
        };
        let license = content.lines().next().and_then(|line| {
            capture_outer_and_inner(line, &utils::RE_SOL_SDPX_LICENSE_IDENTIFIER, &["license"])
                .first()
                .map(|(cap, l)| SolDataUnit::new(l.as_str().to_owned(), cap.range()))
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

#[derive(Debug, Clone)]
pub struct SolImport {
    path: PathBuf,
    aliases: Vec<SolImportAlias>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SolImportAlias {
    File(String),
    Contract(String, String),
}

impl SolImport {
    pub fn new(path: PathBuf) -> Self {
        Self { path, aliases: vec![] }
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn aliases(&self) -> &Vec<SolImportAlias> {
        &self.aliases
    }

    fn set_aliases(mut self, aliases: Vec<SolImportAlias>) -> Self {
        self.aliases = aliases;
        self
    }
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
    loc: Range<usize>,
    data: T,
}

/// Solidity Data Unit decorated with its location within the file
impl<T> SolDataUnit<T> {
    pub fn new(data: T, loc: Range<usize>) -> Self {
        Self { data, loc }
    }

    pub fn from_loc(data: T, loc: Loc) -> Self {
        Self {
            data,
            loc: match loc {
                Loc::File(_, start, end) => Range { start, end: end + 1 },
                _ => Range { start: 0, end: 0 },
            },
        }
    }

    /// Returns the underlying data for the unit
    pub fn data(&self) -> &T {
        &self.data
    }

    /// Returns the location of the given data unit
    pub fn loc(&self) -> Range<usize> {
        self.loc.clone()
    }

    /// Returns the location of the given data unit adjusted by an offset.
    /// Used to determine new position of the unit within the file after
    /// content manipulation.
    pub fn loc_by_offset(&self, offset: isize) -> Range<usize> {
        utils::range_by_offset(&self.loc, offset)
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
/// Capture the import statement information together with aliases
pub fn capture_imports(content: &str) -> Vec<SolDataUnit<SolImport>> {
    let mut imports = vec![];
    for cap in utils::RE_SOL_IMPORT.captures_iter(content) {
        if let Some(name_match) = ["p1", "p2", "p3", "p4"].iter().find_map(|name| cap.name(name)) {
            let statement_match = cap.get(0).unwrap();
            let mut aliases = vec![];
            for alias_cap in utils::RE_SOL_IMPORT_ALIAS.captures_iter(statement_match.as_str()) {
                if let Some(alias) = alias_cap.name("alias") {
                    let alias = alias.as_str().to_owned();
                    let import_alias = match alias_cap.name("target") {
                        Some(target) => SolImportAlias::Contract(alias, target.as_str().to_owned()),
                        None => SolImportAlias::File(alias),
                    };
                    aliases.push(import_alias);
                }
            }
            let sol_import =
                SolImport::new(PathBuf::from(name_match.as_str())).set_aliases(aliases);
            imports.push(SolDataUnit::new(sol_import, statement_match.range()));
        }
    }
    imports
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
            capture_imports(content).into_iter().map(|s| s.data.path).collect::<Vec<_>>();

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

    #[test]
    fn cap_capture_aliases() {
        let content = r#"
import * as T from "./Test.sol";
import { DsTest as Test } from "ds-test/test.sol";
import "ds-test/test.sol" as Test;
import { FloatMath as Math, Math as FloatMath } from "./Math.sol";
"#;

        let caputred_imports =
            capture_imports(content).into_iter().map(|s| s.data.aliases).collect::<Vec<_>>();
        assert_eq!(
            caputred_imports,
            vec![
                vec![SolImportAlias::File("T".into())],
                vec![SolImportAlias::Contract("Test".into(), "DsTest".into())],
                vec![SolImportAlias::File("Test".into())],
                vec![
                    SolImportAlias::Contract("Math".into(), "FloatMath".into()),
                    SolImportAlias::Contract("FloatMath".into(), "Math".into()),
                ],
            ]
        );
    }
}
