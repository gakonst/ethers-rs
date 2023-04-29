use ethers_addressbook::{contract, Chain};

#[test]
fn test_tokens() {
    assert!(contract("dai").is_some());
    assert!(contract("usdc").is_some());
    assert!(contract("rand").is_none());
}

#[test]
fn test_addrs() {
    assert!(contract("dai").unwrap().address(Chain::Mainnet).is_some());
    assert!(contract("dai").unwrap().address(Chain::MoonbeamDev).is_none());
}
