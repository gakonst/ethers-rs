use std::collections::BTreeMap;
use std::path::PathBuf;

use ::ethers::solc::artifacts::{
    EvmVersion, Optimizer, OptimizerDetails, Settings, SettingsMetadata, YulDetails,
};
use ::ethers::solc::{AllowedLibPaths, Project, ProjectPathsConfig, Solc, SolcConfig};

fn main() {
    println!("Compiling Yul Contracts");

    //initialize the solc settings
    let solc_settings = Settings {
        stop_after: Some("TODO:".to_string()),
        remappings: vec![],
        optimizer: Optimizer {
            enabled: Some(false),
            runs: Some(200),
            details: Some(OptimizerDetails {
                peephole: Some(false),
                inliner: Some(false),
                jumpdest_remover: Some(false),
                order_literals: Some(false),
                deduplicate: Some(false),
                cse: Some(false),
                constant_optimizer: Some(false),
                yul: Some(true),
                yul_details: Some(YulDetails {
                    stack_allocation: Some(false),
                    optimizer_steps: Some("TODO:".to_string()),
                }),
            }),
        },
        metadata: Some(SettingsMetadata {
            use_literal_content: Some(false),
            bytecode_hash: Some("TODO:".to_string()),
        }),

        //TODO: this needs to be BTreeMap<String, BTreeMap<String, Vec<String>>>
        output_selection: BTreeMap::new(),
        evm_version: Some(EvmVersion::Byzantium),

        //TODO: This also needs to be BTreeMap<String, BTreeMap<String, Vec<String>>>
        libraries: BTreeMap::new(),
    };

    //create a new project
    let yul_project = Project {
        /// The layout of the project
        paths: ProjectPathsConfig {
            root: PathBuf::from("./"),
            cache: PathBuf::from("./cache"),
            artifacts: PathBuf::from("./artifacts"),
            sources: PathBuf::from("./yul_contracts"),
            tests: PathBuf::from("./tests"),
            libraries: vec![],
            remappings: vec![],
        },
        /// Where to find solc
        solc: Solc {
            //TODO: pass in the path of solc
            solc: PathBuf::from("TODO:"),
            args: vec![],
        },
        /// How solc invocation should be configured.
        solc_config: SolcConfig { settings: solc_settings },
        /// Whether caching is enabled
        cached: false,
        /// Whether writing artifacts to disk is enabled
        no_artifacts: false,
        /// Whether writing artifacts to disk is enabled
        auto_detect: false,
        /// Handles all artifacts related tasks, reading and writing from the artifact dir.
        // artifacts: //TODO:
        /// Errors/Warnings which match these error codes are not going to be logged
        ignored_error_codes: vec![],
        /// The paths which will be allowed for library inclusion
        allowed_lib_paths: AllowedLibPaths,
        /// Maximum number of `solc` processes to run simultaneously.
        solc_jobs: 1,
        /// Offline mode, if set, network access (download solc) is disallowed
        offline: true,
    };

    //compile the yul project
    yul_project.svm_compile_yul();
}
