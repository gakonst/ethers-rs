use ethers::{
    core::types::GethDebugTracingOptions,
    providers::{Http, Middleware, Provider},
    types::{
        Address, BlockId, Bytes, GethDebugBuiltInTracerType, GethDebugTracerType,
        GethDebugTracingCallOptions, TransactionRequest,
    },
};
use eyre::Result;
use std::str::FromStr;

/// use `debug_traceCall` to fetch traces
/// requires, a valid endpoint in `RPC_URL` env var that supports `debug_traceCall`
#[tokio::main]
async fn main() -> Result<()> {
    if let Ok(url) = std::env::var("RPC_URL") {
        let client = Provider::<Http>::try_from(url)?;
        let tx = TransactionRequest::new().from(Address::from_str("0xdeadbeef29292929192939494959594933929292").unwrap()).to(Address::from_str("0xde929f939d939d393f939393f93939f393929023").unwrap()).gas("0x7a120").data(Bytes::from_str("0xf00d4b5d00000000000000000000000001291230982139282304923482304912923823920000000000000000000000001293123098123928310239129839291010293810").unwrap());
        let block = BlockId::from(16213100);
        let options: GethDebugTracingCallOptions = GethDebugTracingCallOptions {
            tracing_options: GethDebugTracingOptions {
                disable_storage: Some(true),
                enable_memory: Some(false),
                tracer: Some(GethDebugTracerType::BuiltInTracer(
                    GethDebugBuiltInTracerType::CallTracer,
                )),
                ..Default::default()
            },
            state_overrides: None,
            block_overrides: None,
        };
        let traces = client.debug_trace_call(tx, Some(block), options).await?;
        println!("{traces:?}");
    }

    Ok(())
}
