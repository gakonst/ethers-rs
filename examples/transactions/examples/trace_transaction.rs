use ethers::{
    core::types::{GethDebugTracingOptions, H256},
    providers::{Http, Middleware, Provider},
    types::{GethDebugBuiltInTracerType, GethDebugTracerType},
};
use eyre::Result;
use std::str::FromStr;

/// use `debug_traceTransaction` to fetch traces
/// requires, a valid endpoint in `RPC_URL` env var that supports `debug_traceTransaction`
#[tokio::main]
async fn main() -> Result<()> {
    if let Ok(url) = std::env::var("RPC_URL") {
        let client = Provider::<Http>::try_from(url)?;
        let tx_hash = "0x97a02abf405d36939e5b232a5d4ef5206980c5a6661845436058f30600c52df7";
        let h: H256 = H256::from_str(tx_hash)?;

        // default tracer
        let options = GethDebugTracingOptions::default();
        let traces = client.debug_trace_transaction(h, options).await?;
        println!("{traces:?}");

        // call tracer
        let options = GethDebugTracingOptions {
            disable_storage: Some(true),
            enable_memory: Some(false),
            tracer: Some(GethDebugTracerType::BuiltInTracer(
                GethDebugBuiltInTracerType::CallTracer,
            )),
            ..Default::default()
        };
        let traces = client.debug_trace_transaction(h, options).await?;
        println!("{traces:?}");

        // js tracer
        let options = GethDebugTracingOptions {
                disable_storage: Some(true),
                enable_memory: Some(false),
                tracer: Some(GethDebugTracerType::JsTracer(String::from("{data: [], fault: function(log) {}, step: function(log) { if(log.op.toString() == \"DELEGATECALL\") this.data.push(log.stack.peek(0)); }, result: function() { return this.data; }}"))),
                ..Default::default()
            };
        let traces = client.debug_trace_transaction(h, options).await?;
        println!("{traces:?}");
    }

    Ok(())
}
