use crate::*;
use ethers_etherscan::contract::SourceCodeMetadata;
use serial_test::serial;

/// Abi of [0xBB9bc244D798123fDe783fCc1C72d3Bb8C189413](https://api.etherscan.io/api?module=contract&action=getsourcecode&address=0xBB9bc244D798123fDe783fCc1C72d3Bb8C189413).
const DAO_ABI: &str = include!("../../../testdata/the_dao_abi.expr");

#[tokio::test]
#[serial]
async fn can_fetch_ftm_contract_abi() {
    run_with_client(Chain::Fantom, |client| async move {
        let _abi = client
            .contract_abi("0x80AA7cb0006d5DDD91cce684229Ac6e398864606".parse().unwrap())
            .await
            .unwrap();
    })
    .await;
}

#[tokio::test]
#[serial]
async fn can_fetch_contract_abi() {
    run_with_client(Chain::Mainnet, |client| async move {
        let abi = client
            .contract_abi("0xBB9bc244D798123fDe783fCc1C72d3Bb8C189413".parse().unwrap())
            .await
            .unwrap();
        assert_eq!(abi, serde_json::from_str(DAO_ABI).unwrap());
    })
    .await;
}

#[tokio::test]
#[serial]
async fn can_fetch_contract_source_code() {
    run_with_client(Chain::Mainnet, |client| async move {
        let meta = client
            .contract_source_code("0xBB9bc244D798123fDe783fCc1C72d3Bb8C189413".parse().unwrap())
            .await
            .unwrap();

        assert_eq!(meta.items.len(), 1);
        let item = &meta.items[0];
        assert!(matches!(item.source_code, SourceCodeMetadata::SourceCode(_)));
        assert_eq!(item.source_code.sources().len(), 1);
        assert_eq!(item.abi().unwrap(), serde_json::from_str(DAO_ABI).unwrap());
    })
    .await
}

#[tokio::test]
#[serial]
async fn can_get_error_on_unverified_contract() {
    init_tracing();
    run_with_client(Chain::Mainnet, |client| async move {
        let addr = "0xb5c31a0e22cae98ac08233e512bd627885aa24e5".parse().unwrap();
        let err = client.contract_source_code(addr).await.unwrap_err();
        assert!(matches!(err, EtherscanError::ContractCodeNotVerified(_)));
    })
    .await
}

/// Query a contract that has a single string source entry instead of underlying JSON metadata.
#[tokio::test]
#[serial]
async fn can_fetch_contract_source_tree_for_singleton_contract() {
    run_with_client(Chain::Mainnet, |client| async move {
        let meta = client
            .contract_source_code("0xBB9bc244D798123fDe783fCc1C72d3Bb8C189413".parse().unwrap())
            .await
            .unwrap();

        assert_eq!(meta.items.len(), 1);
        let item = &meta.items[0];
        assert!(matches!(item.source_code, SourceCodeMetadata::SourceCode(_)));
        assert_eq!(item.source_code.sources().len(), 1);
        assert_eq!(item.abi().unwrap(), serde_json::from_str(DAO_ABI).unwrap());
    })
    .await
}

/// Query a contract that has many source entries as JSON metadata and ensure they are reflected.
#[tokio::test]
#[serial]
async fn can_fetch_contract_source_tree_for_multi_entry_contract() {
    run_with_client(Chain::Mainnet, |client| async move {
        let meta = client
            .contract_source_code("0x8d04a8c79cEB0889Bdd12acdF3Fa9D207eD3Ff63".parse().unwrap())
            .await
            .unwrap();

        assert_eq!(meta.items.len(), 1);
        assert!(matches!(meta.items[0].source_code, SourceCodeMetadata::Metadata { .. }));
        assert_eq!(meta.source_tree().entries.len(), 15);
    })
    .await
}

/// Query a contract that has a plain source code mapping instead of tagged structures.
#[tokio::test]
#[serial]
async fn can_fetch_contract_source_tree_for_plain_source_code_mapping() {
    run_with_client(Chain::Mainnet, |client| async move {
        let meta = client
            .contract_source_code("0x68b26dcf21180d2a8de5a303f8cc5b14c8d99c4c".parse().unwrap())
            .await
            .unwrap();

        assert_eq!(meta.items.len(), 1);
        assert!(matches!(meta.items[0].source_code, SourceCodeMetadata::Sources(_)));
        assert_eq!(meta.source_tree().entries.len(), 6);
    })
    .await
}
