use ethers::{contract::Abigen, utils::Solc};

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args();
    args.next().unwrap(); // skip program name

    let contract_name = args.next().unwrap_or("SimpleStorage".to_owned());
    let contract: String = args.next().unwrap_or("examples/contract.sol".to_owned());

    println!("Generating bindings for {}\n", contract);

    // compile it if needed
    let abi = if contract.ends_with(".sol") {
        let contracts = Solc::new(&contract).build_raw()?;
        contracts.get(&contract_name).unwrap().abi.clone()
    } else {
        contract
    };

    let bindings = Abigen::new(&contract_name, abi)?.generate()?;

    // print to stdout if no output arg is given
    if let Some(output_path) = args.next() {
        bindings.write_to_file(&output_path)?;
    } else {
        bindings.write(std::io::stdout())?;
    }

    Ok(())
}
