//! project tests

use std::{
    collections::{HashMap, HashSet},
    io,
    path::{Path, PathBuf},
    str::FromStr,
};

use ethers_solc::{
    cache::{SolFilesCache, SOLIDITY_FILES_CACHE_FILENAME},
    project_util::*,
    remappings::Remapping,
    ConfigurableArtifacts, ExtraOutputValues, Graph, Project, ProjectCompileOutput,
    ProjectPathsConfig,
};
use pretty_assertions::assert_eq;

#[allow(unused)]
fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
}

#[test]
fn can_get_versioned_linkrefs() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-data/test-versioned-linkrefs");
    let paths = ProjectPathsConfig::builder()
        .sources(root.join("src"))
        .lib(root.join("lib"))
        .build()
        .unwrap();

    let project = Project::builder().paths(paths).ephemeral().no_artifacts().build().unwrap();

    let compiled = project.compile().unwrap();
    assert!(!compiled.has_compiler_errors());
}

#[test]
fn can_compile_hardhat_sample() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-data/hardhat-sample");
    let paths = ProjectPathsConfig::builder()
        .sources(root.join("contracts"))
        .lib(root.join("node_modules"));
    let project = TempProject::<ConfigurableArtifacts>::new(paths).unwrap();

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
    std::fs::remove_dir_all(&project.paths().artifacts).unwrap();
    let compiled = project.compile().unwrap();
    assert!(compiled.find("Greeter").is_some());
    assert!(compiled.find("console").is_some());
    assert!(!compiled.is_unchanged());
}

#[test]
fn can_compile_dapp_sample() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-data/dapp-sample");
    let paths = ProjectPathsConfig::builder().sources(root.join("src")).lib(root.join("lib"));
    let project = TempProject::<ConfigurableArtifacts>::new(paths).unwrap();

    let compiled = project.compile().unwrap();
    assert!(compiled.find("Dapp").is_some());
    assert!(!compiled.has_compiler_errors());

    // nothing to compile
    let compiled = project.compile().unwrap();
    assert!(compiled.find("Dapp").is_some());
    assert!(compiled.is_unchanged());

    let cache = SolFilesCache::read(project.cache_path()).unwrap();

    // delete artifacts
    std::fs::remove_dir_all(&project.paths().artifacts).unwrap();
    let compiled = project.compile().unwrap();
    assert!(compiled.find("Dapp").is_some());
    assert!(!compiled.is_unchanged());

    let updated_cache = SolFilesCache::read(project.cache_path()).unwrap();
    assert_eq!(cache, updated_cache);
}

#[test]
fn can_compile_configured() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-data/dapp-sample");
    let paths = ProjectPathsConfig::builder().sources(root.join("src")).lib(root.join("lib"));

    let handler = ConfigurableArtifacts {
        additional_values: ExtraOutputValues {
            metadata: true,
            ir: true,
            ir_optimized: true,
            ..Default::default()
        },
        ..Default::default()
    };

    let settings = handler.settings();
    let project = TempProject::with_artifacts(paths, handler).unwrap().with_settings(settings);
    let compiled = project.compile().unwrap();
    let artifact = compiled.find("Dapp").unwrap();
    assert!(artifact.metadata.is_some());
    assert!(artifact.ir.is_some());
    assert!(artifact.ir_optimized.is_some());
}

#[test]
fn can_compile_dapp_detect_changes_in_libs() {
    let mut project = TempProject::<ConfigurableArtifacts>::dapptools().unwrap();

    let remapping = project.paths().libraries[0].join("remapping");
    project
        .paths_mut()
        .remappings
        .push(Remapping::from_str(&format!("remapping={}/", remapping.display())).unwrap());

    let src = project
        .add_source(
            "Foo",
            r#"
    pragma solidity ^0.8.10;
    import "remapping/Bar.sol";

    contract Foo {}
   "#,
        )
        .unwrap();

    let lib = project
        .add_lib(
            "remapping/Bar",
            r#"
    pragma solidity ^0.8.10;

    contract Bar {}
    "#,
        )
        .unwrap();

    let graph = Graph::resolve(project.paths()).unwrap();
    assert_eq!(graph.files().len(), 2);
    assert_eq!(graph.files().clone(), HashMap::from([(src, 0), (lib, 1),]));

    let compiled = project.compile().unwrap();
    assert!(compiled.find("Foo").is_some());
    assert!(compiled.find("Bar").is_some());
    assert!(!compiled.has_compiler_errors());

    // nothing to compile
    let compiled = project.compile().unwrap();
    assert!(compiled.find("Foo").is_some());
    assert!(compiled.is_unchanged());

    let cache = SolFilesCache::read(&project.paths().cache).unwrap();
    assert_eq!(cache.files.len(), 2);

    // overwrite lib
    project
        .add_lib(
            "remapping/Bar",
            r#"
    pragma solidity ^0.8.10;

    // changed lib
    contract Bar {}
    "#,
        )
        .unwrap();

    let graph = Graph::resolve(project.paths()).unwrap();
    assert_eq!(graph.files().len(), 2);

    let compiled = project.compile().unwrap();
    assert!(compiled.find("Foo").is_some());
    assert!(compiled.find("Bar").is_some());
    // ensure change is detected
    assert!(!compiled.is_unchanged());
}

