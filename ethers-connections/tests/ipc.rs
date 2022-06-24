use std::path::PathBuf;

use tokio::runtime::Builder;

use ethers_core::utils::Geth;

use ethers_connections::{connections::ipc::Ipc, types::BlockNumber, Provider};

#[cfg(unix)]
#[test]
fn geth_ipc() {
    use ethers_core::types::Address;

    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("geth").with_extension("ipc");
    let geth = Geth::new().ipc_path(&path).block_time(5u64).spawn();

    let rt = Builder::new_current_thread().enable_all().build().unwrap();
    rt.block_on(async move {
        let ipc = Ipc::connect(path).await.unwrap();
        let provider = Provider::new(ipc);

        // eth_protocolVersion
        let err = provider.get_protocol_version().await.unwrap_err();
        let jsonrpc = err.as_jsonrpc().unwrap();
        assert_eq!(
            jsonrpc.message,
            "the method eth_protocolVersion does not exist/is not available"
        );

        // eth_syncing
        let syncing = provider.syncing().await.unwrap();
        assert!(syncing.is_none());

        // eth_coinbase
        let _ = provider.get_coinbase().await.unwrap();

        // eth_mining
        let mining = provider.get_mining().await.unwrap();
        assert!(mining);

        // eth_balance
        let zero = Address::zero();
        let balance = provider.get_balance(&zero, &0u64.into()).await.unwrap();
        assert_eq!(balance, 0.into());
        let balance = provider.get_balance(&zero, &BlockNumber::Number(0x0)).await.unwrap();
        assert_eq!(balance, 0.into());
        let balance = provider.get_balance(&zero, &BlockNumber::Earliest).await.unwrap();
        assert_eq!(balance, 0.into());
        let balance = provider.get_balance(&zero, &BlockNumber::Latest).await.unwrap();
        assert_eq!(balance, 0.into());
        let balance = provider.get_balance(&zero, &BlockNumber::Pending).await.unwrap();
        assert_eq!(balance, 0.into());
    });

    drop(geth);
}
