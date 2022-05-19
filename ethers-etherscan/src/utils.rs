use semver::Version;

use crate::{EtherscanError, Result};

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
}