#[test]
fn can_compile_dapp_detect_changes_in_sources() {
    let project = TempProject::<ConfigurableArtifacts>::dapptools().unwrap();

    let src = project
        .add_source(
            "DssSpell.t",
            r#"
    pragma solidity ^0.8.10;
    import "./DssSpell.t.base.sol";

   contract DssSpellTest is DssSpellTestBase { }
   "#,
        )
        .unwrap();

    let base = project
        .add_source(
            "DssSpell.t.base",
            r#"
    pragma solidity ^0.8.10;

  contract DssSpellTestBase {
       address deployed_spell;
       function setUp() public {
           deployed_spell = address(0xA867399B43aF7790aC800f2fF3Fa7387dc52Ec5E);
       }
  }
   "#,
        )
        .unwrap();

    let graph = Graph::resolve(project.paths()).unwrap();
    assert_eq!(graph.files().len(), 2);
    assert_eq!(graph.files().clone(), HashMap::from([(base, 0), (src, 1),]));
    assert_eq!(graph.imported_nodes(1).to_vec(), vec![0]);

    let compiled = project.compile().unwrap();
    assert!(!compiled.has_compiler_errors());
    assert!(compiled.find("DssSpellTest").is_some());
    assert!(compiled.find("DssSpellTestBase").is_some());

    // nothing to compile
    let compiled = project.compile().unwrap();
    assert!(compiled.is_unchanged());
    assert!(compiled.find("DssSpellTest").is_some());
    assert!(compiled.find("DssSpellTestBase").is_some());

    let cache = SolFilesCache::read(&project.paths().cache).unwrap();
    assert_eq!(cache.files.len(), 2);

    let mut artifacts = compiled.into_artifacts().collect::<HashMap<_, _>>();

    // overwrite import
    let _ = project
        .add_source(
            "DssSpell.t.base",
            r#"
    pragma solidity ^0.8.10;

  contract DssSpellTestBase {
       address deployed_spell;
       function setUp() public {
           deployed_spell = address(0);
       }
  }
   "#,
        )
        .unwrap();
    let graph = Graph::resolve(project.paths()).unwrap();
    assert_eq!(graph.files().len(), 2);

    let compiled = project.compile().unwrap();
    assert!(compiled.find("DssSpellTest").is_some());
    assert!(compiled.find("DssSpellTestBase").is_some());
    // ensure change is detected
    assert!(!compiled.is_unchanged());

    // and all recompiled artifacts are different
    for (p, artifact) in compiled.into_artifacts() {
        let other = artifacts.remove(&p).unwrap();
        assert_ne!(artifact, other);
    }
}

#[test]
fn can_compile_dapp_sample_with_cache() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let root = tmp_dir.path();
    let cache = root.join("cache").join(SOLIDITY_FILES_CACHE_FILENAME);
    let artifacts = root.join("out");

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let orig_root = manifest_dir.join("test-data/dapp-sample");
    let cache_testdata_dir = manifest_dir.join("test-data/cache-sample/");
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
    std::fs::remove_dir_all(&project.artifacts_path()).unwrap();
    let compiled = project.compile().unwrap();
    assert!(compiled.find("Dapp").is_some());
    assert!(!compiled.is_unchanged());

    // new file is compiled even with partial cache
    std::fs::copy(cache_testdata_dir.join("NewContract.sol"), root.join("src/NewContract.sol"))
        .unwrap();
    let compiled = project.compile().unwrap();
    assert!(compiled.find("Dapp").is_some());
    assert!(compiled.find("NewContract").is_some());
    assert!(!compiled.is_unchanged());
    assert_eq!(
        compiled.into_artifacts().map(|(artifact_id, _)| artifact_id.name).collect::<HashSet<_>>(),
        HashSet::from([
            "Dapp".to_string(),
            "DappTest".to_string(),
            "DSTest".to_string(),
            "NewContract".to_string(),
        ])
    );

    // old cached artifact is not taken from the cache
    std::fs::copy(cache_testdata_dir.join("Dapp.sol"), root.join("src/Dapp.sol")).unwrap();
    let compiled = project.compile().unwrap();
    assert_eq!(
        compiled.into_artifacts().map(|(artifact_id, _)| artifact_id.name).collect::<HashSet<_>>(),
        HashSet::from([
            "DappTest".to_string(),
            "NewContract".to_string(),
            "DSTest".to_string(),
            "Dapp".to_string(),
        ])
    );

    // deleted artifact is not taken from the cache
    std::fs::remove_file(&project.paths.sources.join("Dapp.sol")).unwrap();
    let compiled: ProjectCompileOutput<_> = project.compile().unwrap();
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

