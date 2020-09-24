use gumdrop::Options;

use std::process;

use ethers_cli::cli_common::{abigen::generate, Command, EthersCliOpts};

fn main() {
    let opts = EthersCliOpts::parse_args_default_or_exit();

    let command = opts.command.unwrap_or_else(|| {
        eprintln!("No command was provided.");
        eprintln!("{}", EthersCliOpts::usage());
        process::exit(2);
    });

    match command {
        Command::Abigen(opts) => {
            generate(opts);
        }
    }
}
