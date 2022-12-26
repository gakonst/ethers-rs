//! project tests

use ethers_core::types::Address;
use ethers_solc::{
    artifacts::{
        BytecodeHash, DevDoc, ErrorDoc, EventDoc, Libraries, MethodDoc, ModelCheckerEngine::CHC,
        ModelCheckerSettings, UserDoc, UserDocNotice,
    },
    buildinfo::BuildInfo,
    cache::{SolFilesCache, SOLIDITY_FILES_CACHE_FILENAME},
    info::ContractInfo,
    project_util::*,
    remappings::Remapping,
    Artifact, CompilerInput, ConfigurableArtifacts, ExtraOutputValues, Graph, Project,
    ProjectCompileOutput, ProjectPathsConfig, Solc, TestFileFilter,
};
use pretty_assertions::assert_eq;
use semver::Version;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs, io,
    path::{Path, PathBuf},
    str::FromStr,
};

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
    assert!(compiled.find_first("Greeter").is_some());
    assert!(compiled.find_first("console").is_some());
    assert!(!compiled.has_compiler_errors());

    // nothing to compile
    let compiled = project.compile().unwrap();
    assert!(compiled.find_first("Greeter").is_some());
    assert!(compiled.find_first("console").is_some());
    assert!(compiled.is_unchanged());

    // delete artifacts
    std::fs::remove_dir_all(&project.paths().artifacts).unwrap();
    let compiled = project.compile().unwrap();
    assert!(compiled.find_first("Greeter").is_some());
    assert!(compiled.find_first("console").is_some());
    assert!(!compiled.is_unchanged());
}

#[test]
fn can_compile_dapp_sample() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-data/dapp-sample");
    let paths = ProjectPathsConfig::builder().sources(root.join("src")).lib(root.join("lib"));
    let project = TempProject::<ConfigurableArtifacts>::new(paths).unwrap();

    let compiled = project.compile().unwrap();
    assert!(compiled.find_first("Dapp").is_some());
    assert!(!compiled.has_compiler_errors());

    // nothing to compile
    let compiled = project.compile().unwrap();
    assert!(compiled.find_first("Dapp").is_some());
    assert!(compiled.is_unchanged());

    let cache = SolFilesCache::read(project.cache_path()).unwrap();

    // delete artifacts
    std::fs::remove_dir_all(&project.paths().artifacts).unwrap();
    let compiled = project.compile().unwrap();
    assert!(compiled.find_first("Dapp").is_some());
    assert!(!compiled.is_unchanged());

    let updated_cache = SolFilesCache::read(project.cache_path()).unwrap();
    assert_eq!(cache, updated_cache);
}

#[test]
fn can_compile_yul_sample() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-data/yul-sample");
    let paths = ProjectPathsConfig::builder().sources(root);
    let project = TempProject::<ConfigurableArtifacts>::new(paths).unwrap();

    let compiled = project.compile().unwrap();
    assert!(compiled.find_first("Dapp").is_some());
    assert!(compiled.find_first("SimpleStore").is_some());
    assert!(!compiled.has_compiler_errors());

    // nothing to compile
    let compiled = project.compile().unwrap();
    assert!(compiled.find_first("Dapp").is_some());
    assert!(compiled.find_first("SimpleStore").is_some());
    assert!(compiled.is_unchanged());

    let cache = SolFilesCache::read(project.cache_path()).unwrap();

    // delete artifacts
    std::fs::remove_dir_all(&project.paths().artifacts).unwrap();
    let compiled = project.compile().unwrap();
    assert!(compiled.find_first("Dapp").is_some());
    assert!(compiled.find_first("SimpleStore").is_some());
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
            opcodes: true,
            ..Default::default()
        },
        ..Default::default()
    };

    let settings = handler.settings();
    let project = TempProject::with_artifacts(paths, handler).unwrap().with_settings(settings);
    let compiled = project.compile().unwrap();
    let artifact = compiled.find_first("Dapp").unwrap();
    assert!(artifact.metadata.is_some());
    assert!(artifact.raw_metadata.is_some());
    assert!(artifact.ir.is_some());
    assert!(artifact.ir_optimized.is_some());
    assert!(artifact.opcodes.is_some());
}

#[test]
fn can_compile_dapp_detect_changes_in_libs() {
    let mut project = TempProject::<ConfigurableArtifacts>::dapptools().unwrap();

    let remapping = project.paths().libraries[0].join("remapping");
    project
        .paths_mut()
        .remappings
        .push(Remapping::from_str(&format!("remapping/={}/", remapping.display())).unwrap());

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
    assert!(compiled.find_first("Foo").is_some());
    assert!(compiled.find_first("Bar").is_some());
    assert!(!compiled.has_compiler_errors());

    // nothing to compile
    let compiled = project.compile().unwrap();
    assert!(compiled.find_first("Foo").is_some());
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
    assert!(compiled.find_first("Foo").is_some());
    assert!(compiled.find_first("Bar").is_some());
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
    assert!(compiled.find_first("DssSpellTest").is_some());
    assert!(compiled.find_first("DssSpellTestBase").is_some());

    // nothing to compile
    let compiled = project.compile().unwrap();
    assert!(compiled.is_unchanged());
    assert!(compiled.find_first("DssSpellTest").is_some());
    assert!(compiled.find_first("DssSpellTestBase").is_some());

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
    assert!(compiled.find_first("DssSpellTest").is_some());
    assert!(compiled.find_first("DssSpellTestBase").is_some());
    // ensure change is detected
    assert!(!compiled.is_unchanged());

    // and all recompiled artifacts are different
    for (p, artifact) in compiled.into_artifacts() {
        let other = artifacts.remove(&p).unwrap();
        assert_ne!(artifact, other);
    }
}

#[test]
fn can_compile_dapp_only_recompile_dirty_sources() {
    let project = TempProject::dapptools().unwrap();
    project
        .add_source(
            "A",
            r#"
    pragma solidity ^0.8.10;
    import "./B.sol";
    contract A { }
    "#,
        )
        .unwrap();

    project
        .add_source(
            "B",
            r#"
    pragma solidity ^0.8.10;
    contract B { }
    "#,
        )
        .unwrap();

    let compiled = project.compile().unwrap();
    assert!(!compiled.has_compiler_errors());

    let cache = SolFilesCache::read(project.cache_path()).unwrap();
    // A.sol and B.sol are compatible and should be compiled into one unit
    assert_eq!(cache.compilation_units.len(), 1);
    let path_a = Path::new("src/A.sol");
    let path_b = Path::new("src/B.sol");
    let original_a = cache.entry(path_a).unwrap();
    let original_b = cache.entry(path_b).unwrap();

    // modify B.sol
    project
        .add_source(
            "B",
            r#"
    pragma solidity ^0.8.10;
    contract B { 
        function testExample() public {}
    }
    "#,
        )
        .unwrap();

    let compiled = project.compile().unwrap();
    assert!(!compiled.has_compiler_errors());
    let updated_cache = SolFilesCache::read(project.cache_path()).unwrap();
    assert_eq!(updated_cache.compilation_units.len(), 1);

    let cached_a = updated_cache.entry(path_a).unwrap();
    // A.sol should not be recompiled
    assert_eq!(original_a.last_modification_date, cached_a.last_modification_date);

    let updated_b = updated_cache.entry(path_b).unwrap();
    // Changing source content should not invalidate compilation unit id
    assert_eq!(updated_b.compilation_unit, original_b.compilation_unit);
    // B.sol should be recompiled
    assert_ne!(updated_b.last_modification_date, original_b.last_modification_date);

    project.artifacts_snapshot().unwrap().assert_artifacts_essentials_present();
}

