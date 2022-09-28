use crate::{EtherscanError, Result};
use ethers_core::types::Address;
use semver::Version;
use serde::{Deserialize, Deserializer};

static SOLC_BIN_LIST_URL: &str =
    "https://raw.githubusercontent.com/ethereum/solc-bin/gh-pages/bin/list.txt";

/// Given the compiler version  lookup the build metadata
/// and return full semver
/// i.e. `0.8.13` -> `0.8.13+commit.abaa5c0e`
pub async fn lookup_compiler_version(version: &Version) -> Result<Version> {
    let response = reqwest::get(SOLC_BIN_LIST_URL).await?.text().await?;
    let version = format!("{}", version);
    let v = response
        .lines()
        .find(|l| !l.contains("nightly") && l.contains(&version))
        .map(|l| l.trim_start_matches("soljson-v").trim_end_matches(".js").to_owned())
        .ok_or(EtherscanError::MissingSolcVersion(version))?;

    Ok(v.parse().expect("failed to parse semver"))
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::run_at_least_duration;
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
    fn can_deserialize_versions() {
        #[derive(Deserialize)]
        struct Test {
            #[serde(deserialize_with = "deserialize_version")]
            version: Version,
        }

        // https://api.etherscan.io/api?module=contract&action=getsourcecode&address=0xBB9bc244D798123fDe783fCc1C72d3Bb8C189413
        let json = r#"{"version":"v0.3.1-2016-04-12-3ad5e82"}"#;
        let de: Test = serde_json::from_str(json).unwrap();
        let mut expected = Version::new(0, 3, 1);
        expected.pre = Prerelease::new("2016-04-12-3ad5e82").unwrap();
        assert_eq!(de.version, expected);

        // https://api.etherscan.io/api?module=contract&action=getsourcecode&address=0xDef1C0ded9bec7F1a1670819833240f027b25EfF
        let json = r#"{"version":"v0.6.8+commit.0bbfe453"}"#;
        let de: Test = serde_json::from_str(json).unwrap();
        let mut expected = Version::new(0, 6, 8);
        expected.build = BuildMetadata::new("commit.0bbfe453").unwrap();
        assert_eq!(de.version, expected);

        // https://api.etherscan.io/api?module=contract&action=getsourcecode&address=0xdf5e0e81dff6faf3a7e52ba697820c5e32d806a8
        let json = r#"{"version":"vyper:0.1.0b16"}"#;
        let de: Test = serde_json::from_str(json).unwrap();
        let mut expected = Version::new(0, 1, 0);
        expected.pre = Prerelease::new("beta.16").unwrap();
        assert_eq!(de.version, expected);

        // https://api.etherscan.io/api?module=contract&action=getsourcecode&address=0x4f62af8ff4b9b22f53ee56cb576b02efe2866825
        let json = r#"{"version":"vyper:0.3.6"}"#;
        let de: Test = serde_json::from_str(json).unwrap();
        let expected = Version::new(0, 3, 6);
        assert_eq!(de.version, expected);
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
}