#[test]
fn can_flatten_file() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-data/test-contract-libs");
    let target = root.join("src").join("Foo.sol");
    let paths = ProjectPathsConfig::builder()
        .sources(root.join("src"))
        .lib(root.join("lib1"))
        .lib(root.join("lib2"));
    let project = TempProject::<ConfigurableArtifacts>::new(paths).unwrap();

    let result = project.flatten(&target);
    assert!(result.is_ok());

    let result = result.unwrap();
    assert!(result.contains("contract Foo"));
    assert!(result.contains("contract Bar"));
}

#[test]
fn can_flatten_file_with_external_lib() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-data/hardhat-sample");
    let paths = ProjectPathsConfig::builder()
        .sources(root.join("contracts"))
        .lib(root.join("node_modules"));
    let project = TempProject::<ConfigurableArtifacts>::new(paths).unwrap();

    let target = root.join("contracts").join("Greeter.sol");

    let result = project.flatten(&target);
    assert!(result.is_ok());

    let result = result.unwrap();
    assert!(result.contains("library console"));
    assert!(result.contains("contract Greeter"));
}

#[test]
fn can_flatten_file_in_dapp_sample() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-data/dapp-sample");
    let paths = ProjectPathsConfig::builder().sources(root.join("src")).lib(root.join("lib"));
    let project = TempProject::<ConfigurableArtifacts>::new(paths).unwrap();

    let target = root.join("src/Dapp.t.sol");

    let result = project.flatten(&target);
    assert!(result.is_ok());

    let result = result.unwrap();
    assert!(result.contains("contract DSTest"));
    assert!(result.contains("contract Dapp"));
    assert!(result.contains("contract DappTest"));
}

#[test]
fn can_flatten_file_with_duplicates() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-data/flatten-sample");
    let paths = ProjectPathsConfig::builder().sources(root.join("contracts"));
    let project = TempProject::<ConfigurableArtifacts>::new(paths).unwrap();

    let target = root.join("contracts/FooBar.sol");

    let result = project.flatten(&target);
    assert!(result.is_ok());

    let result = result.unwrap();
    assert_eq!(result.matches("contract Foo {").count(), 1);
    assert_eq!(result.matches("contract Bar {").count(), 1);
    assert_eq!(result.matches("contract FooBar {").count(), 1);
    assert_eq!(result.matches(';').count(), 1);
}

#[test]
fn can_detect_type_error() {
    let project = TempProject::<ConfigurableArtifacts>::dapptools().unwrap();

    project
        .add_source(
            "Contract",
            r#"
    pragma solidity ^0.8.10;

   contract Contract {
        function xyz() public {
            require(address(0), "Error");
        }
   }
   "#,
        )
        .unwrap();

    let compiled = project.compile().unwrap();
    assert!(compiled.has_compiler_errors());
}

#[test]
fn can_compile_single_files() {
    let tmp = TempProject::dapptools().unwrap();

    let foo = tmp
        .add_contract(
            "examples/Foo",
            r#"
    pragma solidity ^0.8.10;

    contract Foo {}
   "#,
        )
        .unwrap();

    let compiled = tmp.project().compile_file(foo.clone()).unwrap();
    assert!(!compiled.has_compiler_errors());
    assert!(compiled.find("Foo").is_some());

    let bar = tmp
        .add_contract(
            "examples/Bar",
            r#"
    pragma solidity ^0.8.10;

    contract Bar {}
   "#,
        )
        .unwrap();

    let compiled = tmp.project().compile_files(vec![foo, bar]).unwrap();
    assert!(!compiled.has_compiler_errors());
    assert!(compiled.find("Foo").is_some());
    assert!(compiled.find("Bar").is_some());
}
