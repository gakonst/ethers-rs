use std::sync::Arc;

use ethers::{
    contract::{abigen, ContractFactory},
    middleware::SignerMiddleware,
    providers::{Http, Middleware, Provider, UserOperation},
    signers::{LocalWallet, Signer}, types::{Bytes, Address, U256, transaction::eip2718::TypedTransaction}, abi::ethereum_types::Signature, utils::hex
};
use eyre::Result;


/// `eth_sendUserOperation`
#[tokio::main]
async fn main() -> Result<()> {
    if let Ok(url) = std::env::var("RPC_URL") {

        abigen!(
            EntryPointContract,
            r#"[
                function getNonce(address, uint192) external view returns (uint256)
                function getSenderAddress(bytes) view returns (address)
                function createAccount(address, uint256) view returns (address)
            ]"#,
        );
        let provider = Provider::<Http>::try_from(url)?;
        let wallet: LocalWallet =
            "".parse()?;
        let from = wallet.address();
        println!("from: {:?}", from);

        let mut uo =  
            UserOperation {
                sender: "0x921f125A92930cabb2969AD9323261D3A2A784e7".parse().unwrap(),
                nonce: 1.into(),
                init_code: Bytes::default(),
                // transfer 0 eth
                call_data: "0xb61d27f6000000000000000000000000a02bfd0ba5d182226627a933333ba92d1a60e234000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000000".parse().unwrap(),
                call_gas_limit: 530_100.into(), 
                verification_gas_limit: 500_624.into(),
                pre_verification_gas: 104_056.into(),
                max_fee_per_gas: 1_695_000_030_u64.into(),
                max_priority_fee_per_gas: 1_695_000_000.into(),
                paymaster_and_data: Bytes::default(),
                signature: Bytes::default(),
            };
        let client =  Arc::new(SignerMiddleware::new(provider, wallet.clone()));
        let entry_point:Address = "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789".parse().unwrap();
        let entry_point_contract = EntryPointContract::new(entry_point, client.clone());
        let account_factory_address: Address = "0x9406Cc6185a346906296840746125a0E44976454".parse().unwrap();
        let account_factory_contract = EntryPointContract::new(account_factory_address, client.clone());
        let nonce = entry_point_contract.get_nonce("0x9c5754De1443984659E1b3a8d1931D83475ba29C".parse().unwrap(), 0.into()).call().await?;
        uo = uo.nonce(nonce);
        if nonce.eq(&U256::from(0)) {
            let call = account_factory_contract.create_account(from, U256::from(0));
            let tx: TypedTransaction = call.tx;
            println!("tx: {:?}", tx);
            let mut vec1:Vec<u8> = account_factory_address.as_bytes().to_vec();
            let vec2:Vec<u8> = tx.data().unwrap().clone().to_vec();
            vec1.extend(vec2);
            let init_code: Bytes = Bytes::from(vec1);
            println!("init_code: {:?}", init_code);
            // let sender_address = entry_point_contract.get_sender_address(init_code.clone()).await?;
            uo = uo.init_code(init_code);
            // uo = uo.sender(sender_address);
        };
        let uo_hash = uo.cal_uo_hash(entry_point, 5.into());
        let signature = wallet.sign_message(uo_hash.as_bytes()).await?;
        uo = uo.signature(signature.to_vec().into());
        println!("uo_hash: {:?}", uo_hash);
        println!("sig: {:?}", uo.signature);
        println!("uo: {:?}", uo);

        let pending_uo = client
            .send_user_operation(
                uo.clone(),
                entry_point     
            )
            .await 
            .unwrap();


        println!("Sent uo hash: {}\n", serde_json::to_string(&pending_uo)?);
    }

    Ok(())
}
