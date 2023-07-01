use crate::{contract::SourceCodeMetadata, EtherscanError, Result};
use ethers_core::types::Address;
use semver::Version;
use serde::{Deserialize, Deserializer};

static SOLC_BIN_LIST_URL: &str = "https://binaries.soliditylang.org/bin/list.txt";

/// Given a Solc [Version], lookup the build metadata and return the full SemVer.
/// e.g. `0.8.13` -> `0.8.13+commit.abaa5c0e`
pub async fn lookup_compiler_version(version: &Version) -> Result<Version> {
    let response = reqwest::get(SOLC_BIN_LIST_URL).await?.text().await?;
    // Ignore extra metadata (`pre` or `build`)
    let version = format!("{}.{}.{}", version.major, version.minor, version.patch);
    let v = response
        .lines()
        .find(|l| !l.contains("nightly") && l.contains(&version))
        .map(|l| l.trim_start_matches("soljson-v").trim_end_matches(".js"))
        .ok_or_else(|| EtherscanError::MissingSolcVersion(version))?;

    Ok(v.parse().expect("failed to parse semver"))
}

/// Return None if empty, otherwise parse as [Address].
pub fn deserialize_address_opt<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> std::result::Result<Option<Address>, D::Error> {
    match Option::<String>::deserialize(deserializer)? {
        None => Ok(None),
        Some(s) => match s.is_empty() {
            true => Ok(None),
            _ => Ok(Some(s.parse().map_err(serde::de::Error::custom)?)),
        },
    }
}

/// Deserializes as JSON either:
///
/// - Object: `{ "SourceCode": { language: "Solidity", .. }, ..}`
/// - Stringified JSON object:
///     - `{ "SourceCode": "{{\r\n  \"language\": \"Solidity\", ..}}", ..}`
///     - `{ "SourceCode": "{ \"file.sol\": \"...\" }", ... }`
/// - Normal source code string: `{ "SourceCode": "// SPDX-License-Identifier: ...", .. }`
pub fn deserialize_source_code<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> std::result::Result<SourceCodeMetadata, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum SourceCode {
        String(String), // this must come first
        Obj(SourceCodeMetadata),
    }
    let s = SourceCode::deserialize(deserializer)?;
    match s {
        SourceCode::String(s) => {
            if s.starts_with('{') && s.ends_with('}') {
                let mut s = s.as_str();
                // skip double braces
                if s.starts_with("{{") && s.ends_with("}}") {
                    s = &s[1..s.len() - 1];
                }
                serde_json::from_str(s).map_err(serde::de::Error::custom)
            } else {
                Ok(SourceCodeMetadata::SourceCode(s))
            }
        }
        SourceCode::Obj(obj) => Ok(obj),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::SourceCodeLanguage;

    #[test]
    fn can_deserialize_address_opt() {
        #[derive(serde::Serialize, Deserialize)]
        struct Test {
            #[serde(deserialize_with = "deserialize_address_opt")]
            address: Option<Address>,
        }

        // https://api.etherscan.io/api?module=contract&action=getsourcecode&address=0xBB9bc244D798123fDe783fCc1C72d3Bb8C189413
        let json = r#"{"address":""}"#;
        let de: Test = serde_json::from_str(json).unwrap();
        assert_eq!(de.address, None);

        // Round-trip the above
        let json = serde_json::to_string(&de).unwrap();
        let de: Test = serde_json::from_str(&json).unwrap();
        assert_eq!(de.address, None);

        // https://api.etherscan.io/api?module=contract&action=getsourcecode&address=0xDef1C0ded9bec7F1a1670819833240f027b25EfF
        let json = r#"{"address":"0x4af649ffde640ceb34b1afaba3e0bb8e9698cb01"}"#;
        let de: Test = serde_json::from_str(json).unwrap();
        let expected = "0x4af649ffde640ceb34b1afaba3e0bb8e9698cb01".parse().unwrap();
        assert_eq!(de.address, Some(expected));
    }

    #[test]
    fn can_deserialize_source_code() {
        #[derive(Deserialize)]
        struct Test {
            #[serde(deserialize_with = "deserialize_source_code")]
            source_code: SourceCodeMetadata,
        }

        let src = "source code text";

        // Normal JSON
        let json = r#"{
            "source_code": { "language": "Solidity", "sources": { "Contract": { "content": "source code text" } } }
        }"#;
        let de: Test = serde_json::from_str(json).unwrap();
        assert!(matches!(de.source_code.language().unwrap(), SourceCodeLanguage::Solidity));
        assert_eq!(de.source_code.sources().len(), 1);
        assert_eq!(de.source_code.sources().get("Contract").unwrap().content, src);
        #[cfg(feature = "ethers-solc")]
        assert!(de.source_code.settings().unwrap().is_none());

        // Stringified JSON
        let json = r#"{
            "source_code": "{{ \"language\": \"Solidity\", \"sources\": { \"Contract\": { \"content\": \"source code text\" } } }}"
        }"#;
        let de: Test = serde_json::from_str(json).unwrap();
        assert!(matches!(de.source_code.language().unwrap(), SourceCodeLanguage::Solidity));
        assert_eq!(de.source_code.sources().len(), 1);
        assert_eq!(de.source_code.sources().get("Contract").unwrap().content, src);
        #[cfg(feature = "ethers-solc")]
        assert!(de.source_code.settings().unwrap().is_none());

        let json = r#"{"source_code": "source code text"}"#;
        let de: Test = serde_json::from_str(json).unwrap();
        assert_eq!(de.source_code.source_code(), src);
    }
}
