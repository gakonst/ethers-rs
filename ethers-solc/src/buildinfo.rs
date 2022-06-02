//! Represents an entire build

use crate::{utils, CompilerInput, CompilerOutput, SolcError};
use semver::Version;
use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};
use std::path::Path;

pub const ETHERS_FORMAT_VERSION: &str = "ethers-rs-sol-build-info-1";

// A hardhat compatible build info representation
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildInfo {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "_format")]
    pub format: String,
    pub solc_version: Version,
    pub solc_long_version: Version,
    pub input: CompilerInput,
    pub output: CompilerOutput,
}

impl BuildInfo {
    /// Deserializes the `BuildInfo` object from the given file
    pub fn read(path: impl AsRef<Path>) -> Result<Self, SolcError> {
        utils::read_json_file(path)
    }

    /// Serializes a `BuildInfo` object as String
    pub fn serialize_to_string(
        input: &CompilerInput,
        output: &CompilerOutput,
        version: &Version,
    ) -> serde_json::Result<String> {
        let mut w = Vec::with_capacity(128);
        let mut serializer = serde_json::Serializer::pretty(&mut w);
        let mut s = serializer.serialize_struct("BuildInfo", 5)?;
        s.serialize_field("_format", &ETHERS_FORMAT_VERSION)?;
        let solc_short = format!("{}.{}.{}", version.major, version.minor, version.patch);
        s.serialize_field("solcVersion", &solc_short)?;
        s.serialize_field("solcLongVersion", &version)?;
        s.serialize_field("input", input)?;
        s.serialize_field("output", output)?;
        s.end()?;

        let string = unsafe {
            // serde_json does not emit non UTF8
            String::from_utf8_unchecked(w)
        };
        Ok(string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Source;
    use std::{collections::BTreeMap, path::PathBuf};

    #[test]
    fn build_info_serde() {
        let inputs = CompilerInput::with_sources(BTreeMap::from([(
            PathBuf::from("input.sol"),
            Source { content: "".to_string() },
        )]));
        let output = CompilerOutput::default();
        let v: Version = "0.8.4+commit.c7e474f2".parse().unwrap();
        let content = BuildInfo::serialize_to_string(&inputs[0], &output, &v).unwrap();
        let _info: BuildInfo = serde_json::from_str(&content).unwrap();
    }
}
