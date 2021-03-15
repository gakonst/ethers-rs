//! Test cases to validate the `abigen!` macro
use ethers_contract::{abigen, EthEvent};

abigen!(
    SimpleContract,
    r#"[
        function setValue(string)
        function getValue() external view returns (string)
        event ValueChanged(address indexed author, string oldValue, string newValue)
    ]"#,
    event_derives(serde::Deserialize, serde::Serialize)
);

#[test]
fn can_gen_human_readable() {
    assert_eq!("ValueChanged", ValueChangedFilter::name());
    assert_eq!(
        "ValueChanged(address,string,string)",
        ValueChangedFilter::abi_signature()
    );
}
