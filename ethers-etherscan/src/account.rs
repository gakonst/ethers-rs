use std::{
    borrow::Cow,
    collections::HashMap,
    fmt::{Display, Error, Formatter},
};

use serde::{Deserialize, Serialize};

use crate::{Client, EtherscanError, Query, Response, Result};

/// The raw response from the balance-related API endpoints
#[derive(Debug, Serialize, Deserialize)]
pub struct AccountBalance {
    pub account: String,
    pub balance: String,
}

/// The raw response from the transaction list API endpoint
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NormalTransaction {
    pub is_error: String,
    pub block_number: String,
    pub time_stamp: String,
    pub hash: String,
    pub nonce: String,
    pub block_hash: String,
    pub transaction_index: String,
    pub from: String,
    pub to: String,
    pub value: String,
    pub gas: String,
    pub gas_price: String,
    #[serde(rename = "txreceipt_status")]
    pub tx_receipt_status: String,
    pub input: String,
    pub contract_address: String,
    pub gas_used: String,
    pub cumulative_gas_used: String,
    pub confirmations: String,
}

/// The raw response from the internal transaction list API endpoint
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InternalTransaction {
    pub block_number: String,
    pub time_stamp: String,
    pub hash: String,
    pub from: String,
    pub to: String,
    pub value: String,
    pub contract_address: String,
    pub input: String,
    #[serde(rename = "type")]
    pub result_type: String,
    pub gas: String,
    pub gas_used: String,
    pub trace_id: String,
    pub is_error: String,
    pub err_code: String,
}

/// The raw response from the ERC20 transfer list API endpoint
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ERC20TokenTransferEvent {
    pub block_number: String,
    pub time_stamp: String,
    pub hash: String,
    pub nonce: String,
    pub block_hash: String,
    pub from: String,
    pub contract_address: String,
    pub to: String,
    pub value: String,
    pub token_name: String,
    pub token_symbol: String,
    pub token_decimal: String,
    pub transaction_index: String,
    pub gas: String,
    pub gas_price: String,
    pub gas_used: String,
    pub cumulative_gas_used: String,
    pub input: String,
    pub confirmations: String,
}

/// The raw response from the ERC721 transfer list API endpoint
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ERC721TokenTransferEvent {
    pub block_number: String,
    pub time_stamp: String,
    pub hash: String,
    pub nonce: String,
    pub block_hash: String,
    pub from: String,
    pub contract_address: String,
    pub to: String,
    #[serde(rename = "tokenID")]
    pub token_id: String,
    pub token_name: String,
    pub token_symbol: String,
    pub token_decimal: String,
    pub transaction_index: String,
    pub gas: String,
    pub gas_price: String,
    pub gas_used: String,
    pub cumulative_gas_used: String,
    pub input: String,
    pub confirmations: String,
}

/// The raw response from the mined blocks API endpoint
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MinedBlock {
    pub block_number: String,
    pub time_stamp: String,
    pub block_reward: String,
}

/// The pre-defined block parameter for balance API endpoints
pub enum Tag {
    Earliest,
    Pending,
    Latest,
}

impl Display for Tag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match self {
            Tag::Earliest => write!(f, "earliest"),
            Tag::Pending => write!(f, "pending"),
            Tag::Latest => write!(f, "latest"),
        }
    }
}

impl Default for Tag {
    fn default() -> Self {
        Tag::Latest
    }
}

/// The list sorting preference
pub enum Sort {
    Asc,
    Desc,
}

impl Display for Sort {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match self {
            Sort::Asc => write!(f, "asc"),
            Sort::Desc => write!(f, "desc"),
        }
    }
}

/// Common optional arguments for the transaction or event list API endpoints
pub struct TxListParams {
    start_block: u64,
    end_block: u64,
    page: u64,
    offset: u64,
    sort: Sort,
}

impl TxListParams {
    pub fn new(start_block: u64, end_block: u64, page: u64, offset: u64, sort: Sort) -> Self {
        Self { start_block, end_block, page, offset, sort }
    }
}

impl Default for TxListParams {
    fn default() -> Self {
        Self { start_block: 0, end_block: 99999999, page: 0, offset: 10000, sort: Sort::Asc }
    }
}

impl From<TxListParams> for HashMap<&'static str, String> {
    fn from(tx_params: TxListParams) -> Self {
        let mut params = HashMap::new();
        params.insert("startBlock", tx_params.start_block.to_string());
        params.insert("endBlock", tx_params.end_block.to_string());
        params.insert("page", tx_params.page.to_string());
        params.insert("offset", tx_params.offset.to_string());
        params.insert("sort", tx_params.sort.to_string());
        params
    }
}

