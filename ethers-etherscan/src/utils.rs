use crate::{contract::SourceCodeMetadata, EtherscanError, Result};
use ethers_core::types::Address;
use semver::Version;
use serde::{Deserialize, Deserializer};

static SOLC_BIN_LIST_URL: &str =
    "https://raw.githubusercontent.com/ethereum/solc-bin/gh-pages/bin/list.txt";

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
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        Ok(None)
    } else {
        let addr: Address = s.parse().map_err(serde::de::Error::custom)?;
        Ok(Some(addr))
    }
}

/// Deserializes as JSON:
///
/// `{ "SourceCode": "{{ .. }}", ..}`
///
/// or
///
/// `{ "SourceCode": "..", .. }`
pub fn deserialize_stringified_source_code<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> std::result::Result<SourceCodeMetadata, D::Error> {
    let s = String::deserialize(deserializer)?;
    if s.starts_with("{{") && s.ends_with("}}") {
        let s = &s[1..s.len() - 1];
        serde_json::from_str(s).map_err(serde::de::Error::custom)
    } else {
        Ok(SourceCodeMetadata::SourceCode(s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{contract::SourceCodeLanguage, tests::run_at_least_duration};
    use semver::{BuildMetadata, Prerelease};
    use serial_test::serial;
    use std::time::Duration;

    #[tokio::test]
    #[serial]
    async fn can_lookup_compiler_version_build_metadata() {
        run_at_least_duration(Duration::from_millis(250), async {
            let v = Version::new(0, 8, 13);
            let version = lookup_compiler_version(&v).await.unwrap();
            assert_eq!(v.major, version.major);
            assert_eq!(v.minor, version.minor);
            assert_eq!(v.patch, version.patch);
            assert_ne!(version.build, BuildMetadata::EMPTY);
            assert_eq!(version.pre, Prerelease::EMPTY);
        })
        .await
    }

    #[tokio::test]
    #[serial]
    async fn errors_on_invalid_solc() {
        run_at_least_duration(Duration::from_millis(250), async {
            let v = Version::new(100, 0, 0);
            let err = lookup_compiler_version(&v).await.unwrap_err();
            assert!(matches!(err, EtherscanError::MissingSolcVersion(_)));
        })
        .await
    }

    #[test]
    fn can_deserialize_address_opt() {
        #[derive(Deserialize)]
        struct Test {
            #[serde(deserialize_with = "deserialize_address_opt")]
            address: Option<Address>,
        }

        // https://api.etherscan.io/api?module=contract&action=getsourcecode&address=0xBB9bc244D798123fDe783fCc1C72d3Bb8C189413
        let json = r#"{"address":""}"#;
        let de: Test = serde_json::from_str(json).unwrap();
        assert_eq!(de.address, None);

        // https://api.etherscan.io/api?module=contract&action=getsourcecode&address=0xDef1C0ded9bec7F1a1670819833240f027b25EfF
        let json = r#"{"address":"0x4af649ffde640ceb34b1afaba3e0bb8e9698cb01"}"#;
        let de: Test = serde_json::from_str(json).unwrap();
        let expected = "0x4af649ffde640ceb34b1afaba3e0bb8e9698cb01".parse().unwrap();
        assert_eq!(de.address, Some(expected));
    }

    #[test]
    fn can_deserialize_stringified_source_code() {
        #[derive(Deserialize)]
        struct Test {
            #[serde(deserialize_with = "deserialize_stringified_source_code")]
            source_code: SourceCodeMetadata,
        }

        let src = "source code text";

        let json = r#"{
            "source_code": "{{ \"language\": \"Solidity\", \"sources\": {\"Contract\": { \"content\": \"source code text\" } } }}"
        }"#;
        let de: Test = serde_json::from_str(json).unwrap();
        assert!(matches!(de.source_code.language().unwrap(), SourceCodeLanguage::Solidity));
        assert_eq!(de.source_code.sources().len(), 1);
        assert_eq!(de.source_code.sources().get("Contract").unwrap().content, src);
        #[cfg(feature = "ethers-solc")]
        assert!(matches!(de.source_code.settings().unwrap(), None));

        let json = r#"{"source_code": "source code text"}"#;
        let de: Test = serde_json::from_str(json).unwrap();
        assert_eq!(de.source_code.source_code(), src);
    }
}
