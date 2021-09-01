use ethers_providers::{Http, Middleware, Provider};

use ethers_core::types::TransactionRequest;
use ethers_middleware::signer::SignerMiddleware;
use ethers_signers::{LocalWallet, Signer};
use std::{convert::TryFrom, time::Duration};

#[tokio::test]
#[cfg(not(feature = "celo"))]
async fn send_eth() {
    use ethers_core::utils::Ganache;

    let ganache = Ganache::new().spawn();

    // this private key belongs to the above mnemonic
    let wallet: LocalWallet = ganache.keys()[0].clone().into();
    let wallet2: LocalWallet = ganache.keys()[1].clone().into();

    // connect to the network
    let provider = Provider::<Http>::try_from(ganache.endpoint())
        .unwrap()
        .interval(Duration::from_millis(10u64));
    let chain_id = provider.get_chainid().await.unwrap().as_u64();
    let wallet = wallet.with_chain_id(chain_id);
    let provider = SignerMiddleware::new(provider, wallet);

    // craft the transaction
    let tx = TransactionRequest::new().to(wallet2.address()).value(10000);

    let balance_before = provider
        .get_balance(provider.address(), None)
        .await
        .unwrap();

    // send it!
    provider.send_transaction(tx, None).await.unwrap();

    let balance_after = provider
        .get_balance(provider.address(), None)
        .await
        .unwrap();

    assert!(balance_before > balance_after);
}

#[tokio::test]
#[cfg(not(feature = "celo"))]
async fn pending_txs_with_confirmations_testnet() {
    let provider =
        Provider::<Http>::try_from("https://rinkeby.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27")
            .unwrap()
            .interval(Duration::from_millis(3000));
    let chain_id = provider.get_chainid().await.unwrap();
    let wallet = "59c37cb6b16fa2de30675f034c8008f890f4b2696c729d6267946d29736d73e4"
        .parse::<LocalWallet>()
        .unwrap()
        .with_chain_id(chain_id.as_u64());
    let address = wallet.address();
    let provider = SignerMiddleware::new(provider, wallet);
    generic_pending_txs_test(provider, address).await;
}

#[cfg(not(feature = "celo"))]
use ethers_core::types::{
    transaction::eip2718::TypedTransaction, Address, Eip1559TransactionRequest,
};

// different keys to avoid nonce errors
#[tokio::test]
#[cfg(not(feature = "celo"))]
async fn websocket_pending_txs_with_confirmations_testnet() {
    let provider =
        Provider::connect("wss://rinkeby.infura.io/ws/v3/c60b0bb42f8a4c6481ecd229eddaca27")
            .await
            .unwrap()
            .interval(Duration::from_millis(3000));
    let chain_id = provider.get_chainid().await.unwrap();
    let wallet = "ff7f80c6e9941865266ed1f481263d780169f1d98269c51167d20c630a5fdc8a"
        .parse::<LocalWallet>()
        .unwrap()
        .with_chain_id(chain_id.as_u64());
    let address = wallet.address();
    let provider = SignerMiddleware::new(provider, wallet);
    generic_pending_txs_test(provider, address).await;
}

#[cfg(not(feature = "celo"))]
async fn generic_pending_txs_test<M: Middleware>(provider: M, who: Address) {
    let tx = TransactionRequest::new().to(who).from(who);
    let pending_tx = provider.send_transaction(tx, None).await.unwrap();
    let tx_hash = *pending_tx;
    let receipt = pending_tx.confirmations(3).await.unwrap().unwrap();
    // got the correct receipt
    assert_eq!(receipt.transaction_hash, tx_hash);
}

#[tokio::test]
#[cfg(not(feature = "celo"))]
async fn typed_txs() {
    let provider =
        Provider::<Http>::try_from("https://rinkeby.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27")
            .unwrap();

    let chain_id = provider.get_chainid().await.unwrap();
    let wallet = "87203087aed9246e0b2417e248752a1a0df4fdaf65085c11a2b48087ba036b41"
        .parse::<LocalWallet>()
        .unwrap()
        .with_chain_id(chain_id.as_u64());
    let address = wallet.address();
    let provider = SignerMiddleware::new(provider, wallet);

    async fn check_tx<M: Middleware>(provider: &M, tx: TypedTransaction, expected: u64) {
        let receipt = provider
            .send_transaction(tx, None)
            .await
            .unwrap()
            .await
            .unwrap()
            .unwrap();
        let tx = provider
            .get_transaction(receipt.transaction_hash)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(receipt.transaction_type, Some(expected.into()));
        assert_eq!(tx.transaction_type, Some(expected.into()));
    }

    let tx: TypedTransaction = TransactionRequest::new().from(address).to(address).into();
    check_tx(&provider, tx, 0).await;

    let tx: TypedTransaction = TransactionRequest::new()
        .from(address)
        .to(address)
        .with_access_list(vec![])
        .into();
    check_tx(&provider, tx, 1).await;

    let tx: TypedTransaction = Eip1559TransactionRequest::new()
        .from(address)
        .to(address)
        .into();
    check_tx(&provider, tx, 2).await;
}

