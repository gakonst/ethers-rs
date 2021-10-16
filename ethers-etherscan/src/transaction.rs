use std::collections::HashMap;

use serde::Deserialize;

use crate::{Client, Response};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ContractExecutionStatus {
    is_error: String,
    err_description: String,
}

#[derive(Deserialize)]
struct TransactionReceiptStatus {
    status: String,
}

impl Client {
    /// Returns the status of a contract execution
    pub async fn check_contract_execution_status(
        &self,
        tx_hash: impl AsRef<str>,
    ) -> anyhow::Result<anyhow::Result<()>> {
        let mut map = HashMap::new();
        map.insert("txhash", tx_hash.as_ref());

        let query = self.create_query("transaction", "getstatus", map);
        let response: Response<ContractExecutionStatus> = self.get_json(&query).await?;

        if response.result.is_error == "0" {
            Ok(Ok(()))
        } else {
            Ok(Err(anyhow::anyhow!(response.result.err_description)))
        }
    }

    /// Returns the status of a transaction execution: `false` for failed and `true` for successful
    pub async fn check_transaction_receipt_status(
        &self,
        tx_hash: impl AsRef<str>,
    ) -> anyhow::Result<bool> {
        let mut map = HashMap::new();
        map.insert("txhash", tx_hash.as_ref());

        let query = self.create_query("transaction", "gettxreceiptstatus", map);
        let response: Response<TransactionReceiptStatus> = self.get_json(&query).await?;

        match response.result.status.as_str() {
            "0" => Ok(false),
            "1" => Ok(true),
            _ => Err(anyhow::anyhow!("bad status")),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Chain, Client};

    #[tokio::test]
    async fn check_contract_execution_status_success() {
        let client = Client::new_from_env(Chain::Mainnet).unwrap();

        let status = client
            .check_contract_execution_status(
                "0x16197e2a0eacc44c1ebdfddcfcfcafb3538de557c759a66e0ba95263b23d9007",
            )
            .await
            .unwrap();

        assert!(status.is_ok());
    }

    #[tokio::test]
    async fn check_contract_execution_status_error() {
        let client = Client::new_from_env(Chain::Mainnet).unwrap();

        let status = client
            .check_contract_execution_status(
                "0x15f8e5ea1079d9a0bb04a4c58ae5fe7654b5b2b4463375ff7ffb490aa0032f3a",
            )
            .await
            .unwrap();

        assert_eq!(status.unwrap_err().to_string(), "Bad jump destination");
    }

    #[tokio::test]
    async fn check_transaction_receipt_status_success() {
        let client = Client::new_from_env(Chain::Mainnet).unwrap();

        let success = client
            .check_transaction_receipt_status(
                "0x513c1ba0bebf66436b5fed86ab668452b7805593c05073eb2d51d3a52f480a76",
            )
            .await
            .unwrap();

        assert!(success);
    }

    #[tokio::test]
    async fn check_transaction_receipt_status_failed() {
        let client = Client::new_from_env(Chain::Mainnet).unwrap();

        let success = client
            .check_transaction_receipt_status(
                "0x21a29a497cb5d4bf514c0cca8d9235844bd0215c8fab8607217546a892fd0758",
            )
            .await
            .unwrap();

        assert!(!success);
    }
}