#[test]
fn can_emit_build_info() {
    let mut project = TempProject::dapptools().unwrap();
    project.project_mut().build_info = true;
    project
        .add_source(
            "A",
            r#"
pragma solidity ^0.8.10;
import "./B.sol";
contract A { }
"#,
        )
        .unwrap();

    project
        .add_source(
            "B",
            r#"
pragma solidity ^0.8.10;
contract B { }
"#,
        )
        .unwrap();

    let compiled = project.compile().unwrap();
    assert!(!compiled.has_compiler_errors());

    let info_dir = project.project().build_info_path();
    assert!(info_dir.exists());

    let mut build_info_count = 0;
    for entry in fs::read_dir(info_dir).unwrap() {
        let _info = BuildInfo::read(entry.unwrap().path()).unwrap();
        build_info_count += 1;
    }
    assert_eq!(build_info_count, 1);
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
    assert!(compiled.find_first("Dapp").is_some());
    assert!(!compiled.has_compiler_errors());

    // cache is used when nothing to compile
    let compiled = project.compile().unwrap();
    assert!(compiled.find_first("Dapp").is_some());
    assert!(compiled.is_unchanged());

    // deleted artifacts cause recompile even with cache
    std::fs::remove_dir_all(project.artifacts_path()).unwrap();
    let compiled = project.compile().unwrap();
    assert!(compiled.find_first("Dapp").is_some());
    assert!(!compiled.is_unchanged());

    // new file is compiled even with partial cache
    std::fs::copy(cache_testdata_dir.join("NewContract.sol"), root.join("src/NewContract.sol"))
        .unwrap();
    let compiled = project.compile().unwrap();
    assert!(compiled.find_first("Dapp").is_some());
    assert!(compiled.find_first("NewContract").is_some());
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
    std::fs::remove_file(project.paths.sources.join("Dapp.sol")).unwrap();
    let compiled: ProjectCompileOutput<_> = project.compile().unwrap();
    assert!(compiled.find_first("Dapp").is_none());
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
    assert!(!result.contains("import"));
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
    assert!(!result.contains("import"));
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
    assert!(!result.contains("import"));
    assert!(result.contains("contract DSTest"));
    assert!(result.contains("contract Dapp"));
    assert!(result.contains("contract DappTest"));
}

#[test]
fn can_flatten_unique() {
    let project = TempProject::dapptools().unwrap();

    let f = project
        .add_source(
            "A",
            r#"
pragma solidity ^0.8.10;
import "./C.sol";
import "./B.sol";
contract A { }
"#,
        )
        .unwrap();

    project
        .add_source(
            "B",
            r#"
pragma solidity ^0.8.10;
import "./C.sol";
contract B { }
"#,
        )
        .unwrap();

    project
        .add_source(
            "C",
            r#"
pragma solidity ^0.8.10;
import "./A.sol";
contract C { }
"#,
        )
        .unwrap();

    let result = project.flatten(&f).unwrap();

    assert_eq!(
        result,
        r#"pragma solidity ^0.8.10;

contract C { }

contract B { }

contract A { }
"#
    );
}

#[test]
fn can_flatten_experimental_pragma() {
    let project = TempProject::dapptools().unwrap();

    let f = project
        .add_source(
            "A",
            r#"
pragma solidity ^0.8.10;
pragma experimental ABIEncoderV2;
import "./C.sol";
import "./B.sol";
contract A { }
"#,
        )
        .unwrap();

    project
        .add_source(
            "B",
            r#"
pragma solidity ^0.8.10;
pragma experimental ABIEncoderV2;
import "./C.sol";
contract B { }
"#,
        )
        .unwrap();

    project
        .add_source(
            "C",
            r#"
pragma solidity ^0.8.10;
pragma experimental ABIEncoderV2;
import "./A.sol";
contract C { }
"#,
        )
        .unwrap();

    let result = project.flatten(&f).unwrap();

    assert_eq!(
        result,
        r#"pragma solidity ^0.8.10;
pragma experimental ABIEncoderV2;

contract C { }

contract B { }

contract A { }
"#
    );
}

#[test]
fn can_flatten_file_with_duplicates() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-data/test-flatten-duplicates");
    let paths = ProjectPathsConfig::builder().sources(root.join("contracts"));
    let project = TempProject::<ConfigurableArtifacts>::new(paths).unwrap();

    let target = root.join("contracts/FooBar.sol");

    let result = project.flatten(&target);
    assert!(result.is_ok());

    let result = result.unwrap();
    assert_eq!(
        result,
        r#"//SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.6.0;

contract Bar {}

contract Foo {}

contract FooBar {}
"#
    );
}

#[test]
fn can_flatten_on_solang_failure() {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-data/test-flatten-solang-failure");
    let paths = ProjectPathsConfig::builder().sources(root.join("contracts"));
    let project = TempProject::<ConfigurableArtifacts>::new(paths).unwrap();

    let target = root.join("contracts/Contract.sol");

    let result = project.flatten(&target);
    assert!(result.is_ok());

    let result = result.unwrap();
    assert_eq!(
        result,
        r#"// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.10;

library Lib {}

// Intentionally erroneous code
contract Contract {
    failure();
}
"#
    );
}

#[test]
fn can_flatten_multiline() {
    let project = TempProject::dapptools().unwrap();

    let f = project
        .add_source(
            "A",
            r#"
pragma solidity ^0.8.10;
import "./C.sol";
import {
    IllegalArgument,
    IllegalState
} from "./Errors.sol";
contract A { }
"#,
        )
        .unwrap();

    project
        .add_source(
            "Errors",
            r#"
pragma solidity ^0.8.10;
error IllegalArgument();
error IllegalState();
"#,
        )
        .unwrap();

    project
        .add_source(
            "C",
            r#"
pragma solidity ^0.8.10;
contract C { }
"#,
        )
        .unwrap();

    let result = project.flatten(&f).unwrap();
    assert_eq!(
        result,
        r#"pragma solidity ^0.8.10;

contract C { }

error IllegalArgument();
error IllegalState();

contract A { }
"#
    );
}

#[test]
fn can_flatten_remove_extra_spacing() {
    let project = TempProject::dapptools().unwrap();

    let f = project
        .add_source(
            "A",
            r#"pragma solidity ^0.8.10;
import "./C.sol";
import "./B.sol";
contract A { }
"#,
        )
        .unwrap();

    project
        .add_source(
            "B",
            r#"// This is a B Contract
pragma solidity ^0.8.10;

import "./C.sol";

contract B { }
"#,
        )
        .unwrap();

    project
        .add_source(
            "C",
            r#"pragma solidity ^0.8.10;
contract C { }
"#,
        )
        .unwrap();

    let result = project.flatten(&f).unwrap();
    assert_eq!(
        result,
        r#"pragma solidity ^0.8.10;

contract C { }

// This is a B Contract

contract B { }

contract A { }
"#
    );
}

#[test]
fn can_flatten_with_alias() {
    let project = TempProject::dapptools().unwrap();

    let f = project
        .add_source(
            "Contract",
            r#"pragma solidity ^0.8.10;
import { ParentContract as Parent } from "./Parent.sol";
import { AnotherParentContract as AnotherParent } from "./AnotherParent.sol";
import { PeerContract as Peer } from "./Peer.sol";
import { MathLibrary as Math } from "./Math.sol";
import * as Lib from "./SomeLib.sol";

contract Contract is Parent,
    AnotherParent {
    using Math for uint256;

    string public usingString = "using Math for uint256;";
    string public inheritanceString = "\"Contract is Parent {\"";
    string public castString = 'Peer(smth) ';
    string public methodString = '\' Math.max()';

    Peer public peer;

    error Peer();

    constructor(address _peer) {
        peer = Peer(_peer);
    }

    function Math(uint256 value) external pure returns (uint256) {
        return Math.minusOne(Math.max() - value.diffMax());
    }
}
"#,
        )
        .unwrap();

    project
        .add_source(
            "Parent",
            r#"pragma solidity ^0.8.10;
contract ParentContract { }
"#,
        )
        .unwrap();

    project
        .add_source(
            "AnotherParent",
            r#"pragma solidity ^0.8.10;
contract AnotherParentContract { }
"#,
        )
        .unwrap();

    project
        .add_source(
            "Peer",
            r#"pragma solidity ^0.8.10;
contract PeerContract { }
"#,
        )
        .unwrap();

    project
        .add_source(
            "Math",
            r#"pragma solidity ^0.8.10;
library MathLibrary {
    function minusOne(uint256 val) internal returns (uint256) {
        return val - 1;
    }

    function max() internal returns (uint256) {
        return type(uint256).max;
    }

    function diffMax(uint256 value) internal returns (uint256) {
        return type(uint256).max - value;
    }
}
"#,
        )
        .unwrap();

    project
        .add_source(
            "SomeLib",
            r#"pragma solidity ^0.8.10;
library SomeLib { }
"#,
        )
        .unwrap();

    let result = project.flatten(&f).unwrap();
    assert_eq!(
        result,
        r#"pragma solidity ^0.8.10;

contract ParentContract { }

contract AnotherParentContract { }

contract PeerContract { }

library MathLibrary {
    function minusOne(uint256 val) internal returns (uint256) {
        return val - 1;
    }

    function max() internal returns (uint256) {
        return type(uint256).max;
    }

    function diffMax(uint256 value) internal returns (uint256) {
        return type(uint256).max - value;
    }
}

library SomeLib { }

contract Contract is ParentContract,
    AnotherParentContract {
    using MathLibrary for uint256;

    string public usingString = "using Math for uint256;";
    string public inheritanceString = "\"Contract is Parent {\"";
    string public castString = 'Peer(smth) ';
    string public methodString = '\' Math.max()';

    PeerContract public peer;

    error Peer();

    constructor(address _peer) {
        peer = PeerContract(_peer);
    }

    function Math(uint256 value) external pure returns (uint256) {
        return MathLibrary.minusOne(MathLibrary.max() - value.diffMax());
    }
}
"#
    );
}

