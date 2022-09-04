use crate::{
    resolver::parse::{SolData, SolDataUnit, SolImport},
    utils, SolcError, Source,
};
use std::{
    fmt,
    path::{Path, PathBuf},
};

/// Represents a node (sol file) in the project graph
#[derive(Debug)]
pub struct Node {
    /// path of the solidity  file
    pub path: PathBuf,
    /// content of the solidity file
    pub source: Source,
    /// parsed data
    pub data: SolData,
}

impl Node {
    /// Reads the content of the file and returns a [Node] containing relevant information
    pub fn read(file: impl AsRef<Path>) -> crate::Result<Self> {
        let file = file.as_ref();
        let source = Source::read(file).map_err(|err| {
            if !err.path().exists() && err.path().is_symlink() {
                SolcError::ResolveBadSymlink(err)
            } else {
                SolcError::Resolve(err)
            }
        })?;
        let data = SolData::parse(source.as_ref(), file);
        Ok(Self { path: file.to_path_buf(), source, data })
    }

    pub fn content(&self) -> &str {
        &self.source.content
    }

    pub fn imports(&self) -> &Vec<SolDataUnit<SolImport>> {
        &self.data.imports
    }

    pub fn version(&self) -> &Option<SolDataUnit<String>> {
        &self.data.version
    }

    pub fn experimental(&self) -> &Option<SolDataUnit<String>> {
        &self.data.experimental
    }

    pub fn license(&self) -> &Option<SolDataUnit<String>> {
        &self.data.license
    }

    pub fn unpack(&self) -> (&PathBuf, &Source) {
        (&self.path, &self.source)
    }
}

/// Helper type for formatting a node
pub(crate) struct DisplayNode<'a> {
    pub node: &'a Node,
    pub root: &'a PathBuf,
}

impl<'a> fmt::Display for DisplayNode<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let path = utils::source_name(&self.node.path, self.root);
        write!(f, "{}", path.display())?;
        if let Some(ref v) = self.node.data.version {
            write!(f, " {}", v.data())?;
        }
        Ok(())
    }
}
