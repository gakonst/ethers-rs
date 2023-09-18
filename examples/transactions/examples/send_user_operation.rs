use std::sync::Arc;

use ethers::{
    contract::abigen,
    middleware::SignerMiddleware,
    providers::{
        user_operation::{UserOperationByHash, UserOperationReceipt},
        Http, Middleware, Provider, UserOperation,
    },
    signers::{LocalWallet, Signer},
    types::{transaction::eip2718::TypedTransaction, Address, Bytes, U256},
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

        // add privateKey
        let wallet: LocalWallet = "".parse()?;
        let from = wallet.address();
        println!("from: {:?}", from);

        let mut uo =  
            UserOperation {
                sender: Address::default(),
                nonce: U256::default(),
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
        let client = Arc::new(SignerMiddleware::new(provider, wallet.clone()));

        // get preferred entryPoint by the client
        let supported_entry_points = client.get_supported_entry_points().await.unwrap();
        let entry_point: Address = supported_entry_points[0].into();

        let entry_point_contract = EntryPointContract::new(entry_point, client.clone());
        let account_factory_address: Address =
            "0x9406Cc6185a346906296840746125a0E44976454".parse().unwrap();
        let account_factory_contract =
            EntryPointContract::new(account_factory_address, client.clone());
        let init_code: Bytes;
        let call = account_factory_contract.create_account(from, U256::from(0));
        let tx: TypedTransaction = call.tx;

        let mut vec1: Vec<u8> = account_factory_address.as_bytes().to_vec();
        let vec2: Vec<u8> = tx.data().unwrap().clone().to_vec();
        vec1.extend(vec2);
        init_code = Bytes::from(vec1);
        println!("init_code: {:?}", init_code);

        let sender_addr_result =
            entry_point_contract.get_sender_address(init_code.clone()).call().await;
        let sender: Address = match sender_addr_result {
            Ok(sender) => sender,
            Err(err) => {
                let data: Bytes = err.as_revert().unwrap().clone();
                let address_array: Result<[u8; 20], _> = data[data.len() - 20..].try_into();
                let address: Address = address_array.unwrap().into();
                address
            }
        };

        let nonce = entry_point_contract.get_nonce(sender, 0.into()).call().await?;
        uo = uo.nonce(nonce);
        uo = uo.sender(sender);
        if nonce.eq(&U256::from(0)) {
            uo = uo.init_code(init_code.clone());
        };

        // use dummy signature
        uo = uo.signature("0xfffffffffffffffffffffffffffffff0000000000000000000000000000000007aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa1c".parse::<Bytes>().unwrap());

        // estimate user operation gas
        let pending_estimation =
            client.estimate_user_operation_gas(uo.clone(), entry_point).await.unwrap();
        println!("estimation result: {}\n", serde_json::to_string(&pending_estimation)?);

        uo = uo.pre_verification_gas(pending_estimation.pre_verification_gas);
        uo = uo.verification_gas_limit(pending_estimation.verification_gas_limit);
        uo = uo.call_gas_limit(pending_estimation.call_gas_limit);

        let uo_hash = uo.cal_uo_hash(entry_point, 5.into());
        let signature = wallet.sign_message(uo_hash.as_bytes()).await?;
        uo = uo.signature(signature.to_vec().into());

        println!("user_operation: {:?}", uo);

        let pending_uo = client.send_user_operation(uo.clone(), entry_point).await.unwrap();

        println!("Sent uo hash: {}\n", serde_json::to_string(&pending_uo)?);

        let mut user_operation_by_hash: Option<UserOperationByHash>;
        loop {
            user_operation_by_hash = client.get_user_operation(pending_uo).await.unwrap();

            if !user_operation_by_hash.is_none() {
                break;
            }
        }
        println!("user_operation_by_hash: {}\n", serde_json::to_string(&user_operation_by_hash)?);

        let mut user_operation_receipt: Option<UserOperationReceipt>;
        loop {
            user_operation_receipt = client.get_user_operation_receipt(pending_uo).await.unwrap();

            if !user_operation_receipt.is_none() {
                break;
            }
        }
        println!("user_operation_receipt: {}\n", serde_json::to_string(&user_operation_receipt)?);
    }

    Ok(())
}
