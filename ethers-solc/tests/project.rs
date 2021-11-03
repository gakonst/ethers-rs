//! project tests

use ethers_solc::{
    cache::SOLIDITY_FILES_CACHE_FILENAME, Project, ProjectCompileOutput, ProjectPathsConfig,
};
use std::path::PathBuf;
use tempdir::TempDir;

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

    let project = Project::builder().paths(paths).build().unwrap();
    let compiled = project.compile().unwrap();
    match compiled {
        ProjectCompileOutput::Compiled((out, _)) => assert!(!out.has_error()),
        _ => panic!("must compile"),
    }
    // nothing to compile
    assert_eq!(project.compile().unwrap(), ProjectCompileOutput::Unchanged);
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

    let project = Project::builder().paths(paths).build().unwrap();
    let compiled = project.compile().unwrap();
    match compiled {
        ProjectCompileOutput::Compiled((out, _)) => assert!(!out.has_error()),
        _ => panic!("must compile"),
    }
    // nothing to compile
    assert_eq!(project.compile().unwrap(), ProjectCompileOutput::Unchanged);
}
