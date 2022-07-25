use std::path::PathBuf;

use tokio::runtime::Builder;
use tokio_stream::StreamExt;

use ethers_core::utils::Geth;

use ethers_connections::{connection::ipc::Ipc, types::BlockNumber, Provider};

#[cfg(unix)]
#[test]
fn ipc_raw_rpc_calls() {
    use ethers_core::types::{Address, U256, U64};

    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("geth").with_extension("ipc");
    let geth = Geth::new().ipc_path(&path).block_time(5u64).spawn();

    let rt = Builder::new_current_thread().enable_all().build().unwrap();
    rt.block_on(async move {
        let ipc = Ipc::connect(path).await.unwrap();
        let provider = Provider::new(ipc);

        // eth_blockNumber
        let call = provider.prepare_rpc_call("eth_blockNumber", ());
        let block: U64 = call.await.expect("failed to get block number");
        assert!(block.low_u64() < 2);

        // eth_getBalance
        let address = Address::zero();
        let call = provider.prepare_rpc_call("eth_getBalance", (address, "latest"));
        let balance: U256 = call.await.expect("failed to get balance");
        assert_eq!(balance, 0.into());

        let balance =
            provider.get_balance(&address, "latest".into()).await.expect("failed to get balance");
        assert_eq!(balance, 0.into());
    });

    drop(geth);
}

#[cfg(unix)]
#[test]
fn ipc_rpc_calls() {
    use ethers_core::types::Address;

    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("geth").with_extension("ipc");
    let geth = Geth::new().ipc_path(&path).block_time(5u64).spawn();

    let rt = Builder::new_current_thread().enable_all().build().unwrap();
    rt.block_on(async move {
        let ipc = Ipc::connect(path).await.unwrap();
        let provider = Provider::new(ipc);

        // eth_syncing
        let syncing = provider.get_syncing().await.unwrap();
        assert!(syncing.is_synced());

        // eth_coinbase
        let _ = provider.get_coinbase().await.unwrap();

        // eth_mining
        let mining = provider.get_mining().await.unwrap();
        assert!(mining);

        // eth_balance
        let zero = Address::zero();
        let balance = provider.get_balance(&zero, 0u64.into()).await.unwrap();
        assert_eq!(balance, 0.into());
        let balance = provider.get_balance(&zero, BlockNumber::Number(0x0)).await.unwrap();
        assert_eq!(balance, 0.into());
        let balance = provider.get_balance(&zero, BlockNumber::Earliest).await.unwrap();
        assert_eq!(balance, 0.into());
        let balance = provider.get_balance(&zero, BlockNumber::Latest).await.unwrap();
        assert_eq!(balance, 0.into());
        let balance = provider.get_balance(&zero, BlockNumber::Pending).await.unwrap();
        assert_eq!(balance, 0.into());
    });

    drop(geth);
}

#[cfg(unix)]
#[test]
fn ipc_dyn_connect() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("geth").with_extension("ipc");
    let geth = Geth::new().ipc_path(&path).block_time(5u64).spawn();

    let rt = Builder::new_current_thread().enable_all().build().unwrap();
    rt.block_on(async move {
        let provider = Provider::connect(path.to_str().unwrap()).await.unwrap();
        let status = provider.get_syncing().await.unwrap();
        assert!(status.is_synced());
    });

    drop(geth);
}

#[cfg(unix)]
#[test]
fn ipc_subscribe() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("geth").with_extension("ipc");
    let geth = Geth::new().ipc_path(&path).block_time(1u64).spawn();

    let rt = Builder::new_current_thread().enable_all().build().unwrap();
    rt.block_on(async move {
        let connection = Ipc::connect(&path).await.expect("failed to connect to IPC socket");
        let provider = Provider { connection };

        let mut stream = provider.borrow().subscribe_blocks().await.unwrap();
        let mut curr = stream.next().await.unwrap().unwrap().number.unwrap().low_u64();

        let end = 10;
        println!("got block #{curr}/{end}");

        while let Some(res) = stream.next().await {
            let block = res.unwrap().number.unwrap().low_u64();
            assert_eq!(block, curr + 1);
            curr = block;
            println!("got block #{curr}/{end}");

            if curr == end {
                break;
            }
        }

        stream.unsubscribe().await.unwrap();
    });

    drop(geth);
}