#[test]
fn can_flatten_with_version_pragma_after_imports() {
    let project = TempProject::dapptools().unwrap();

    let f = project
        .add_source(
            "A",
            r#"
pragma solidity ^0.8.10;

import * as B from "./B.sol";

contract A { }
"#,
        )
        .unwrap();

    project
        .add_source(
            "B",
            r#"
import D from "./D.sol";
pragma solidity ^0.8.10;
import * as C from "./C.sol";
contract B { }
"#,
        )
        .unwrap();

    project
        .add_source(
            "C",
            r#"
pragma solidity ^0.8.10;
contract C { }
"#,
        )
        .unwrap();

    project
        .add_source(
            "D",
            r#"
pragma solidity ^0.8.10;
contract D { }
"#,
        )
        .unwrap();

    let result = project.flatten(&f).unwrap();
    assert_eq!(
        result,
        r#"pragma solidity ^0.8.10;

contract D { }

contract C { }

contract B { }

contract A { }
"#
    );
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

    let f = tmp
        .add_contract(
            "examples/Foo",
            r#"
    pragma solidity ^0.8.10;

    contract Foo {}
   "#,
        )
        .unwrap();

    let compiled = tmp.project().compile_file(f.clone()).unwrap();
    assert!(!compiled.has_compiler_errors());
    assert!(compiled.find_first("Foo").is_some());

    let bar = tmp
        .add_contract(
            "examples/Bar",
            r#"
    pragma solidity ^0.8.10;

    contract Bar {}
   "#,
        )
        .unwrap();

    let compiled = tmp.project().compile_files(vec![f, bar]).unwrap();
    assert!(!compiled.has_compiler_errors());
    assert!(compiled.find_first("Foo").is_some());
    assert!(compiled.find_first("Bar").is_some());
}

#[test]
fn consistent_bytecode() {
    let tmp = TempProject::dapptools().unwrap();

    tmp.add_source(
        "LinkTest",
        r#"
// SPDX-License-Identifier: MIT
library LibTest {
    function foobar(uint256 a) public view returns (uint256) {
    	return a * 100;
    }
}
contract LinkTest {
    function foo() public returns (uint256) {
        return LibTest.foobar(1);
    }
}
"#,
    )
    .unwrap();

    let compiled = tmp.compile().unwrap();
    assert!(!compiled.has_compiler_errors());

    let contract = compiled.find_first("LinkTest").unwrap();
    let bytecode = &contract.bytecode.as_ref().unwrap().object;
    assert!(bytecode.is_unlinked());
    let s = bytecode.as_str().unwrap();
    assert!(!s.starts_with("0x"));

    let s = serde_json::to_string(&bytecode).unwrap();
    assert_eq!(bytecode.clone(), serde_json::from_str(&s).unwrap());
}

#[test]
fn can_apply_libraries() {
    let mut tmp = TempProject::dapptools().unwrap();

    tmp.add_source(
        "LinkTest",
        r#"
// SPDX-License-Identifier: MIT
import "./MyLib.sol";
contract LinkTest {
    function foo() public returns (uint256) {
        return MyLib.foobar(1);
    }
}
"#,
    )
    .unwrap();

    let lib = tmp
        .add_source(
            "MyLib",
            r#"
// SPDX-License-Identifier: MIT
library MyLib {
    function foobar(uint256 a) public view returns (uint256) {
    	return a * 100;
    }
}
"#,
        )
        .unwrap();

    let compiled = tmp.compile().unwrap();
    assert!(!compiled.has_compiler_errors());

    assert!(compiled.find_first("MyLib").is_some());
    let contract = compiled.find_first("LinkTest").unwrap();
    let bytecode = &contract.bytecode.as_ref().unwrap().object;
    assert!(bytecode.is_unlinked());

    // provide the library settings to let solc link
    tmp.project_mut().solc_config.settings.libraries = BTreeMap::from([(
        lib,
        BTreeMap::from([("MyLib".to_string(), format!("{:?}", Address::zero()))]),
    )])
    .into();

    let compiled = tmp.compile().unwrap();
    assert!(!compiled.has_compiler_errors());

    assert!(compiled.find_first("MyLib").is_some());
    let contract = compiled.find_first("LinkTest").unwrap();
    let bytecode = &contract.bytecode.as_ref().unwrap().object;
    assert!(!bytecode.is_unlinked());

    let libs = Libraries::parse(&[format!("./src/MyLib.sol:MyLib:{:?}", Address::zero())]).unwrap();
    // provide the library settings to let solc link
    tmp.project_mut().solc_config.settings.libraries = libs.with_applied_remappings(tmp.paths());

    let compiled = tmp.compile().unwrap();
    assert!(!compiled.has_compiler_errors());

    assert!(compiled.find_first("MyLib").is_some());
    let contract = compiled.find_first("LinkTest").unwrap();
    let bytecode = &contract.bytecode.as_ref().unwrap().object;
    assert!(!bytecode.is_unlinked());
}

#[test]
fn can_apply_libraries_with_remappings() {
    let mut tmp = TempProject::dapptools().unwrap();

    let remapping = tmp.paths().libraries[0].join("remapping");
    tmp.paths_mut()
        .remappings
        .push(Remapping::from_str(&format!("remapping/={}/", remapping.display())).unwrap());

    tmp.add_source(
        "LinkTest",
        r#"
// SPDX-License-Identifier: MIT
import "remapping/MyLib.sol";
contract LinkTest {
    function foo() public returns (uint256) {
        return MyLib.foobar(1);
    }
}
"#,
    )
    .unwrap();

    tmp.add_lib(
        "remapping/MyLib",
        r#"
// SPDX-License-Identifier: MIT
library MyLib {
    function foobar(uint256 a) public view returns (uint256) {
    	return a * 100;
    }
}
"#,
    )
    .unwrap();

    let compiled = tmp.compile().unwrap();
    assert!(!compiled.has_compiler_errors());

    assert!(compiled.find_first("MyLib").is_some());
    let contract = compiled.find_first("LinkTest").unwrap();
    let bytecode = &contract.bytecode.as_ref().unwrap().object;
    assert!(bytecode.is_unlinked());

    let libs =
        Libraries::parse(&[format!("remapping/MyLib.sol:MyLib:{:?}", Address::zero())]).unwrap(); // provide the library settings to let solc link
    tmp.project_mut().solc_config.settings.libraries = libs.with_applied_remappings(tmp.paths());

    let compiled = tmp.compile().unwrap();
    assert!(!compiled.has_compiler_errors());

    assert!(compiled.find_first("MyLib").is_some());
    let contract = compiled.find_first("LinkTest").unwrap();
    let bytecode = &contract.bytecode.as_ref().unwrap().object;
    assert!(!bytecode.is_unlinked());
}
#[test]
fn can_recompile_with_changes() {
    let mut tmp = TempProject::dapptools().unwrap();
    tmp.project_mut().allowed_paths = vec![tmp.root().join("modules")].into();

    let content = r#"
    pragma solidity ^0.8.10;
    import "../modules/B.sol";
    contract A {}
   "#;
    tmp.add_source("A", content).unwrap();

    tmp.add_contract(
        "modules/B",
        r#"
    pragma solidity ^0.8.10;
    contract B {}
   "#,
    )
    .unwrap();

    let compiled = tmp.compile().unwrap();
    assert!(!compiled.has_compiler_errors());
    assert!(compiled.find_first("A").is_some());
    assert!(compiled.find_first("B").is_some());

    let compiled = tmp.compile().unwrap();
    assert!(compiled.find_first("A").is_some());
    assert!(compiled.find_first("B").is_some());
    assert!(compiled.is_unchanged());

    // modify A.sol
    tmp.add_source("A", format!("{content}\n")).unwrap();
    let compiled = tmp.compile().unwrap();
    assert!(!compiled.has_compiler_errors());
    assert!(!compiled.is_unchanged());
    assert!(compiled.find_first("A").is_some());
    assert!(compiled.find_first("B").is_some());
}

