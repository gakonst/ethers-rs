use ethers_core::{types::TransactionRequest, utils::GanacheBuilder};
use ethers_providers::{Http, Provider};
use ethers_signers::Wallet;
use std::convert::TryFrom;

#[tokio::test]
async fn send_eth() {
    let port = 8545u64;
    let url = format!("http://localhost:{}", port).to_string();
    let _ganache = GanacheBuilder::new()
        .port(port)
        .mnemonic("abstract vacuum mammal awkward pudding scene penalty purchase dinner depart evoke puzzle")
        .spawn();

    // this private key belongs to the above mnemonic
    let wallet: Wallet = "380eb0f3d505f087e438eca80bc4df9a7faa24f868e69fc0440261a0fc0567dc"
        .parse().unwrap();

    // connect to the network
    let provider = Provider::<Http>::try_from(url.as_str()).unwrap();

    // connect the wallet to the provider
    let client = wallet.connect(provider);

    // craft the transaction
    let tx = TransactionRequest::new()
        .send_to_str("986eE0C8B91A58e490Ee59718Cca41056Cf55f24")
        .unwrap()
        .value(10000);

    let balance_before = client.get_balance(client.address(), None).await.unwrap();

    // send it!
    client.send_transaction(tx, None).await.unwrap();

    let balance_after = client.get_balance(client.address(), None).await.unwrap();

    assert!(balance_before > balance_after);
}
