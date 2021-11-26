//! project tests

use ethers_solc::{cache::SOLIDITY_FILES_CACHE_FILENAME, Project, ProjectPathsConfig};
use std::{
    io,
    path::{Path, PathBuf},
};
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
    assert!(compiled.find("Greeter").is_some());
    assert!(compiled.find("console").is_some());
    assert!(!compiled.has_compiler_errors());

    // nothing to compile
    let compiled = project.compile().unwrap();
    assert!(compiled.find("Greeter").is_some());
    assert!(compiled.find("console").is_some());
    assert!(compiled.is_unchanged());

    // delete artifacts
    std::fs::remove_dir_all(&project.paths.artifacts).unwrap();
    let compiled = project.compile().unwrap();
    assert!(compiled.find("Greeter").is_some());
    assert!(compiled.find("console").is_some());
    assert!(!compiled.is_unchanged());
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
    assert!(compiled.find("Dapp").is_some());
    assert!(!compiled.has_compiler_errors());

    // nothing to compile
    let compiled = project.compile().unwrap();
    assert!(compiled.find("Dapp").is_some());
    assert!(compiled.is_unchanged());

    // delete artifacts
    std::fs::remove_dir_all(&project.paths.artifacts).unwrap();
    let compiled = project.compile().unwrap();
    assert!(compiled.find("Dapp").is_some());
    assert!(!compiled.is_unchanged());
}

#[test]
fn can_compile_dapp_sample_with_cache() {
    let tmp_dir = TempDir::new("root").unwrap();
    let root = tmp_dir.path();
    let cache = root.join("cache").join(SOLIDITY_FILES_CACHE_FILENAME);
    let artifacts = root.join("out");

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let orig_root = manifest_dir.join("test-data/dapp-sample");
    let new_file = manifest_dir.join("test-data/cache-sample/NewContract.sol");
    copy_dir_all(orig_root, &tmp_dir).unwrap();
    let paths = ProjectPathsConfig::builder()
        .cache(cache)
        .sources(root.join("src"))
        .artifacts(artifacts)
        .lib(root.join("lib"))
        .root(root)
        .build()
        .unwrap();

    // first compile
    let project = Project::builder().paths(paths).build().unwrap();
    let compiled = project.compile().unwrap();
    assert!(compiled.find("Dapp").is_some());
    assert!(!compiled.has_compiler_errors());

    // cache is used when nothing to compile
    let compiled = project.compile().unwrap();
    assert!(compiled.find("Dapp").is_some());
    assert!(compiled.is_unchanged());

    // deleted artifacts cause recompile even with cache
    std::fs::remove_dir_all(&project.paths.artifacts).unwrap();
    let compiled = project.compile().unwrap();
    assert!(compiled.find("Dapp").is_some());
    assert!(!compiled.is_unchanged());

    // new file is compiled even with partial cache
    std::fs::copy(new_file, root.join("src/NewContract.sol")).unwrap();
    let compiled = project.compile().unwrap();
    assert!(compiled.find("Dapp").is_some());
    assert!(compiled.find("NewContract").is_some());
    assert!(!compiled.is_unchanged());

    // deleted artifact is not taken from the cache
    std::fs::remove_file(&project.paths.sources.join("Dapp.sol")).unwrap();
    let compiled = project.compile().unwrap();
    assert!(compiled.find("Dapp").is_none());
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    std::fs::create_dir_all(&dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}