#[test]
fn can_recompile_with_lowercase_names() {
    let tmp = TempProject::dapptools().unwrap();

    tmp.add_source(
        "deployProxy.sol",
        r#"
    pragma solidity =0.8.12;
    contract DeployProxy {}
   "#,
    )
    .unwrap();

    let upgrade = r#"
    pragma solidity =0.8.12;
    import "./deployProxy.sol";
    import "./ProxyAdmin.sol";
    contract UpgradeProxy {}
   "#;
    tmp.add_source("upgradeProxy.sol", upgrade).unwrap();

    tmp.add_source(
        "ProxyAdmin.sol",
        r#"
    pragma solidity =0.8.12;
    contract ProxyAdmin {}
   "#,
    )
    .unwrap();

    let compiled = tmp.compile().unwrap();
    assert!(!compiled.has_compiler_errors());
    assert!(compiled.find_first("DeployProxy").is_some());
    assert!(compiled.find_first("UpgradeProxy").is_some());
    assert!(compiled.find_first("ProxyAdmin").is_some());

    let artifacts = tmp.artifacts_snapshot().unwrap();
    assert_eq!(artifacts.artifacts.as_ref().len(), 3);
    artifacts.assert_artifacts_essentials_present();

    let compiled = tmp.compile().unwrap();
    assert!(compiled.find_first("DeployProxy").is_some());
    assert!(compiled.find_first("UpgradeProxy").is_some());
    assert!(compiled.find_first("ProxyAdmin").is_some());
    assert!(compiled.is_unchanged());

    // modify upgradeProxy.sol
    tmp.add_source("upgradeProxy.sol", format!("{upgrade}\n")).unwrap();
    let compiled = tmp.compile().unwrap();
    assert!(!compiled.has_compiler_errors());
    assert!(!compiled.is_unchanged());
    assert!(compiled.find_first("DeployProxy").is_some());
    assert!(compiled.find_first("UpgradeProxy").is_some());
    assert!(compiled.find_first("ProxyAdmin").is_some());

    let artifacts = tmp.artifacts_snapshot().unwrap();
    assert_eq!(artifacts.artifacts.as_ref().len(), 3);
    artifacts.assert_artifacts_essentials_present();
}

#[test]
fn can_recompile_unchanged_with_empty_files() {
    let tmp = TempProject::dapptools().unwrap();

    tmp.add_source(
        "A",
        r#"
    pragma solidity ^0.8.10;
    import "./B.sol";
    contract A {}
   "#,
    )
    .unwrap();

    tmp.add_source(
        "B",
        r#"
    pragma solidity ^0.8.10;
    import "./C.sol";
   "#,
    )
    .unwrap();

    let c = r#"
    pragma solidity ^0.8.10;
    contract C {}
   "#;
    tmp.add_source("C", c).unwrap();

    let compiled = tmp.compile().unwrap();
    assert!(!compiled.has_compiler_errors());
    assert!(compiled.find_first("A").is_some());
    assert!(compiled.find_first("C").is_some());

    let compiled = tmp.compile().unwrap();
    assert!(compiled.find_first("A").is_some());
    assert!(compiled.find_first("C").is_some());
    assert!(compiled.is_unchanged());

    // modify C.sol
    tmp.add_source("C", format!("{c}\n")).unwrap();
    let compiled = tmp.compile().unwrap();
    assert!(!compiled.has_compiler_errors());
    assert!(!compiled.is_unchanged());
    assert!(compiled.find_first("A").is_some());
    assert!(compiled.find_first("C").is_some());
}

#[test]
fn can_emit_empty_artifacts() {
    let tmp = TempProject::dapptools().unwrap();

    let top_level = tmp
        .add_source(
            "top_level",
            r#"
    function test() {}
   "#,
        )
        .unwrap();

    tmp.add_source(
        "Contract",
        r#"
// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.10;

import "./top_level.sol";

contract Contract {
    function a() public{
        test();
    }
}
   "#,
    )
    .unwrap();

    let compiled = tmp.compile().unwrap();
    assert!(!compiled.has_compiler_errors());
    assert!(compiled.find_first("Contract").is_some());
    assert!(compiled.find_first("top_level").is_some());
    let mut artifacts = tmp.artifacts_snapshot().unwrap();

    assert_eq!(artifacts.artifacts.as_ref().len(), 2);

    let mut top_level =
        artifacts.artifacts.as_mut().remove(top_level.to_string_lossy().as_ref()).unwrap();

    assert_eq!(top_level.len(), 1);

    let artifact = top_level.remove("top_level").unwrap().remove(0);
    assert!(artifact.artifact.ast.is_some());

    // recompile
    let compiled = tmp.compile().unwrap();
    assert!(compiled.is_unchanged());

    // modify standalone file

    tmp.add_source(
        "top_level",
        r#"
    error MyError();
    function test() {}
   "#,
    )
    .unwrap();
    let compiled = tmp.compile().unwrap();
    assert!(!compiled.is_unchanged());
}

#[test]
fn can_detect_contract_def_source_files() {
    let tmp = TempProject::dapptools().unwrap();

    let mylib = tmp
        .add_source(
            "MyLib",
            r#"
        pragma solidity 0.8.10;
        library MyLib {
        }
   "#,
        )
        .unwrap();

    let myinterface = tmp
        .add_source(
            "MyInterface",
            r#"
        pragma solidity 0.8.10;
        interface MyInterface {}
   "#,
        )
        .unwrap();

    let mycontract = tmp
        .add_source(
            "MyContract",
            r#"
        pragma solidity 0.8.10;
        contract MyContract {}
   "#,
        )
        .unwrap();

    let myabstract_contract = tmp
        .add_source(
            "MyAbstractContract",
            r#"
        pragma solidity 0.8.10;
        contract MyAbstractContract {}
   "#,
        )
        .unwrap();

    let myerr = tmp
        .add_source(
            "MyError",
            r#"
        pragma solidity 0.8.10;
       error MyError();
   "#,
        )
        .unwrap();

    let myfunc = tmp
        .add_source(
            "MyFunction",
            r#"
        pragma solidity 0.8.10;
        function abc(){}
   "#,
        )
        .unwrap();

    let compiled = tmp.compile().unwrap();
    assert!(!compiled.has_compiler_errors());

    let mut sources = compiled.output().sources;
    let myfunc = sources.remove_by_path(myfunc.to_string_lossy()).unwrap();
    assert!(!myfunc.contains_contract_definition());

    let myerr = sources.remove_by_path(myerr.to_string_lossy()).unwrap();
    assert!(!myerr.contains_contract_definition());

    let mylib = sources.remove_by_path(mylib.to_string_lossy()).unwrap();
    assert!(mylib.contains_contract_definition());

    let myabstract_contract =
        sources.remove_by_path(myabstract_contract.to_string_lossy()).unwrap();
    assert!(myabstract_contract.contains_contract_definition());

    let myinterface = sources.remove_by_path(myinterface.to_string_lossy()).unwrap();
    assert!(myinterface.contains_contract_definition());

    let mycontract = sources.remove_by_path(mycontract.to_string_lossy()).unwrap();
    assert!(mycontract.contains_contract_definition());
}