/// Options for querying internal transactions
pub enum InternalTxQueryOption<A> {
    ByAddress(A),
    ByTransactionHash(A),
    ByBlockRange,
}

/// Options for querying ERC20 or ERC721 token transfers
pub enum TokenQueryOption<A> {
    ByAddress(A),
    ByContract(A),
    ByAddressAndContract(A, A),
}

impl<'a, A: Into<Cow<'a, str>>> TokenQueryOption<A> {
    pub fn into_params(self, list_params: TxListParams) -> HashMap<&'static str, String> {
        let mut params: HashMap<&'static str, String> = list_params.into();
        match self {
            TokenQueryOption::ByAddress(address) => {
                params.insert("address", address.into().into_owned());
                params
            }
            TokenQueryOption::ByContract(contract) => {
                params.insert("contractaddress", contract.into().into_owned());
                params
            }
            TokenQueryOption::ByAddressAndContract(address, contract) => {
                params.insert("address", address.into().into_owned());
                params.insert("contractaddress", contract.into().into_owned());
                params
            }
        }
    }
}

/// The pre-defined block type for retrieving mined blocks
pub enum BlockType {
    CanonicalBlocks,
    Uncles,
}

impl Display for BlockType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match self {
            BlockType::CanonicalBlocks => write!(f, "blocks"),
            BlockType::Uncles => write!(f, "uncles"),
        }
    }
}

impl Default for BlockType {
    fn default() -> Self {
        BlockType::CanonicalBlocks
    }
}

impl Client {
    /// Returns the Ether balance of a given address.
    ///
    /// ```no_run
    /// # use ethers_etherscan::Client;
    /// # use ethers_core::types::Chain;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    ///     let client = Client::new(Chain::Mainnet, "API_KEY").unwrap();
    ///     let balance = client
    ///         .get_ether_balance_single("0x58eB28A67731c570Ef827C365c89B5751F9E6b0a", None)
    ///         .await.unwrap();
    /// # }
    /// ```
    pub async fn get_ether_balance_single(
        &self,
        address: impl AsRef<str>,
        tag: Option<Tag>,
    ) -> Result<AccountBalance> {
        let tag_str = tag.unwrap_or_default().to_string();
        let query = self.create_query(
            "account",
            "balance",
            HashMap::from([("address", address.as_ref()), ("tag", &tag_str)]),
        );
        let response: Response<String> = self.get_json(&query).await?;

        match response.status.as_str() {
            "0" => Err(EtherscanError::BalanceFailed),
            "1" => Ok(AccountBalance {
                account: address.as_ref().to_string(),
                balance: response.result,
            }),
            err => Err(EtherscanError::BadStatusCode(err.to_string())),
        }
    }

    /// Returns the balance of the accounts from a list of addresses.
    ///
    /// ```no_run
    /// # use ethers_etherscan::Client;
    /// # use ethers_core::types::Chain;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    ///     let client = Client::new(Chain::Mainnet, "API_KEY").unwrap();
    ///     let balances = client
    ///         .get_ether_balance_multi(&vec!["0x58eB28A67731c570Ef827C365c89B5751F9E6b0a"], None)
    ///         .await.unwrap();
    /// # }
    /// ```
    pub async fn get_ether_balance_multi<A: AsRef<str>>(
        &self,
        addresses: &[A],
        tag: Option<Tag>,
    ) -> Result<Vec<AccountBalance>> {
        let tag_str = tag.unwrap_or_default().to_string();
        let addrs = addresses.iter().map(AsRef::as_ref).collect::<Vec<&str>>().join(",");

        let query: Query<HashMap<&str, &str>> = self.create_query(
            "account",
            "balancemulti",
            HashMap::from([("address", addrs.as_ref()), ("tag", tag_str.as_ref())]),
        );
        let response: Response<Vec<AccountBalance>> = self.get_json(&query).await?;

        match response.status.as_str() {
            "0" => Err(EtherscanError::BalanceFailed),
            "1" => Ok(response.result),
            err => Err(EtherscanError::BadStatusCode(err.to_string())),
        }
    }

    /// Returns the list of transactions performed by an address, with optional pagination.
    ///
    /// ```no_run
    /// # use ethers_etherscan::Client;
    /// # use ethers_core::types::Chain;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    ///     let client = Client::new(Chain::Mainnet, "API_KEY").unwrap();
    ///     let txs = client
    ///         .get_normal_transactions("0x58eB28A67731c570Ef827C365c89B5751F9E6b0a", None)
    ///         .await.unwrap();
    /// # }
    /// ```
    pub async fn get_normal_transactions(
        &self,
        address: impl AsRef<str>,
        params: Option<TxListParams>,
    ) -> Result<Vec<NormalTransaction>> {
        let mut tx_params: HashMap<&str, String> = params.unwrap_or_default().into();
        tx_params.insert("address", address.as_ref().to_string());
        let query = self.create_query("account", "txlist", tx_params);
        let response: Response<Vec<NormalTransaction>> = self.get_json(&query).await?;

        Ok(response.result)
    }

