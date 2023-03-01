# ethers-signers

A unified interface for locally signing Ethereum transactions.

You can implement the `Signer` trait to extend functionality to other signers
such as Hardware Security Modules, KMS etc.

The exposed interfaces return a recoverable signature. In order to convert the
signature and the [`TransactionRequest`] to a [`Transaction`], look at the
signing middleware.

Supported signers:

-   [Private key](./src/wallet)
-   [Ledger](./src/ledger)
-   [Trezor](./src/trezor)
-   [YubiHSM2](./src/wallet/yubi.rs)
-   [AWS KMS](./src/aws)

For more information, please refer to the [book](https://gakonst.com/ethers-rs).

[`transaction`]: ethers_core::types::Transaction
[`transactionrequest`]: ethers_core::types::TransactionRequest

## Examples

```rust,no_run
# use ethers_signers::{LocalWallet, Signer};
# use ethers_core::{k256::ecdsa::SigningKey, types::TransactionRequest};
# async fn foo() -> Result<(), Box<dyn std::error::Error>> {
// instantiate the wallet
let wallet = "dcf2cbdd171a21c480aa7f53d77f31bb102282b3ff099c78e3118b37348c72f7"
    .parse::<LocalWallet>()?;

// create a transaction
let tx = TransactionRequest::new()
    .to("vitalik.eth") // this will use ENS
    .value(10000).into();

// sign it
let signature = wallet.sign_transaction(&tx).await?;

// can also sign a message
let signature = wallet.sign_message("hello world").await?;
signature.verify("hello world", wallet.address()).unwrap();
# Ok(())
# }
```

Sign an Ethereum prefixed message ([eip-712](https://eips.ethereum.org/EIPS/eip-712)):

```rust,no_run
# use ethers_signers::{Signer, LocalWallet};
# async fn foo() -> Result<(), Box<dyn std::error::Error>> {
let message = "Some data";
let wallet = LocalWallet::new(&mut rand::thread_rng());

// Sign the message
let signature = wallet.sign_message(message).await?;

// Recover the signer from the message
let recovered = signature.recover(message)?;

assert_eq!(recovered, wallet.address());
# Ok(())
# }
```
