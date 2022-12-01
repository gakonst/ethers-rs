#![cfg(not(target_arch = "wasm32"))]
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

#[cfg(not(feature = "celo"))]
mod eth_tests {
    use ethers_core::types::Filter;
    use ethers_providers::{StreamExt, Ws};
    use tokio_tungstenite::connect_async;

    use super::*;

    #[tokio::test]
    async fn graceful_disconnect_on_ws_errors() {
        // Spawn a fake Ws server that will drop our connection after a while
        spawn_ws_server().await;

        // Connect to the fake server
        let (ws, _) = connect_async(format!("ws://{}", WS_ENDPOINT)).await.unwrap();
        let provider = Provider::new(Ws::new(ws));
        let filter = Filter::new().event("Transfer(address,address,uint256)");
        let mut stream = provider.subscribe_logs(&filter).await.unwrap();

        while let Some(_) = stream.next().await {
            assert!(false); // force test to fail
        }

        assert!(true);
    }
}

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

    while let Some(_) = ws_stream.next().await {
        let res: String =
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":\"0xcd0c3e8af590364c09d0fa6a1210faf5\"}"
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
