use ethers_contract_abigen::Abigen;
use std::path::Path;

fn main() {
    let bindings = Abigen::new("MulticallContract", include_str!("src/multicall/multicall.json"))
        .unwrap()
        .generate()
        .unwrap();
    let tokens = bindings.into_tokens();
    // this is a hack to make the generated rust code compile in this crate (`ethers_contract`)
    let code = tokens.to_string().replace("ethers_contract", "crate").replace("crate :: {", "crate :: {self as ethers_contract,");

    let out_dir = std::env::var("OUT_DIR").expect("cargo OUT_DIR var should be set");

    std::fs::write(Path::new(&out_dir).join("multicall.rs"), code).expect("Failed to write code");
}