    /// Returns the list of internal transactions performed by an address or within a transaction,
    /// with optional pagination.
    ///
    /// ```no_run
    /// # use ethers_etherscan::{Client, account::InternalTxQueryOption};
    /// # use ethers_core::types::Chain;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    ///     let client = Client::new(Chain::Mainnet, "API_KEY").unwrap();
    ///     let txs = client
    ///         .get_internal_transactions(
    ///             InternalTxQueryOption::ByAddress("0x2c1ba59d6f58433fb1eaee7d20b26ed83bda51a3"),
    ///             None).await.unwrap();
    /// # }
    /// ```
    pub async fn get_internal_transactions<'a, A: Into<Cow<'a, str>>>(
        &self,
        tx_query_option: InternalTxQueryOption<A>,
        params: Option<TxListParams>,
    ) -> Result<Vec<InternalTransaction>> {
        let mut tx_params: HashMap<&str, String> = params.unwrap_or_default().into();
        match tx_query_option {
            InternalTxQueryOption::ByAddress(address) => {
                tx_params.insert("address", address.into().into_owned());
            }
            InternalTxQueryOption::ByTransactionHash(tx_hash) => {
                tx_params.insert("txhash", tx_hash.into().into_owned());
            }
            _ => {}
        }
        let query = self.create_query("account", "txlistinternal", tx_params);
        let response: Response<Vec<InternalTransaction>> = self.get_json(&query).await?;

        Ok(response.result)
    }

    /// Returns the list of ERC-20 tokens transferred by an address, with optional filtering by
    /// token contract.
    ///
    /// ```no_run
    /// # use ethers_etherscan::{Client, account::TokenQueryOption};
    /// # use ethers_core::types::Chain;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    ///     let client = Client::new(Chain::Mainnet, "API_KEY").unwrap();
    ///     let txs = client
    ///         .get_erc20_token_transfer_events(
    ///             TokenQueryOption::ByAddress("0x4e83362442b8d1bec281594cea3050c8eb01311c"),
    ///             None).await.unwrap();
    /// # }
    /// ```
    pub async fn get_erc20_token_transfer_events<'a, A: Into<Cow<'a, str>>>(
        &self,
        event_query_option: TokenQueryOption<A>,
        params: Option<TxListParams>,
    ) -> Result<Vec<ERC20TokenTransferEvent>> {
        let params = event_query_option.into_params(params.unwrap_or_default());
        let query = self.create_query("account", "tokentx", params);
        let response: Response<Vec<ERC20TokenTransferEvent>> = self.get_json(&query).await?;

        Ok(response.result)
    }

    /// Returns the list of ERC-721 ( NFT ) tokens transferred by an address, with optional
    /// filtering by token contract.
    ///
    /// ```no_run
    /// # use ethers_etherscan::{Client, account::TokenQueryOption};
    /// # use ethers_core::types::Chain;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    ///     let client = Client::new(Chain::Mainnet, "API_KEY").unwrap();
    ///     let txs = client
    ///         .get_erc721_token_transfer_events(
    ///             TokenQueryOption::ByAddressAndContract(
    ///                 "0x6975be450864c02b4613023c2152ee0743572325",
    ///                 "0x06012c8cf97bead5deae237070f9587f8e7a266d",
    ///          ), None).await.unwrap();
    /// # }
    /// ```
    pub async fn get_erc721_token_transfer_events<'a, A: Into<Cow<'a, str>>>(
        &self,
        event_query_option: TokenQueryOption<A>,
        params: Option<TxListParams>,
    ) -> Result<Vec<ERC721TokenTransferEvent>> {
        let params = event_query_option.into_params(params.unwrap_or_default());
        let query = self.create_query("account", "tokennfttx", params);
        let response: Response<Vec<ERC721TokenTransferEvent>> = self.get_json(&query).await?;

        Ok(response.result)
    }

