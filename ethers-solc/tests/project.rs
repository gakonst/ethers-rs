//! project tests

use ethers_solc::{cache::SOLIDITY_FILES_CACHE_FILENAME, Project, ProjectPathsConfig, Solc};
use std::{fs::create_dir_all, path::PathBuf};
use tempdir::TempDir;

fn solc() -> Solc {
    std::env::var("SOLC_PATH").map(Solc::new).unwrap_or_default()
}

#[test]
fn can_compile_project() {
    let tmp_dir = TempDir::new("contracts").unwrap();
    let cache = tmp_dir.path().join("cache");
    create_dir_all(&cache).unwrap();
    let cache = cache.join(SOLIDITY_FILES_CACHE_FILENAME);
    let artifacts = tmp_dir.path().join("artifacts");
    create_dir_all(&artifacts).unwrap();

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-data/sample");
    let paths =
        ProjectPathsConfig::builder().root(root).cache(cache).artifacts(artifacts).build().unwrap();

    // let paths = ProjectPathsConfig::builder()
    //     .root(root)
    //     .build().unwrap();

    let project = Project::builder().paths(paths).solc(solc()).build().unwrap();

    assert!(project.compile().unwrap().is_some());
    // nothing to compile
    assert!(project.compile().unwrap().is_none());
}
