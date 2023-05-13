use ethers_core::{types::Filter, utils::Anvil};
use ethers_providers::{Middleware, Provider, StreamExt};
use futures_util::SinkExt;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{
    accept_async,
    tungstenite::{
        self,
        protocol::{frame::coding::CloseCode, CloseFrame},
        Error,
    },
};
use tungstenite::protocol::Message;

const WS_ENDPOINT: &str = "127.0.0.1:9002";

async fn spawn_ws_server() {
    let listener = TcpListener::bind(&WS_ENDPOINT).await.expect("Can't listen");
    tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            tokio::spawn(handle_conn(stream));
        }
    });
}

async fn handle_conn(stream: TcpStream) -> Result<(), Error> {
    let mut ws_stream = accept_async(stream).await?;

    while ws_stream.next().await.is_some() {
        let res: String =
            "{\"jsonrpc\":\"2.0\",\"id\":0,\"result\":\"0xcd0c3e8af590364c09d0fa6a1210faf5\"}"
                .into();

        // Answer with a valid RPC response to keep the connection alive
        ws_stream.send(Message::Text(res)).await?;

        // Wait for a while
        let timeout = Duration::from_secs(2);
        tokio::time::sleep(timeout).await;

        // Drop the connection
        ws_stream
            .send(Message::Close(Some(CloseFrame {
                code: CloseCode::Error,
                reason: "Upstream went away".into(),
            })))
            .await?;
    }

    Ok(())
}

#[tokio::test]
async fn graceful_disconnect_on_ws_errors() {
    // Spawn a fake Ws server that will drop our connection after a while
    spawn_ws_server().await;

    // Connect to the fake server
    let provider =
        Provider::connect_with_reconnects(format!("ws://{WS_ENDPOINT}"), 1).await.unwrap();
    let filter = Filter::new().event("Transfer(address,address,uint256)");
    let mut stream = provider.subscribe_logs(&filter).await.unwrap();

    assert!(stream.next().await.is_none());
}

#[tokio::test]
async fn resubscribe_on_ws_reconnect() {
    let anvil = Anvil::new().block_time(1u64).spawn();
    let port = anvil.port();
    let provider = Provider::connect_with_reconnects(anvil.ws_endpoint(), 1).await.unwrap();

    // Attempt to ensure a different server-side subscription id after reconnect by making
    // the subscription we care about be the second one after initial startup, but the first
    // (and only) one after reconnection.
    let ignored_sub = provider.subscribe_blocks().await.unwrap();
    let mut blocks = provider.subscribe_blocks().await.unwrap();
    ignored_sub.unsubscribe().await.expect("unsubscribe failed");

    blocks.next().await.expect("no block notice before reconnect");

    // Kill & restart using the same port so we end up with the same endpoint url:
    drop(anvil);
    let _anvil = Anvil::new().port(port).block_time(1u64).spawn();

    // Wait for the next block on existing subscription. Will fail w/o resubscription:
    blocks.next().await.expect("no block notice after reconnect");
}