#[test]
fn can_compile_sparse_with_link_references() {
    let tmp = TempProject::dapptools().unwrap();

    tmp.add_source(
        "ATest.t.sol",
        r#"
    pragma solidity =0.8.12;
    import {MyLib} from "./mylib.sol";
    contract ATest {
      function test_mylib() public returns (uint256) {
         return MyLib.doStuff();
      }
    }
   "#,
    )
    .unwrap();

    let my_lib_path = tmp
        .add_source(
            "mylib.sol",
            r#"
    pragma solidity =0.8.12;
    library MyLib {
       function doStuff() external pure returns (uint256) {return 1337;}
    }
   "#,
        )
        .unwrap();

    let mut compiled = tmp.compile_sparse(TestFileFilter::default()).unwrap();
    assert!(!compiled.has_compiler_errors());

    let mut output = compiled.clone().output();

    assert!(compiled.find_first("ATest").is_some());
    assert!(compiled.find_first("MyLib").is_some());
    let lib = compiled.remove_first("MyLib").unwrap();
    assert!(lib.bytecode.is_some());
    let lib = compiled.remove_first("MyLib");
    assert!(lib.is_none());

    let mut dup = output.clone();
    let lib = dup.remove_first("MyLib");
    assert!(lib.is_some());
    let lib = dup.remove_first("MyLib");
    assert!(lib.is_none());

    dup = output.clone();
    let lib = dup.remove(my_lib_path.to_string_lossy(), "MyLib");
    assert!(lib.is_some());
    let lib = dup.remove(my_lib_path.to_string_lossy(), "MyLib");
    assert!(lib.is_none());

    let info = ContractInfo::new(format!("{}:{}", my_lib_path.to_string_lossy(), "MyLib"));
    let lib = output.remove_contract(&info);
    assert!(lib.is_some());
    let lib = output.remove_contract(&info);
    assert!(lib.is_none());
}

#[test]
fn can_sanitize_bytecode_hash() {
    let mut tmp = TempProject::dapptools().unwrap();
    tmp.project_mut().solc_config.settings.metadata = Some(BytecodeHash::Ipfs.into());

    tmp.add_source(
        "A",
        r#"
    pragma solidity =0.5.17;
    contract A {}
   "#,
    )
    .unwrap();

    let compiled = tmp.compile().unwrap();
    assert!(!compiled.has_compiler_errors());
    assert!(compiled.find_first("A").is_some());
}

#[test]
fn can_compile_std_json_input() {
    let tmp = TempProject::dapptools_init().unwrap();
    tmp.assert_no_errors();
    let source = tmp.list_source_files().into_iter().find(|p| p.ends_with("Dapp.t.sol")).unwrap();
    let input = tmp.project().standard_json_input(source).unwrap();

    assert!(input.settings.remappings.contains(&"ds-test/=lib/ds-test/src/".parse().unwrap()));
    let input: CompilerInput = input.into();
    assert!(input.sources.contains_key(Path::new("lib/ds-test/src/test.sol")));

    // should be installed
    if let Some(solc) = Solc::find_svm_installed_version("0.8.10").ok().flatten() {
        let out = solc.compile(&input).unwrap();
        assert!(!out.has_error());
        assert!(out.sources.contains_key("lib/ds-test/src/test.sol"));
    }
}

#[test]
fn can_compile_model_checker_sample() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-data/model-checker-sample");
    let paths = ProjectPathsConfig::builder().sources(root);

    let mut project = TempProject::<ConfigurableArtifacts>::new(paths).unwrap();
    project.project_mut().solc_config.settings.model_checker = Some(ModelCheckerSettings {
        contracts: BTreeMap::new(),
        engine: Some(CHC),
        targets: None,
        timeout: Some(10000),
    });
    let compiled = project.compile().unwrap();

    assert!(compiled.find_first("Assert").is_some());
    assert!(!compiled.has_compiler_errors());
    assert!(compiled.has_compiler_warnings());
}

#[test]
fn test_compiler_severity_filter() {
    fn gen_test_data_warning_path() -> ProjectPathsConfig {
        let root =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-data/test-contract-warnings");

        ProjectPathsConfig::builder().sources(root).build().unwrap()
    }

    let project = Project::builder()
        .no_artifacts()
        .paths(gen_test_data_warning_path())
        .ephemeral()
        .build()
        .unwrap();
    let compiled = project.compile().unwrap();
    assert!(compiled.has_compiler_warnings());
    assert!(!compiled.has_compiler_errors());

    let project = Project::builder()
        .no_artifacts()
        .paths(gen_test_data_warning_path())
        .ephemeral()
        .set_compiler_severity_filter(ethers_solc::artifacts::Severity::Warning)
        .build()
        .unwrap();
    let compiled = project.compile().unwrap();
    assert!(compiled.has_compiler_warnings());
    assert!(compiled.has_compiler_errors());
}

#[test]
fn test_compiler_severity_filter_and_ignored_error_codes() {
    fn gen_test_data_licensing_warning() -> ProjectPathsConfig {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("test-data/test-contract-warnings/LicenseWarning.sol");

        ProjectPathsConfig::builder().sources(root).build().unwrap()
    }

    let missing_license_error_code = 1878;

    let project = Project::builder()
        .no_artifacts()
        .paths(gen_test_data_licensing_warning())
        .ephemeral()
        .build()
        .unwrap();
    let compiled = project.compile().unwrap();
    assert!(compiled.has_compiler_warnings());

    let project = Project::builder()
        .no_artifacts()
        .paths(gen_test_data_licensing_warning())
        .ephemeral()
        .ignore_error_code(missing_license_error_code)
        .build()
        .unwrap();
    let compiled = project.compile().unwrap();
    assert!(!compiled.has_compiler_warnings());
    assert!(!compiled.has_compiler_errors());

    let project = Project::builder()
        .no_artifacts()
        .paths(gen_test_data_licensing_warning())
        .ephemeral()
        .ignore_error_code(missing_license_error_code)
        .set_compiler_severity_filter(ethers_solc::artifacts::Severity::Warning)
        .build()
        .unwrap();
    let compiled = project.compile().unwrap();
    assert!(!compiled.has_compiler_warnings());
    assert!(!compiled.has_compiler_errors());
}