#[tokio::test]
#[cfg(feature = "celo")]
async fn test_send_transaction() {
    // Celo testnet
    let provider = Provider::<Http>::try_from("https://alfajores-forno.celo-testnet.org")
        .unwrap()
        .interval(Duration::from_millis(3000u64));
    let chain_id = provider.get_chainid().await.unwrap().as_u64();

    // Funded with https://celo.org/developers/faucet
    // Please do not drain this account :)
    let wallet = "d652abb81e8c686edba621a895531b1f291289b63b5ef09a94f686a5ecdd5db1"
        .parse::<LocalWallet>()
        .unwrap()
        .with_chain_id(chain_id);
    let client = SignerMiddleware::new(provider, wallet);

    let balance_before = client.get_balance(client.address(), None).await.unwrap();
    let tx = TransactionRequest::pay(client.address(), 100);
    let _receipt = client
        .send_transaction(tx, None)
        .await
        .unwrap()
        .confirmations(3)
        .await
        .unwrap();
    let balance_after = client.get_balance(client.address(), None).await.unwrap();
    assert!(balance_before > balance_after);
}

#[tokio::test]
#[cfg(not(feature = "celo"))]
async fn send_transaction_handles_tx_from_field() {
    use ethers_core::utils::Ganache;

    // launch ganache
    let ganache = Ganache::new().spawn();

    // grab 2 wallets
    let signer: LocalWallet = ganache.keys()[0].clone().into();
    let other: LocalWallet = ganache.keys()[1].clone().into();

    // connect to the network
    let provider = Provider::try_from(ganache.endpoint()).unwrap();
    let provider = SignerMiddleware::new(provider, signer.clone());

    // sending a TransactionRequest with a from field of None should result
    // in a transaction from the signer address
    let request_from_none = TransactionRequest::new();
    let receipt = provider
        .send_transaction(request_from_none, None)
        .await
        .unwrap()
        .await
        .unwrap()
        .unwrap();
    let sent_tx = provider
        .get_transaction(receipt.transaction_hash)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(sent_tx.from, signer.address());

    // sending a TransactionRequest with the signer as the from address should
    // result in a transaction from the signer address
    let request_from_signer = TransactionRequest::new().from(signer.address());
    let receipt = provider
        .send_transaction(request_from_signer, None)
        .await
        .unwrap()
        .await
        .unwrap()
        .unwrap();
    let sent_tx = provider
        .get_transaction(receipt.transaction_hash)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(sent_tx.from, signer.address());

    // sending a TransactionRequest with a from address that is not the signer
    // should result in a transaction from the specified address
    let request_from_other = TransactionRequest::new().from(other.address());
    let receipt = provider
        .send_transaction(request_from_other, None)
        .await
        .unwrap()
        .await
        .unwrap()
        .unwrap();
    let sent_tx = provider
        .get_transaction(receipt.transaction_hash)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(sent_tx.from, other.address());
}

#[tokio::test]
#[cfg(feature = "celo")]
async fn deploy_and_call_contract() {
    use ethers_contract::ContractFactory;
    use ethers_core::{
        abi::Abi,
        types::{BlockNumber, Bytes, H256, U256},
        utils::Solc,
    };
    use std::sync::Arc;

    fn compile_contract(name: &str, filename: &str) -> (Abi, Bytes) {
        let compiled = Solc::new(&format!("./tests/solidity-contracts/{}", filename))
            .build()
            .unwrap();
        let contract = compiled.get(name).expect("could not find contract");
        (contract.abi.clone(), contract.bytecode.clone())
    }

    let (abi, bytecode) = compile_contract("SimpleStorage", "SimpleStorage.sol");

    // Celo testnet
    let provider = Provider::<Http>::try_from("https://alfajores-forno.celo-testnet.org")
        .unwrap()
        .interval(Duration::from_millis(6000));
    let chain_id = provider.get_chainid().await.unwrap().as_u64();

    // Funded with https://celo.org/developers/faucet
    let wallet = "58ea5643a78c36926ad5128a6b0d8dfcc7fc705788a993b1c724be3469bc9697"
        .parse::<LocalWallet>()
        .unwrap()
        .with_chain_id(chain_id);

    let client = SignerMiddleware::new(provider, wallet);
    let client = Arc::new(client);

    let factory = ContractFactory::new(abi, bytecode, client);
    let deployer = factory.deploy(()).unwrap().legacy();
    let contract = deployer.block(BlockNumber::Pending).send().await.unwrap();

    let value: U256 = contract.method("value", ()).unwrap().call().await.unwrap();
    assert_eq!(value, 0.into());

    // make a state mutating transaction
    // gas estimation costs are sometimes under-reported on celo,
    // so we manually set it to avoid failures
    let call = contract
        .method::<_, H256>("setValue", U256::from(1))
        .unwrap()
        .gas(100000);
    let pending_tx = call.send().await.unwrap();
    let _receipt = pending_tx.await.unwrap();

    let value: U256 = contract.method("value", ()).unwrap().call().await.unwrap();
    assert_eq!(value, 1.into());
}
