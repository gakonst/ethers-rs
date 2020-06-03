use ethers_contract_abigen::Abigen;

fn main() {
    Abigen::new("ERC20Token", "./abi.json")
        .unwrap()
        .generate()
        .unwrap()
        .write_to_file("token.rs")
        .unwrap();
}
