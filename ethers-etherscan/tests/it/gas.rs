use crate::*;
use ethers_core::types::U256;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn gas_estimate_success() {
    run_with_client(Chain::Mainnet, |client| async move {
        let result = client.gas_estimate(2000000000u32.into()).await;

        result.unwrap();
    })
    .await
}

#[tokio::test]
#[serial]
async fn gas_estimate_error() {
    run_with_client(Chain::Mainnet, |client| async move {
        let err = client.gas_estimate(7123189371829732819379218u128.into()).await.unwrap_err();

        assert!(matches!(err, EtherscanError::GasEstimationFailed));
    })
    .await
}

#[tokio::test]
#[serial]
async fn gas_oracle_success() {
    run_with_client(Chain::Mainnet, |client| async move {
        let result = client.gas_oracle().await;

        assert!(result.is_ok());

        let oracle = result.unwrap();

        assert!(oracle.safe_gas_price > U256::zero());
        assert!(oracle.propose_gas_price > U256::zero());
        assert!(oracle.fast_gas_price > U256::zero());
        assert!(oracle.last_block > 0);
        assert!(oracle.suggested_base_fee > U256::zero());
        assert!(!oracle.gas_used_ratio.is_empty());
    })
    .await
}
