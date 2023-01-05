use ethers::{prelude::Abigen, solc::Solc};
use eyre::Result;

fn main() -> Result<()> {
    let mut args = std::env::args();
    args.next().unwrap(); // skip program name

    let contract_name = args.next().unwrap_or_else(|| "SimpleStorage".to_owned());
    let contract: String = args
        .next()
        .unwrap_or_else(|| "examples/contracts/examples/contracts/contract.sol".to_owned());

    println!("Generating bindings for {contract}\n");

    // compile it
    let abi = if contract.ends_with(".sol") {
        let contracts = Solc::default().compile_source(&contract)?;
        let abi = contracts.get(&contract, &contract_name).unwrap().abi.unwrap();
        serde_json::to_string(abi).unwrap()
    } else {
        contract
    };

    let bindings = Abigen::new(&contract_name, abi)?.generate()?;

    // print to stdout if no output arg is given
    if let Some(output_path) = args.next() {
        bindings.write_to_file(output_path)?;
    } else {
        bindings.write(&mut std::io::stdout())?;
    }

    Ok(())
}