    /// Returns the list of blocks mined by an address.
    ///
    /// ```no_run
    /// # use ethers_etherscan::Client;
    /// # use ethers_core::types::Chain;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    ///     let client = Client::new(Chain::Mainnet, "API_KEY").unwrap();
    ///     let blocks = client
    ///         .get_mined_blocks("0x9dd134d14d1e65f84b706d6f205cd5b1cd03a46b", None, None)
    ///         .await.unwrap();
    /// # }
    /// ```
    pub async fn get_mined_blocks(
        &self,
        address: impl AsRef<str>,
        block_type: Option<BlockType>,
        page_and_offset: Option<(u64, u64)>,
    ) -> Result<Vec<MinedBlock>> {
        let mut params = HashMap::new();
        params.insert("address", address.as_ref().to_string());
        params.insert("blocktype", block_type.unwrap_or_default().to_string());
        if let Some((page, offset)) = page_and_offset {
            params.insert("page", page.to_string());
            params.insert("offset", offset.to_string());
        }
        let query = self.create_query("account", "getminedblocks", params);
        let response: Response<Vec<MinedBlock>> = self.get_json(&query).await?;

        Ok(response.result)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use serial_test::serial;

    use crate::{tests::run_at_least_duration, Chain};

    use super::*;

    #[tokio::test]
    #[serial]
    async fn get_ether_balance_single_success() {
        run_at_least_duration(Duration::from_millis(250), async {
            let client = Client::new_from_env(Chain::Mainnet).unwrap();

            let balance = client
                .get_ether_balance_single("0x58eB28A67731c570Ef827C365c89B5751F9E6b0a", None)
                .await;
            assert!(balance.is_ok());
        })
        .await
    }

    #[tokio::test]
    #[serial]
    async fn get_ether_balance_multi_success() {
        run_at_least_duration(Duration::from_millis(250), async {
            let client = Client::new_from_env(Chain::Mainnet).unwrap();

            let balances = client
                .get_ether_balance_multi(&vec!["0x58eB28A67731c570Ef827C365c89B5751F9E6b0a"], None)
                .await;
            assert!(balances.is_ok());
            let balances = balances.unwrap();
            assert!(balances.len() == 1);
        })
        .await
    }

    #[tokio::test]
    #[serial]
    async fn get_normal_transactions_success() {
        run_at_least_duration(Duration::from_millis(250), async {
            let client = Client::new_from_env(Chain::Mainnet).unwrap();

            let txs = client
                .get_normal_transactions("0x58eB28A67731c570Ef827C365c89B5751F9E6b0a", None)
                .await;
            assert!(txs.is_ok());
        })
        .await
    }

    #[tokio::test]
    #[serial]
    async fn get_internal_transactions_success() {
        run_at_least_duration(Duration::from_millis(250), async {
            let client = Client::new_from_env(Chain::Mainnet).unwrap();

            let txs = client
                .get_internal_transactions(
                    InternalTxQueryOption::ByAddress("0x2c1ba59d6f58433fb1eaee7d20b26ed83bda51a3"),
                    None,
                )
                .await;
            assert!(txs.is_ok());
        })
        .await
    }

    #[tokio::test]
    #[serial]
    async fn get_internal_transactions_by_tx_hash_success() {
        run_at_least_duration(Duration::from_millis(250), async {
            let client = Client::new_from_env(Chain::Mainnet).unwrap();

            let txs = client
                .get_internal_transactions(
                    InternalTxQueryOption::ByTransactionHash(
                        "0x40eb908387324f2b575b4879cd9d7188f69c8fc9d87c901b9e2daaea4b442170",
                    ),
                    None,
                )
                .await;
            assert!(txs.is_ok());
        })
        .await
    }

    #[tokio::test]
    #[serial]
    async fn get_erc20_transfer_events_success() {
        run_at_least_duration(Duration::from_millis(250), async {
            let client = Client::new_from_env(Chain::Mainnet).unwrap();

            let txs = client
                .get_erc20_token_transfer_events(
                    TokenQueryOption::ByAddress("0x4e83362442b8d1bec281594cea3050c8eb01311c"),
                    None,
                )
                .await;
            assert!(txs.is_ok());
        })
        .await
    }

    #[tokio::test]
    #[serial]
    async fn get_erc721_transfer_events_success() {
        run_at_least_duration(Duration::from_millis(250), async {
            let client = Client::new_from_env(Chain::Mainnet).unwrap();

            let txs = client
                .get_erc721_token_transfer_events(
                    TokenQueryOption::ByAddressAndContract(
                        "0x6975be450864c02b4613023c2152ee0743572325",
                        "0x06012c8cf97bead5deae237070f9587f8e7a266d",
                    ),
                    None,
                )
                .await;
            assert!(txs.is_ok());
        })
        .await
    }

    #[tokio::test]
    #[serial]
    async fn get_mined_blocks_success() {
        run_at_least_duration(Duration::from_millis(250), async {
            let client = Client::new_from_env(Chain::Mainnet).unwrap();

            let blocks = client
                .get_mined_blocks("0x9dd134d14d1e65f84b706d6f205cd5b1cd03a46b", None, None)
                .await;
            assert!(blocks.is_ok());
        })
        .await
    }
}
