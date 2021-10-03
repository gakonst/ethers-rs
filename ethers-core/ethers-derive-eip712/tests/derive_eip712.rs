use ethers_contract::EthAbiType;
use ethers_core::types::{
    transaction::eip712::{eip712_domain_type_hash, EIP712Domain as Domain, Eip712},
    Address, H160, U256,
};
use ethers_derive_eip712::*;

#[test]
fn test_derive_eip712() {
    #[derive(Debug, Clone, Eip712, EthAbiType)]
    #[eip712(
        name = "Radicle",
        version = "1",
        chain_id = 1,
        verifying_contract = "0x0000000000000000000000000000000000000000"
    )]
    pub struct Puzzle {
        pub organization: H160,
        pub contributor: H160,
        pub commit: String,
        pub project: String,
    }

    let puzzle = Puzzle {
        organization: "0000000000000000000000000000000000000000"
            .parse::<H160>()
            .expect("failed to parse address"),
        contributor: "0000000000000000000000000000000000000000"
            .parse::<H160>()
            .expect("failed to parse address"),
        commit: "5693b7019eb3e4487a81273c6f5e1832d77acb53".to_string(),
        project: "radicle-reward".to_string(),
    };

    let hash = puzzle.encode_eip712().expect("failed to encode struct");

    println!("Hash: {:?}", hash);

    assert_eq!(hash.len(), 32)
}

#[test]
fn test_struct_hash() {
    #[derive(Debug, Clone, Eip712, EthAbiType)]
    #[eip712(
        name = "Radicle",
        version = "1",
        chain_id = 1,
        verifying_contract = "0x0000000000000000000000000000000000000000"
    )]
    pub struct EIP712Domain {
        name: String,
        version: String,
        chain_id: U256,
        verifying_contract: Address,
    }

    let domain = Domain {
        name: "Radicle".to_string(),
        version: "1".to_string(),
        chain_id: U256::from(1),
        verifying_contract: H160::from(&[0; 20]),
    };

    let domain_test = EIP712Domain {
        name: "Radicle".to_string(),
        version: "1".to_string(),
        chain_id: U256::from(1),
        verifying_contract: H160::from(&[0; 20]),
    };

    assert_eq!(
        eip712_domain_type_hash(),
        EIP712Domain::type_hash().unwrap()
    );

    assert_eq!(domain.separator(), domain_test.struct_hash().unwrap());
}

#[test]
fn test_derive_eip712_nested() {
    #[derive(Debug, Clone, Eip712, EthAbiType)]
    #[eip712(
        name = "MyDomain",
        version = "1",
        chain_id = 1,
        verifying_contract = "0x0000000000000000000000000000000000000000"
    )]
    pub struct MyStruct {
        foo: String,
        bar: U256,
        addr: Address,
        // #[eip712] // Todo: Support nested Eip712 structs
        // nested: MyNestedStruct,
    }

    #[derive(Debug, Clone, Eip712, EthAbiType)]
    #[eip712(
        name = "MyDomain",
        version = "1",
        chain_id = 1,
        verifying_contract = "0x0000000000000000000000000000000000000000"
    )]
    pub struct MyNestedStruct {
        foo: String,
        bar: U256,
        addr: Address,
    }

    let my_struct = MyStruct {
        foo: "foo".to_string(),
        bar: U256::from(1),
        addr: Address::from(&[0; 20]),
        // nested: MyNestedStruct {
        //     foo: "foo".to_string(),
        //     bar: U256::from(1),
        //     addr: Address::from(&[0; 20]),
        // },
    };

    let hash = my_struct.struct_hash().expect("failed to hash struct");

    assert_eq!(hash.len(), 32)
}
