//! `abigen` subcommand

#![allow(clippy::never_loop)]

use abscissa_core::{Command, Options, Runnable};
use std::io::stdout;

use ethers_contract::Abigen;
use ethers_solc::Solc;

/// `abigen` subcommand
#[derive(Command, Debug, Default, Options)]
pub struct AbigenCmd {
    #[options(free, help = "abigen [contract_name] [contract_source]")]
    pub args: Vec<String>,
}

impl Runnable for AbigenCmd {
    fn run(&self) {
        let name = self.args.get(0).expect("contract name is required");
        let contract = self.args.get(1).expect("contract name is required");

        // compile it
        let abi = if contract.ends_with(".sol") {
            let contracts = Solc::default().compile_source(&contract).unwrap();
            let abi = contracts.get(&contract, &name).unwrap().abi.unwrap();
            serde_json::to_string(abi).unwrap()
        } else {
            contract.clone()
        };

        let bindings = Abigen::new(&name, abi).unwrap().generate().unwrap();

        // Print to stdout if no output arg is given
        if let Some(output_path) = self.args.get(2) {
            bindings.write_to_file(&output_path);
        } else {
            bindings.write(stdout());
        }
    }
}
