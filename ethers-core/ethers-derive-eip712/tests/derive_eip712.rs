use ethers_contract_derive::EthAbiType;
use ethers_core::{
    types::{
        transaction::eip712::{
            EIP712Domain as Domain, Eip712, EIP712_DOMAIN_TYPE_HASH,
            EIP712_DOMAIN_TYPE_HASH_WITH_SALT,
        },
        Address, H160, U256,
    },
    utils::{keccak256, parse_ether},
};
use ethers_derive_eip712::*;

#[test]
fn test_derive_eip712() {
    #[derive(Debug, Clone, Eip712, EthAbiType)]
    #[eip712(
        name = "Radicle",
        version = "1",
        chain_id = 1,
        verifying_contract = "0x0000000000000000000000000000000000000001"
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

    assert_eq!(hash.len(), 32)
}

#[test]
fn test_struct_hash() {
    #[derive(Debug, Clone, Eip712, EthAbiType)]
    #[eip712(
        name = "Radicle",
        version = "1",
        chain_id = 1,
        verifying_contract = "0x0000000000000000000000000000000000000001",
        salt = "1234567890"
    )]
    pub struct EIP712Domain {
        name: String,
        version: String,
        chain_id: U256,
        verifying_contract: Address,
    }

    let domain = Domain {
        name: Some("Radicle".to_string()),
        version: Some("1".to_string()),
        chain_id: Some(U256::from(1)),
        verifying_contract: Some(Address::zero()),
        salt: None,
    };

    let domain_test = EIP712Domain {
        name: "Radicle".to_string(),
        version: "1".to_string(),
        chain_id: U256::from(1),
        verifying_contract: H160::from(&[0; 20]),
    };

    assert_eq!(EIP712_DOMAIN_TYPE_HASH, EIP712Domain::type_hash().unwrap());

    assert_eq!(domain.separator(), domain_test.struct_hash().unwrap());
}

#[test]
fn test_derive_eip712_nested() {
    #[derive(Debug, Clone, Eip712, EthAbiType)]
    #[eip712(
        name = "MyDomain",
        version = "1",
        chain_id = 1,
        verifying_contract = "0x0000000000000000000000000000000000000001"
    )]
    pub struct MyStruct {
        foo: String,
        bar: U256,
        addr: Address,
        /* #[eip712] // Todo: Support nested Eip712 structs
         * nested: MyNestedStruct, */
    }

    #[derive(Debug, Clone, Eip712, EthAbiType)]
    #[eip712(
        name = "MyDomain",
        version = "1",
        chain_id = 1,
        verifying_contract = "0x0000000000000000000000000000000000000001"
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
        /* nested: MyNestedStruct {
         *     foo: "foo".to_string(),
         *     bar: U256::from(1),
         *     addr: Address::from(&[0; 20]),
         * }, */
    };

    let hash = my_struct.struct_hash().expect("failed to hash struct");

    assert_eq!(hash.len(), 32)
}

#[test]
fn test_uniswap_v2_permit_hash() {
    // See examples/permit_hash.rs for comparison
    // the following produces the same permit_hash as in the example

    #[derive(Debug, Clone, Eip712, EthAbiType)]
    #[eip712(
        name = "Uniswap V2",
        version = "1",
        chain_id = 1,
        verifying_contract = "0xB4e16d0168e52d35CaCD2c6185b44281Ec28C9Dc"
    )]
    struct Permit {
        owner: Address,
        spender: Address,
        value: U256,
        nonce: U256,
        deadline: U256,
    }

    let permit = Permit {
        owner: "0x617072Cb2a1897192A9d301AC53fC541d35c4d9D".parse().unwrap(),
        spender: "0x2819c144D5946404C0516B6f817a960dB37D4929".parse().unwrap(),
        value: parse_ether(10).unwrap(),
        nonce: U256::from(1),
        deadline: U256::from(3133728498_u32),
    };

    let permit_hash = permit.encode_eip712().unwrap();

    assert_eq!(
        hex::encode(permit_hash),
        "7b90248477de48c0b971e0af8951a55974733455191480e1e117c86cc2a6cd03"
    );
}

#[test]
fn test_domain_hash_constants() {
    assert_eq!(
        EIP712_DOMAIN_TYPE_HASH,
        keccak256(
            "EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"
        )
    );
    assert_eq!(
        EIP712_DOMAIN_TYPE_HASH_WITH_SALT,
        keccak256("EIP712Domain(string name,string version,uint256 chainId,address verifyingContract,bytes32 salt)")
    );
}