fn remove_solc_if_exists(version: &Version) {
    if Solc::find_svm_installed_version(version.to_string()).unwrap().is_some() {
        svm::remove_version(version).expect("failed to remove version")
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn can_install_solc_and_compile_version() {
    let project = TempProject::dapptools().unwrap();
    let version = Version::new(0, 8, 10);

    project
        .add_source(
            "Contract",
            format!(
                r#"
pragma solidity {version};
contract Contract {{ }}
"#
            ),
        )
        .unwrap();

    remove_solc_if_exists(&version);

    let compiled = project.compile().unwrap();
    assert!(!compiled.has_compiler_errors());
}

#[tokio::test(flavor = "multi_thread")]
async fn can_install_solc_and_compile_std_json_input_async() {
    let tmp = TempProject::dapptools_init().unwrap();
    tmp.assert_no_errors();
    let source = tmp.list_source_files().into_iter().find(|p| p.ends_with("Dapp.t.sol")).unwrap();
    let input = tmp.project().standard_json_input(source).unwrap();
    let solc = &tmp.project().solc;

    assert!(input.settings.remappings.contains(&"ds-test/=lib/ds-test/src/".parse().unwrap()));
    let input: CompilerInput = input.into();
    assert!(input.sources.contains_key(Path::new("lib/ds-test/src/test.sol")));

    remove_solc_if_exists(&solc.version().expect("failed to get version"));

    let out = solc.async_compile(&input).await.unwrap();
    assert!(!out.has_error());
    assert!(out.sources.contains_key("lib/ds-test/src/test.sol"));
}

#[test]
fn can_purge_obsolete_artifacts() {
    let mut project = TempProject::<ConfigurableArtifacts>::dapptools().unwrap();
    project.set_solc("0.8.10");
    project
        .add_source(
            "Contract",
            r#"
    pragma solidity >=0.8.10;

   contract Contract {
        function xyz() public {
        }
   }
   "#,
        )
        .unwrap();

    let compiled = project.compile().unwrap();
    assert!(!compiled.has_compiler_errors());
    assert!(!compiled.is_unchanged());
    assert_eq!(compiled.into_artifacts().count(), 1);

    project.set_solc("0.8.13");

    let compiled = project.compile().unwrap();
    assert!(!compiled.has_compiler_errors());
    assert!(!compiled.is_unchanged());
    assert_eq!(compiled.into_artifacts().count(), 1);
}

#[test]
fn can_parse_notice() {
    let mut project = TempProject::<ConfigurableArtifacts>::dapptools().unwrap();
    project.project_mut().artifacts.additional_values.userdoc = true;
    project.project_mut().solc_config.settings = project.project_mut().artifacts.settings();

    let contract = r#"
    pragma solidity $VERSION;

   contract Contract {
      string greeting;

        /**
         * @notice hello
         */    
         constructor(string memory _greeting) public {
            greeting = _greeting;
        }
        
        /**
         * @notice hello
         */
        function xyz() public {
        }
        
        /// @notice hello
        function abc() public {
        }
   }
   "#;
    project.add_source("Contract", contract.replace("$VERSION", "=0.5.17")).unwrap();

    let mut compiled = project.compile().unwrap();
    assert!(!compiled.has_compiler_errors());
    assert!(!compiled.is_unchanged());
    assert!(compiled.find_first("Contract").is_some());
    let userdoc = compiled.remove_first("Contract").unwrap().userdoc;

    assert_eq!(
        userdoc,
        Some(UserDoc {
            version: None,
            kind: None,
            methods: BTreeMap::from([
                ("abc()".to_string(), UserDocNotice::Notice { notice: "hello".to_string() }),
                ("xyz()".to_string(), UserDocNotice::Notice { notice: "hello".to_string() }),
                ("constructor".to_string(), UserDocNotice::Constructor("hello".to_string())),
            ]),
            events: BTreeMap::new(),
            errors: BTreeMap::new(),
            notice: None
        })
    );

    project.add_source("Contract", contract.replace("$VERSION", "^0.8.10")).unwrap();

    let mut compiled = project.compile().unwrap();
    assert!(!compiled.has_compiler_errors());
    assert!(!compiled.is_unchanged());
    assert!(compiled.find_first("Contract").is_some());
    let userdoc = compiled.remove_first("Contract").unwrap().userdoc;

    assert_eq!(
        userdoc,
        Some(UserDoc {
            version: Some(1),
            kind: Some("user".to_string()),
            methods: BTreeMap::from([
                ("abc()".to_string(), UserDocNotice::Notice { notice: "hello".to_string() }),
                ("xyz()".to_string(), UserDocNotice::Notice { notice: "hello".to_string() }),
                ("constructor".to_string(), UserDocNotice::Notice { notice: "hello".to_string() }),
            ]),
            events: BTreeMap::new(),
            errors: BTreeMap::new(),
            notice: None
        })
    );
}

#[test]
fn can_parse_doc() {
    let mut project = TempProject::<ConfigurableArtifacts>::dapptools().unwrap();
    project.project_mut().artifacts.additional_values.userdoc = true;
    project.project_mut().artifacts.additional_values.devdoc = true;
    project.project_mut().solc_config.settings = project.project_mut().artifacts.settings();

    let contract = r#"
// SPDX-License-Identifier: GPL-3.0-only
pragma solidity ^0.8.0;

/// @title Not an ERC20.
/// @author Notadev
/// @notice Do not use this.
/// @dev This is not an ERC20 implementation.
/// @custom:experimental This is an experimental contract.
interface INotERC20 {
    /// @notice Transfer tokens.
    /// @dev Transfer `amount` tokens to account `to`.
    /// @param to Target account.
    /// @param amount Transfer amount.
    /// @return A boolean value indicating whether the operation succeeded.
    function transfer(address to, uint256 amount) external returns (bool);

    /// @notice Transfer some tokens.
    /// @dev Emitted when transfer.
    /// @param from Source account.
    /// @param to Target account.
    /// @param value Transfer amount.
    event Transfer(address indexed from, address indexed to, uint256 value);

    /// @notice Insufficient balance for transfer.
    /// @dev Needed `required` but only `available` available.
    /// @param available Balance available.
    /// @param required Requested amount to transfer.
    error InsufficientBalance(uint256 available, uint256 required);
}

contract NotERC20 is INotERC20 {
    /// @inheritdoc INotERC20
    function transfer(address to, uint256 amount) external returns (bool) {
        return false;
    }
}
    "#;
    project.add_source("Contract", contract).unwrap();

    let mut compiled = project.compile().unwrap();
    assert!(!compiled.has_compiler_errors());
    assert!(!compiled.is_unchanged());

    assert!(compiled.find_first("INotERC20").is_some());
    let contract = compiled.remove_first("INotERC20").unwrap();
    assert_eq!(
        contract.userdoc,
        Some(UserDoc {
            version: Some(1),
            kind: Some("user".to_string()),
            notice: Some("Do not use this.".to_string()),
            methods: BTreeMap::from([(
                "transfer(address,uint256)".to_string(),
                UserDocNotice::Notice { notice: "Transfer tokens.".to_string() }
            ),]),
            events: BTreeMap::from([(
                "Transfer(address,address,uint256)".to_string(),
                UserDocNotice::Notice { notice: "Transfer some tokens.".to_string() }
            ),]),
            errors: BTreeMap::from([(
                "InsufficientBalance(uint256,uint256)".to_string(),
                vec![UserDocNotice::Notice {
                    notice: "Insufficient balance for transfer.".to_string()
                }]
            ),]),
        })
    );
    assert_eq!(
        contract.devdoc,
        Some(DevDoc {
            version: Some(1),
            kind: Some("dev".to_string()),
            author: Some("Notadev".to_string()),
            details: Some("This is not an ERC20 implementation.".to_string()),
            custom_experimental: Some("This is an experimental contract.".to_string()),
            methods: BTreeMap::from([(
                "transfer(address,uint256)".to_string(),
                MethodDoc {
                    details: Some("Transfer `amount` tokens to account `to`.".to_string()),
                    params: BTreeMap::from([
                        ("to".to_string(), "Target account.".to_string()),
                        ("amount".to_string(), "Transfer amount.".to_string())
                    ]),
                    returns: BTreeMap::from([(
                        "_0".to_string(),
                        "A boolean value indicating whether the operation succeeded.".to_string()
                    ),])
                }
            ),]),
            events: BTreeMap::from([(
                "Transfer(address,address,uint256)".to_string(),
                EventDoc {
                    details: Some("Emitted when transfer.".to_string()),
                    params: BTreeMap::from([
                        ("from".to_string(), "Source account.".to_string()),
                        ("to".to_string(), "Target account.".to_string()),
                        ("value".to_string(), "Transfer amount.".to_string()),
                    ]),
                }
            ),]),
            errors: BTreeMap::from([(
                "InsufficientBalance(uint256,uint256)".to_string(),
                vec![ErrorDoc {
                    details: Some("Needed `required` but only `available` available.".to_string()),
                    params: BTreeMap::from([
                        ("available".to_string(), "Balance available.".to_string()),
                        ("required".to_string(), "Requested amount to transfer.".to_string())
                    ]),
                }]
            ),]),
            title: Some("Not an ERC20.".to_string())
        })
    );

    assert!(compiled.find_first("NotERC20").is_some());
    let contract = compiled.remove_first("NotERC20").unwrap();
    assert_eq!(
        contract.userdoc,
        Some(UserDoc {
            version: Some(1),
            kind: Some("user".to_string()),
            notice: None,
            methods: BTreeMap::from([(
                "transfer(address,uint256)".to_string(),
                UserDocNotice::Notice { notice: "Transfer tokens.".to_string() }
            ),]),
            events: BTreeMap::from([(
                "Transfer(address,address,uint256)".to_string(),
                UserDocNotice::Notice { notice: "Transfer some tokens.".to_string() }
            ),]),
            errors: BTreeMap::from([(
                "InsufficientBalance(uint256,uint256)".to_string(),
                vec![UserDocNotice::Notice {
                    notice: "Insufficient balance for transfer.".to_string()
                }]
            ),]),
        })
    );
    assert_eq!(
        contract.devdoc,
        Some(DevDoc {
            version: Some(1),
            kind: Some("dev".to_string()),
            author: None,
            details: None,
            custom_experimental: None,
            methods: BTreeMap::from([(
                "transfer(address,uint256)".to_string(),
                MethodDoc {
                    details: Some("Transfer `amount` tokens to account `to`.".to_string()),
                    params: BTreeMap::from([
                        ("to".to_string(), "Target account.".to_string()),
                        ("amount".to_string(), "Transfer amount.".to_string())
                    ]),
                    returns: BTreeMap::from([(
                        "_0".to_string(),
                        "A boolean value indicating whether the operation succeeded.".to_string()
                    ),])
                }
            ),]),
            events: BTreeMap::new(),
            errors: BTreeMap::from([(
                "InsufficientBalance(uint256,uint256)".to_string(),
                vec![ErrorDoc {
                    details: Some("Needed `required` but only `available` available.".to_string()),
                    params: BTreeMap::from([
                        ("available".to_string(), "Balance available.".to_string()),
                        ("required".to_string(), "Requested amount to transfer.".to_string())
                    ]),
                }]
            ),]),
            title: None
        })
    );
}

#[test]
fn test_relative_cache_entries() {
    let project = TempProject::dapptools().unwrap();
    let _a = project
        .add_source(
            "A",
            r#"
pragma solidity ^0.8.10;
contract A { }
"#,
        )
        .unwrap();
    let _b = project
        .add_source(
            "B",
            r#"
pragma solidity ^0.8.10;
contract B { }
"#,
        )
        .unwrap();
    let _c = project
        .add_source(
            "C",
            r#"
pragma solidity ^0.8.10;
contract C { }
"#,
        )
        .unwrap();
    let _d = project
        .add_source(
            "D",
            r#"
pragma solidity ^0.8.10;
contract D { }
"#,
        )
        .unwrap();

    let compiled = project.compile().unwrap();
    assert!(!compiled.has_compiler_errors());

    let cache = SolFilesCache::read(project.cache_path()).unwrap();

    let entries = vec![
        PathBuf::from("src/A.sol"),
        PathBuf::from("src/B.sol"),
        PathBuf::from("src/C.sol"),
        PathBuf::from("src/D.sol"),
    ];
    assert_eq!(entries, cache.files.keys().cloned().collect::<Vec<_>>());

    let cache = SolFilesCache::read_joined(project.paths()).unwrap();

    assert_eq!(
        entries.into_iter().map(|p| project.root().join(p)).collect::<Vec<_>>(),
        cache.files.keys().cloned().collect::<Vec<_>>()
    );
}

#[test]
fn test_failure_after_removing_file() {
    let project = TempProject::dapptools().unwrap();
    project
        .add_source(
            "A",
            r#"
pragma solidity ^0.8.10;
import "./B.sol";
contract A { }
"#,
        )
        .unwrap();

    project
        .add_source(
            "B",
            r#"
pragma solidity ^0.8.10;
import "./C.sol";
contract B { }
"#,
        )
        .unwrap();

    let c = project
        .add_source(
            "C",
            r#"
pragma solidity ^0.8.10;
contract C { }
"#,
        )
        .unwrap();

    let compiled = project.compile().unwrap();
    assert!(!compiled.has_compiler_errors());

    fs::remove_file(c).unwrap();
    let compiled = project.compile().unwrap();
    assert!(compiled.has_compiler_errors());
}

#[test]
fn can_handle_conflicting_files() {
    let project = TempProject::<ConfigurableArtifacts>::dapptools().unwrap();

    project
        .add_source(
            "Greeter",
            r#"
    pragma solidity ^0.8.10;

    contract Greeter {}
   "#,
        )
        .unwrap();

    project
        .add_source(
            "tokens/Greeter",
            r#"
    pragma solidity ^0.8.10;

    contract Greeter {}
   "#,
        )
        .unwrap();

    let compiled = project.compile().unwrap();
    assert!(!compiled.has_compiler_errors());

    let artifacts = compiled.artifacts().count();
    assert_eq!(artifacts, 2);

    // nothing to compile
    let compiled = project.compile().unwrap();
    assert!(compiled.is_unchanged());
    let artifacts = compiled.artifacts().count();
    assert_eq!(artifacts, 2);

    let cache = SolFilesCache::read(project.cache_path()).unwrap();

    let mut source_files = cache.files.keys().cloned().collect::<Vec<_>>();
    source_files.sort_unstable();

    assert_eq!(
        source_files,
        vec![PathBuf::from("src/Greeter.sol"), PathBuf::from("src/tokens/Greeter.sol"),]
    );

    let mut artifacts = project.artifacts_snapshot().unwrap().artifacts;
    artifacts.strip_prefix_all(&project.paths().artifacts);

    assert_eq!(artifacts.len(), 2);
    let mut artifact_files = artifacts.artifact_files().map(|f| f.file.clone()).collect::<Vec<_>>();
    artifact_files.sort_unstable();

    assert_eq!(
        artifact_files,
        vec![
            PathBuf::from("Greeter.sol/Greeter.json"),
            PathBuf::from("tokens/Greeter.sol/Greeter.json"),
        ]
    );
}

// <https://github.com/foundry-rs/foundry/issues/2843>
#[test]
fn can_handle_conflicting_files_recompile() {
    let project = TempProject::<ConfigurableArtifacts>::dapptools().unwrap();

    project
        .add_source(
            "A",
            r#"
    pragma solidity ^0.8.10;

    contract A {
            function foo() public{}
    }
   "#,
        )
        .unwrap();

    project
        .add_source(
            "inner/A",
            r#"
    pragma solidity ^0.8.10;

    contract A {
            function bar() public{}
    }
   "#,
        )
        .unwrap();

    let compiled = project.compile().unwrap();
    assert!(!compiled.has_compiler_errors());

    let artifacts = compiled.artifacts().count();
    assert_eq!(artifacts, 2);

    // nothing to compile
    let compiled = project.compile().unwrap();
    assert!(compiled.is_unchanged());
    let artifacts = compiled.artifacts().count();
    assert_eq!(artifacts, 2);

    let cache = SolFilesCache::read(project.cache_path()).unwrap();

    let mut source_files = cache.files.keys().cloned().collect::<Vec<_>>();
    source_files.sort_unstable();

    assert_eq!(source_files, vec![PathBuf::from("src/A.sol"), PathBuf::from("src/inner/A.sol"),]);

    let mut artifacts =
        project.artifacts_snapshot().unwrap().artifacts.into_stripped_file_prefixes(project.root());
    artifacts.strip_prefix_all(&project.paths().artifacts);

    assert_eq!(artifacts.len(), 2);
    let mut artifact_files = artifacts.artifact_files().map(|f| f.file.clone()).collect::<Vec<_>>();
    artifact_files.sort_unstable();

    let expected_files = vec![PathBuf::from("A.sol/A.json"), PathBuf::from("inner/A.sol/A.json")];
    assert_eq!(artifact_files, expected_files);

    // overwrite conflicting nested file, effectively changing it
    project
        .add_source(
            "inner/A",
            r#"
    pragma solidity ^0.8.10;
    contract A {
    function bar() public{}
    function baz() public{}
    }
   "#,
        )
        .unwrap();

    let compiled = project.compile().unwrap();
    assert!(!compiled.has_compiler_errors());

    let mut recompiled_artifacts =
        project.artifacts_snapshot().unwrap().artifacts.into_stripped_file_prefixes(project.root());
    recompiled_artifacts.strip_prefix_all(&project.paths().artifacts);

    assert_eq!(recompiled_artifacts.len(), 2);
    let mut artifact_files =
        recompiled_artifacts.artifact_files().map(|f| f.file.clone()).collect::<Vec<_>>();
    artifact_files.sort_unstable();
    assert_eq!(artifact_files, expected_files);

    // ensure that `a.sol/A.json` is unchanged
    let outer = artifacts.find("src/A.sol", "A").unwrap();
    let outer_recompiled = recompiled_artifacts.find("src/A.sol", "A").unwrap();
    assert_eq!(outer, outer_recompiled);

    let inner_recompiled = recompiled_artifacts.find("src/inner/A.sol", "A").unwrap();
    assert!(inner_recompiled.get_abi().unwrap().functions.contains_key("baz"));
}

// <https://github.com/foundry-rs/foundry/issues/2843>
#[test]
fn can_handle_conflicting_files_case_sensitive_recompile() {
    let project = TempProject::<ConfigurableArtifacts>::dapptools().unwrap();

    project
        .add_source(
            "a",
            r#"
    pragma solidity ^0.8.10;

    contract A {
            function foo() public{}
    }
   "#,
        )
        .unwrap();

    project
        .add_source(
            "inner/A",
            r#"
    pragma solidity ^0.8.10;

    contract A {
            function bar() public{}
    }
   "#,
        )
        .unwrap();

    let compiled = project.compile().unwrap();
    assert!(!compiled.has_compiler_errors());

    let artifacts = compiled.artifacts().count();
    assert_eq!(artifacts, 2);

    // nothing to compile
    let compiled = project.compile().unwrap();
    assert!(compiled.is_unchanged());
    let artifacts = compiled.artifacts().count();
    assert_eq!(artifacts, 2);

    let cache = SolFilesCache::read(project.cache_path()).unwrap();

    let mut source_files = cache.files.keys().cloned().collect::<Vec<_>>();
    source_files.sort_unstable();

    assert_eq!(source_files, vec![PathBuf::from("src/a.sol"), PathBuf::from("src/inner/A.sol"),]);

    let mut artifacts =
        project.artifacts_snapshot().unwrap().artifacts.into_stripped_file_prefixes(project.root());
    artifacts.strip_prefix_all(&project.paths().artifacts);

    assert_eq!(artifacts.len(), 2);
    let mut artifact_files = artifacts.artifact_files().map(|f| f.file.clone()).collect::<Vec<_>>();
    artifact_files.sort_unstable();

    let expected_files = vec![PathBuf::from("a.sol/A.json"), PathBuf::from("inner/A.sol/A.json")];
    assert_eq!(artifact_files, expected_files);

    // overwrite conflicting nested file, effectively changing it
    project
        .add_source(
            "inner/A",
            r#"
    pragma solidity ^0.8.10;
    contract A {
    function bar() public{}
    function baz() public{}
    }
   "#,
        )
        .unwrap();

    let compiled = project.compile().unwrap();
    assert!(!compiled.has_compiler_errors());

    let mut recompiled_artifacts =
        project.artifacts_snapshot().unwrap().artifacts.into_stripped_file_prefixes(project.root());
    recompiled_artifacts.strip_prefix_all(&project.paths().artifacts);

    assert_eq!(recompiled_artifacts.len(), 2);
    let mut artifact_files =
        recompiled_artifacts.artifact_files().map(|f| f.file.clone()).collect::<Vec<_>>();
    artifact_files.sort_unstable();
    assert_eq!(artifact_files, expected_files);

    // ensure that `a.sol/A.json` is unchanged
    let outer = artifacts.find("src/a.sol", "A").unwrap();
    let outer_recompiled = recompiled_artifacts.find("src/a.sol", "A").unwrap();
    assert_eq!(outer, outer_recompiled);

    let inner_recompiled = recompiled_artifacts.find("src/inner/A.sol", "A").unwrap();
    assert!(inner_recompiled.get_abi().unwrap().functions.contains_key("baz"));
}

#[test]
fn can_checkout_repo() {
    let project = TempProject::checkout("transmissions11/solmate").unwrap();

    let compiled = project.compile().unwrap();
    assert!(!compiled.has_compiler_errors());
    let _artifacts = project.artifacts_snapshot().unwrap();
}

#[test]
fn can_detect_config_changes() {
    let mut project = TempProject::<ConfigurableArtifacts>::dapptools().unwrap();

    let remapping = project.paths().libraries[0].join("remapping");
    project
        .paths_mut()
        .remappings
        .push(Remapping::from_str(&format!("remapping/={}/", remapping.display())).unwrap());

    project
        .add_source(
            "Foo",
            r#"
    pragma solidity ^0.8.10;
    import "remapping/Bar.sol";

    contract Foo {}
   "#,
        )
        .unwrap();
    project
        .add_lib(
            "remapping/Bar",
            r#"
    pragma solidity ^0.8.10;

    contract Bar {}
    "#,
        )
        .unwrap();

    let compiled = project.compile().unwrap();
    assert!(!compiled.has_compiler_errors());

    let cache = SolFilesCache::read(&project.paths().cache).unwrap();
    assert_eq!(cache.files.len(), 2);

    // nothing to compile
    let compiled = project.compile().unwrap();
    assert!(compiled.is_unchanged());

    project.project_mut().solc_config.settings.optimizer.enabled = Some(true);

    let compiled = project.compile().unwrap();
    assert!(!compiled.has_compiler_errors());
    assert!(!compiled.is_unchanged());
}

#[test]
fn can_add_basic_contract_and_library() {
    let mut project = TempProject::<ConfigurableArtifacts>::dapptools().unwrap();

    let remapping = project.paths().libraries[0].join("remapping");
    project
        .paths_mut()
        .remappings
        .push(Remapping::from_str(&format!("remapping/={}/", remapping.display())).unwrap());

    let src = project.add_basic_source("Foo.sol", "^0.8.0").unwrap();

    let lib = project.add_basic_source("Bar", "^0.8.0").unwrap();

    let graph = Graph::resolve(project.paths()).unwrap();
    assert_eq!(graph.files().len(), 2);
    assert!(graph.files().contains_key(&src));
    assert!(graph.files().contains_key(&lib));

    let compiled = project.compile().unwrap();
    assert!(!compiled.has_compiler_errors());
    assert!(compiled.find_first("Foo").is_some());
    assert!(compiled.find_first("Bar").is_some());
}

// <https://github.com/foundry-rs/foundry/issues/2706>
#[test]
fn can_handle_nested_absolute_imports() {
    let mut project = TempProject::dapptools().unwrap();

    let remapping = project.paths().libraries[0].join("myDepdendency");
    project
        .paths_mut()
        .remappings
        .push(Remapping::from_str(&format!("myDepdendency/={}/", remapping.display())).unwrap());

    project
        .add_lib(
            "myDepdendency/src/interfaces/IConfig.sol",
            r#"
    pragma solidity ^0.8.10;

    interface IConfig {}
   "#,
        )
        .unwrap();

    project
        .add_lib(
            "myDepdendency/src/Config.sol",
            r#"
    pragma solidity ^0.8.10;
    import "src/interfaces/IConfig.sol";

    contract Config {}
   "#,
        )
        .unwrap();

    project
        .add_source(
            "Greeter",
            r#"
    pragma solidity ^0.8.10;
    import "myDepdendency/src/Config.sol";

    contract Greeter {}
   "#,
        )
        .unwrap();

    let compiled = project.compile().unwrap();
    assert!(!compiled.has_compiler_errors());
    assert!(compiled.find_first("Greeter").is_some());
    assert!(compiled.find_first("Config").is_some());
    assert!(compiled.find_first("IConfig").is_some());
}

#[test]
fn can_handle_nested_test_absolute_imports() {
    let project = TempProject::dapptools().unwrap();

    project
        .add_source(
            "Contract.sol",
            r#"
// SPDX-License-Identifier: UNLICENSED
pragma solidity =0.8.13;

library Library {
    function f(uint256 a, uint256 b) public pure returns (uint256) {
        return a + b;
    }
}

contract Contract {
    uint256 c;

    constructor() {
        c = Library.f(1, 2);
    }
}
   "#,
        )
        .unwrap();

    project
        .add_test(
            "Contract.t.sol",
            r#"
// SPDX-License-Identifier: UNLICENSED
pragma solidity =0.8.13;

import "src/Contract.sol";

contract ContractTest {
    function setUp() public {
    }

    function test() public {
        new Contract();
    }
}
   "#,
        )
        .unwrap();

    let compiled = project.compile().unwrap();
    assert!(!compiled.has_compiler_errors());
    assert!(compiled.find_first("Contract").is_some());
}
