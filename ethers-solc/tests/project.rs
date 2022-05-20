//! project tests

use std::{
    collections::{BTreeMap, HashMap, HashSet},
    io,
    path::{Path, PathBuf},
    str::FromStr,
};

use ethers_core::types::Address;
use ethers_solc::{
    artifacts::{
        BytecodeHash, Libraries, ModelCheckerEngine::CHC, ModelCheckerSettings, UserDoc,
        UserDocNotice,
    },
    cache::{SolFilesCache, SOLIDITY_FILES_CACHE_FILENAME},
    project_util::*,
    remappings::Remapping,
    CompilerInput, ConfigurableArtifacts, ExtraOutputValues, Graph, Project, ProjectCompileOutput,
    ProjectPathsConfig, Solc, TestFileFilter,
};
use pretty_assertions::assert_eq;
use semver::Version;

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
fn can_compile_yul_sample() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-data/yul-sample");
    let paths = ProjectPathsConfig::builder().sources(root);
    let project = TempProject::<ConfigurableArtifacts>::new(paths).unwrap();

    let compiled = project.compile().unwrap();
    assert!(compiled.find("Dapp").is_some());
    assert!(compiled.find("SimpleStore").is_some());
    assert!(!compiled.has_compiler_errors());

    // nothing to compile
    let compiled = project.compile().unwrap();
    assert!(compiled.find("Dapp").is_some());
    assert!(compiled.find("SimpleStore").is_some());
    assert!(compiled.is_unchanged());

    let cache = SolFilesCache::read(project.cache_path()).unwrap();

    // delete artifacts
    std::fs::remove_dir_all(&project.paths().artifacts).unwrap();
    let compiled = project.compile().unwrap();
    assert!(compiled.find("Dapp").is_some());
    assert!(compiled.find("SimpleStore").is_some());
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
    let paths = ProjectPathsConfig::builder().sources(&root.join("contracts"));
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

    let compiled = tmp.project().compile_files(vec![f, bar]).unwrap();
    assert!(!compiled.has_compiler_errors());
    assert!(compiled.find("Foo").is_some());
    assert!(compiled.find("Bar").is_some());
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

    let contract = compiled.find("LinkTest").unwrap();
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

    assert!(compiled.find("MyLib").is_some());
    let contract = compiled.find("LinkTest").unwrap();
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

    assert!(compiled.find("MyLib").is_some());
    let contract = compiled.find("LinkTest").unwrap();
    let bytecode = &contract.bytecode.as_ref().unwrap().object;
    assert!(!bytecode.is_unlinked());

    let libs = Libraries::parse(&[format!("./src/MyLib.sol:MyLib:{:?}", Address::zero())]).unwrap();
    // provide the library settings to let solc link
    tmp.project_mut().solc_config.settings.libraries = libs.with_applied_remappings(tmp.paths());

    let compiled = tmp.compile().unwrap();
    assert!(!compiled.has_compiler_errors());

    assert!(compiled.find("MyLib").is_some());
    let contract = compiled.find("LinkTest").unwrap();
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

    assert!(compiled.find("MyLib").is_some());
    let contract = compiled.find("LinkTest").unwrap();
    let bytecode = &contract.bytecode.as_ref().unwrap().object;
    assert!(bytecode.is_unlinked());

    let libs =
        Libraries::parse(&[format!("remapping/MyLib.sol:MyLib:{:?}", Address::zero())]).unwrap(); // provide the library settings to let solc link
    tmp.project_mut().solc_config.settings.libraries = libs.with_applied_remappings(tmp.paths());

    let compiled = tmp.compile().unwrap();
    assert!(!compiled.has_compiler_errors());

    assert!(compiled.find("MyLib").is_some());
    let contract = compiled.find("LinkTest").unwrap();
    let bytecode = &contract.bytecode.as_ref().unwrap().object;
    assert!(!bytecode.is_unlinked());
}
#[test]
fn can_recompile_with_changes() {
    let mut tmp = TempProject::dapptools().unwrap();
    tmp.project_mut().allowed_lib_paths = vec![tmp.root().join("modules")].into();

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
    assert!(compiled.find("A").is_some());
    assert!(compiled.find("B").is_some());

    let compiled = tmp.compile().unwrap();
    assert!(compiled.find("A").is_some());
    assert!(compiled.find("B").is_some());
    assert!(compiled.is_unchanged());

    // modify A.sol
    tmp.add_source("A", format!("{}\n", content)).unwrap();
    let compiled = tmp.compile().unwrap();
    assert!(!compiled.has_compiler_errors());
    assert!(!compiled.is_unchanged());
    assert!(compiled.find("A").is_some());
    assert!(compiled.find("B").is_some());
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
    assert!(compiled.find("DeployProxy").is_some());
    assert!(compiled.find("UpgradeProxy").is_some());
    assert!(compiled.find("ProxyAdmin").is_some());

    let artifacts = tmp.artifacts_snapshot().unwrap();
    assert_eq!(artifacts.artifacts.as_ref().len(), 3);
    artifacts.assert_artifacts_essentials_present();

    let compiled = tmp.compile().unwrap();
    assert!(compiled.find("DeployProxy").is_some());
    assert!(compiled.find("UpgradeProxy").is_some());
    assert!(compiled.find("ProxyAdmin").is_some());
    assert!(compiled.is_unchanged());

    // modify upgradeProxy.sol
    tmp.add_source("upgradeProxy.sol", format!("{}\n", upgrade)).unwrap();
    let compiled = tmp.compile().unwrap();
    assert!(!compiled.has_compiler_errors());
    assert!(!compiled.is_unchanged());
    assert!(compiled.find("DeployProxy").is_some());
    assert!(compiled.find("UpgradeProxy").is_some());
    assert!(compiled.find("ProxyAdmin").is_some());

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
    assert!(compiled.find("A").is_some());
    assert!(compiled.find("C").is_some());

    let compiled = tmp.compile().unwrap();
    assert!(compiled.find("A").is_some());
    assert!(compiled.find("C").is_some());
    assert!(compiled.is_unchanged());

    // modify C.sol
    tmp.add_source("C", format!("{}\n", c)).unwrap();
    let compiled = tmp.compile().unwrap();
    assert!(!compiled.has_compiler_errors());
    assert!(!compiled.is_unchanged());
    assert!(compiled.find("A").is_some());
    assert!(compiled.find("C").is_some());
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

    tmp.add_source(
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

    assert!(compiled.find("ATest").is_some());
    assert!(compiled.find("MyLib").is_some());
    let lib = compiled.remove("MyLib").unwrap();
    assert!(lib.bytecode.is_some());
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
    assert!(compiled.find("A").is_some());
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

    assert!(compiled.find("Assert").is_some());
    assert!(!compiled.has_compiler_errors());
    assert!(compiled.has_compiler_warnings());
}

fn remove_solc_if_exists(version: &Version) {
    match Solc::find_svm_installed_version(version.to_string()).unwrap() {
        Some(_) => svm::remove_version(version).expect("failed to remove version"),
        None => {}
    };
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
pragma solidity {};
contract Contract {{ }}
"#,
                version
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
    assert!(compiled.find("Contract").is_some());
    let userdoc = compiled.remove("Contract").unwrap().userdoc;

    assert_eq!(
        userdoc,
        Some(UserDoc {
            version: None,
            kind: None,
            methods: BTreeMap::from([
                ("abc()".to_string(), UserDocNotice::Method { notice: "hello".to_string() }),
                ("xyz()".to_string(), UserDocNotice::Method { notice: "hello".to_string() }),
                ("constructor".to_string(), UserDocNotice::Constructor("hello".to_string())),
            ]),
            notice: None
        })
    );

    project.add_source("Contract", contract.replace("$VERSION", "^0.8.10")).unwrap();

    let mut compiled = project.compile().unwrap();
    assert!(!compiled.has_compiler_errors());
    assert!(!compiled.is_unchanged());
    assert!(compiled.find("Contract").is_some());
    let userdoc = compiled.remove("Contract").unwrap().userdoc;

    assert_eq!(
        userdoc,
        Some(UserDoc {
            version: Some(1),
            kind: Some("user".to_string()),
            methods: BTreeMap::from([
                ("abc()".to_string(), UserDocNotice::Method { notice: "hello".to_string() }),
                ("xyz()".to_string(), UserDocNotice::Method { notice: "hello".to_string() }),
                ("constructor".to_string(), UserDocNotice::Method { notice: "hello".to_string() }),
            ]),
            notice: None
        })
    );
}
