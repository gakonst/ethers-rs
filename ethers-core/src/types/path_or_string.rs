use std::path::{Path, PathBuf};

/// A type that can either be a `Path` or a `String`
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PathOrString {
    /// A path type
    Path(PathBuf),
    /// A string type
    String(String),
}

impl From<PathBuf> for PathOrString {
    fn from(p: PathBuf) -> Self {
        PathOrString::Path(p)
    }
}

impl From<&str> for PathOrString {
    fn from(s: &str) -> Self {
        let path = Path::new(s);
        if path.exists() {
            PathOrString::Path(path.to_owned())
        } else {
            PathOrString::String(s.to_owned())
        }
    }
}

impl PathOrString {
    /// Reads the contents at path, or simply returns the string.
    pub fn read(&self) -> Result<String, std::io::Error> {
        match self {
            PathOrString::Path(pathbuf) => std::fs::read_to_string(pathbuf),
            PathOrString::String(s) => Ok(s.to_string()),
        }
    }
}
