use crate::error::{CompilerError, Result};
use semver::Version;

pub use crate::compile::compilers::*;

use std::{io::BufRead, process::Output, str::FromStr};

pub mod report;

pub mod compilers;
pub mod contracts;
pub mod many;
pub mod output;
pub mod project;

pub fn compile_output(output: Output) -> Result<Vec<u8>> {
    if output.status.success() {
        Ok(output.stdout)
    } else {
        Err(CompilerError::solc(String::from_utf8_lossy(&output.stderr).to_string()))
    }
}

pub fn version_from_output(output: Output) -> Result<Version> {
    if output.status.success() {
        let version = output
            .stdout
            .lines()
            .last()
            .ok_or_else(|| CompilerError::solc("version not found in solc output"))?
            .map_err(|err| CompilerError::msg(format!("Failed to read output: {}", err)))?;
        // NOTE: semver doesn't like `+` in g++ in build metadata which is invalid semver
        Ok(Version::from_str(&version.trim_start_matches("Version: ").replace(".g++", ".gcc"))?)
    } else {
        Err(CompilerError::solc(String::from_utf8_lossy(&output.stderr).to_string()))
    }
}

#[cfg(test)]
mod tests {
    use semver::VersionReq;

    use super::*;
    use crate::{artifacts::Source, solc, CompilerInput};

    fn solc() -> solc::Solc {
        solc::Solc::default()
    }

    #[test]
    fn solc_version_works() {
        solc().version().unwrap();
    }

    #[test]
    fn can_parse_version_metadata() {
        let _version = Version::from_str("0.6.6+commit.6c089d02.Linux.gcc").unwrap();
    }

    #[cfg(feature = "async")]
    #[tokio::test]
    async fn async_solc_version_works() {
        let _version = solc().async_version().await.unwrap();
    }

    #[test]
    fn solc_compile_works() {
        let input = include_str!("../../test-data/in/compiler-in-1.json");
        let input: CompilerInput = serde_json::from_str(input).unwrap();
        let _out = solc().compile(&input).unwrap();
        // let other = solc().compile(&serde_json::json!(input)).unwrap();
        // assert_eq!(out, other);
    }

    #[test]
    fn solc_metadata_works() {
        let input = include_str!("../../test-data/in/compiler-in-1.json");
        let mut input: CompilerInput = serde_json::from_str(input).unwrap();
        input.settings.push_output_selection("metadata");
        let out = solc().compile(&input).unwrap();
        for (_, c) in out.split().1.contracts_iter() {
            assert!(c.metadata.is_some());
        }
    }

    #[cfg(feature = "async")]
    #[tokio::test]
    async fn async_solc_compile_works() {
        let input = include_str!("../../test-data/in/compiler-in-1.json");
        let input: CompilerInput = serde_json::from_str(input).unwrap();
        let out = solc().async_compile(&input).await.unwrap();
        let other = solc().async_compile(&serde_json::json!(input)).await.unwrap();
        assert_eq!(out, other);
    }
    #[cfg(feature = "async")]
    #[tokio::test]
    async fn async_solc_compile_works2() {
        let input = include_str!("../../test-data/in/compiler-in-2.json");
        let input: CompilerInput = serde_json::from_str(input).unwrap();
        let out = solc().async_compile(&input).await.unwrap();
        let other = solc().async_compile(&serde_json::json!(input)).await.unwrap();
        assert_eq!(out, other);
        let sync_out = solc().compile(&input).unwrap();
        assert_eq!(out, sync_out);
    }

    #[test]
    fn test_version_req() {
        let versions = ["=0.1.2", "^0.5.6", ">=0.7.1", ">0.8.0"];
        let sources = versions.iter().map(|version| source(version));

        sources.zip(versions).for_each(|(source, version)| {
            let version_req = solc::Solc::source_version_req(&source).unwrap();
            assert_eq!(version_req, VersionReq::from_str(version).unwrap());
        });

        // Solidity defines version ranges with a space, whereas the semver package
        // requires them to be separated with a comma
        let version_range = ">=0.8.0 <0.9.0";
        let source = source(version_range);
        let version_req = solc::Solc::source_version_req(&source).unwrap();
        assert_eq!(version_req, VersionReq::from_str(">=0.8.0,<0.9.0").unwrap());
    }

    #[test]
    // This test might be a bit hard to maintain
    #[cfg(all(feature = "svm", feature = "async"))]
    fn test_detect_version() {
        for (pragma, expected) in [
            // pinned
            ("=0.4.14", "0.4.14"),
            // pinned too
            ("0.4.14", "0.4.14"),
            // The latest patch is 0.4.26
            ("^0.4.14", "0.4.26"),
            // latest version above 0.5.0 -> we have to
            // update this test whenever there's a new sol
            // version. that's ok! good reminder to check the
            // patch notes.
            (">=0.5.0", "0.8.12"),
            // range
            (">=0.4.0 <0.5.0", "0.4.26"),
        ]
        .iter()
        {
            let source = source(pragma);
            let res = solc::Solc::detect_version(&source).unwrap();
            assert_eq!(res, Version::from_str(expected).unwrap());
        }
    }

    #[test]
    #[cfg(feature = "full")]
    fn test_find_installed_version_path() {
        // this test does not take the lock by default, so we need to manually
        // add it here.
        let _lock = crate::compilers::solc::LOCK.lock();
        let ver = "0.8.6";
        let version = Version::from_str(ver).unwrap();
        if crate::utils::installed_versions(svm::SVM_HOME.as_path())
            .map(|versions| !versions.contains(&version))
            .unwrap_or_default()
        {
            solc::Solc::blocking_install(&version).unwrap();
        }
        let res = solc::Solc::find_svm_installed_version(&version.to_string()).unwrap().unwrap();
        let expected = svm::SVM_HOME.join(ver).join(format!("solc-{}", ver));
        assert_eq!(res.solc, expected);
    }

    #[test]
    fn does_not_find_not_installed_version() {
        let ver = "1.1.1";
        let version = Version::from_str(ver).unwrap();
        let res = solc::Solc::find_svm_installed_version(&version.to_string()).unwrap();
        assert!(res.is_none());
    }

    #[test]
    fn test_find_latest_matching_installation() {
        let versions = ["0.4.24", "0.5.1", "0.5.2"]
            .iter()
            .map(|version| Version::from_str(version).unwrap())
            .collect::<Vec<_>>();

        let required = VersionReq::from_str(">=0.4.24").unwrap();

        let got = solc::Solc::find_matching_installation(&versions, &required).unwrap();
        assert_eq!(got, versions[2]);
    }

    #[test]
    fn test_no_matching_installation() {
        let versions = ["0.4.24", "0.5.1", "0.5.2"]
            .iter()
            .map(|version| Version::from_str(version).unwrap())
            .collect::<Vec<_>>();

        let required = VersionReq::from_str(">=0.6.0").unwrap();
        let got = solc::Solc::find_matching_installation(&versions, &required);
        assert!(got.is_none());
    }

    ///// helpers

    fn source(version: &str) -> Source {
        Source { content: format!("pragma solidity {};\n", version) }
    }
}
