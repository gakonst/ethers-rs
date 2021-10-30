//! project tests

use ethers_solc::{cache::SOLIDITY_FILES_CACHE_FILENAME, Project, ProjectPathsConfig, Solc};
use std::path::PathBuf;
use tempdir::TempDir;

fn solc() -> Solc {
    std::env::var("SOLC_PATH").map(Solc::new).unwrap_or_default()
}

#[test]
fn can_compile_hardhat_sample() {
    let tmp_dir = TempDir::new("root").unwrap();
    let cache = tmp_dir.path().join("cache");
    let cache = cache.join(SOLIDITY_FILES_CACHE_FILENAME);
    let artifacts = tmp_dir.path().join("artifacts");

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-data/hardhat-sample");
    let paths = ProjectPathsConfig::builder()
        .cache(cache)
        .sources(root.join("contracts"))
        .artifacts(artifacts)
        .lib(root.join("node_modules"))
        .root(root)
        .build()
        .unwrap();
    // let paths = ProjectPathsConfig::hardhat(root).unwrap();

    let project = Project::builder().paths(paths).solc(solc()).build().unwrap();
    assert!(project.compile().unwrap().is_some());
    // nothing to compile
    assert!(project.compile().unwrap().is_none());
}

#[test]
fn can_compile_dapp_sample() {
    let tmp_dir = TempDir::new("root").unwrap();
    let cache = tmp_dir.path().join("cache");
    let cache = cache.join(SOLIDITY_FILES_CACHE_FILENAME);
    let artifacts = tmp_dir.path().join("out");

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-data/dapp-sample");
    let paths = ProjectPathsConfig::builder()
        .cache(cache)
        .sources(root.join("src"))
        .artifacts(artifacts)
        .lib(root.join("lib"))
        .root(root)
        .build()
        .unwrap();
    // let paths = ProjectPathsConfig::dapptools(root).unwrap();

    let project = Project::builder().paths(paths).solc(solc()).build().unwrap();
    assert!(project.compile().unwrap().is_some());
    // nothing to compile
    assert!(project.compile().unwrap().is_none());
}
