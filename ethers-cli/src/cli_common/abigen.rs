use gumdrop::Options;

use std::{io::stdout, process};

use ethers_contract::Abigen;

#[derive(Debug, Options)]
pub struct AbigenOpts {
    help: bool,
    #[options(required, help = "name of the contract")]
    name: String,
    #[options(required, help = "source of the contract ABI")]
    source: String,
    #[options(short = "o", help = "output directory for bindings")]
    output: Option<String>,
}

pub fn generate(opts: AbigenOpts) {
    let bindings = Abigen::new(&opts.name, opts.source)
        .unwrap_or_else(|err| {
            eprintln!("Failed to instantiate Abigen builder: {:?}", err);
            eprintln!("{}", AbigenOpts::usage());
            process::exit(2);
        })
        .generate()
        .unwrap_or_else(|err| {
            eprintln!("Failed to generate bindings: {:?}", err);
            eprintln!("{}", AbigenOpts::usage());
            process::exit(2);
        });

    match opts.output {
        Some(out_path) => {
            bindings
                .write_to_file(&out_path)
                .expect(&format!("Failed to write bindings to path: {}", out_path));
        }
        None => {
            bindings
                .write(stdout())
                .expect("Failed to write bindings to stdout");
        }
    }
}
