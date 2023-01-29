use ethers::prelude::*;
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// use `debug_traceCall` to fetch traces
/// requires, a valid endpoint in `RPC_URL` env var that supports `debug_traceCall`
/// Currently, Geth support several builtin tracers. For details, please read
/// https://geth.ethereum.org/docs/developers/evm-tracing/built-in-tracers.
///
/// However, currently ethers-rs only support 3 types of tracers.
/// 1. Struct/opcode logger tracer (This is the default one when no tracer is configured)
/// 2. callTracer -> https://geth.ethereum.org/docs/developers/evm-tracing/built-in-tracers#call-tracer
/// 3. javascript tracer -> https://geth.ethereum.org/docs/developers/evm-tracing/custom-tracer#custom-javascript-tracing

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
struct CustomJSTracerResult {
    result: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    if let Ok(url) = std::env::var("RPC_URL") {
        let client = Provider::<Http>::try_from(url)?;
        let tx = TransactionRequest::new().from(Address::from_str("0xdeadbeef29292929192939494959594933929292").unwrap()).to(Address::from_str("0xde929f939d939d393f939393f93939f393929023").unwrap()).gas("0x7a120").data(Bytes::from_str("0xf00d4b5d00000000000000000000000001291230982139282304923482304912923823920000000000000000000000001293123098123928310239129839291010293810").unwrap());
        let block = BlockId::from(16213100);

        // 1. default Struct/opcode logger tracer
        let options: GethDebugTracingCallOptions = GethDebugTracingCallOptions {
            tracing_options: GethDebugTracingOptions {
                disable_storage: Some(true),
                enable_memory: Some(false),
                tracer: None,
                ..Default::default()
            },
        };
        let traces: GethTrace =
            client.debug_trace_call(tx.clone(), Some(block.clone()), options).await?;
        println!("1. {traces:?}");

        // 2. callTracer
        let options: GethDebugTracingCallOptions = GethDebugTracingCallOptions {
            tracing_options: GethDebugTracingOptions {
                disable_storage: Some(true),
                enable_memory: Some(false),
                tracer: Some(GethDebugTracerType::CallTracer),
                ..Default::default()
            },
        };
        let traces: CallTrace =
            client.debug_trace_call(tx.clone(), Some(block.clone()), options).await?;
        println!("2. {traces:?}");

        // 3. javascript tracer
        let options: GethDebugTracingCallOptions = GethDebugTracingCallOptions {
            tracing_options: GethDebugTracingOptions {
                disable_storage: Some(true),
                enable_memory: Some(false),
                tracer: Some(GethDebugTracerType::JSTracer(
                    "{data: [], fault: function(log) {},step: function(log) { if(log.op.toString() == \"CALL\") this.data.push(log.stack.peek(0));},result: function() { return {\"result\":\"HelloWorld\"};}}".to_string()
                )),
                ..Default::default()
            },
        };
        let traces: CustomJSTracerResult =
            client.debug_trace_call(tx, Some(block), options).await?;
        println!("3. {traces:?}");
    }
    Ok(())
}
