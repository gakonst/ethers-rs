//! `abigen` subcommand

#![allow(clippy::never_loop)]

use abscissa_core::{Clap, Command, Runnable};
use std::{io::stdout, path::Path};

use ethers_contract::Abigen;
use ethers_solc::Solc;

/// `abigen` subcommand
#[derive(Command, Debug, Default, Clap)]
pub struct AbigenCmd {
    #[clap()]
    pub args: Vec<String>,
}

impl Runnable for AbigenCmd {
    fn run(&self) {
        let name = self.args.get(0).expect("Contract name is required");

        let contract = self.args.get(1).expect("Contract is required");

        // compile it
        let abi = if contract.ends_with("sol") {
            let contracts = Path::new(&contract);
            let contracts = Solc::default().compile_source(contracts).expect("file not found");
            let abi = contracts
                .get(&contract, &name)
                .unwrap()
                .abi
                .expect("Failed to get contract and name");
            serde_json::to_string(abi).unwrap()
        } else {
            contract.clone()
        };

        let bindings = Abigen::new(&name, abi).unwrap().generate().expect("could not get abi");

        // Print to stdout if no output arg is given
        if let Some(output_path) = self.args.get(2) {
            bindings.write_to_file(&output_path);
        } else {
            bindings.write(stdout());
        }
    }
}
