//! tracing layer for logging [CompilerInput] and [CompilerOutput]
//!
//! Useful for debugging purposes.
//! As solc compiler input and output can become quite large (in the tens of MB) we still want a way to get this info when debugging an issue.
//! Most convenient way to look at these object is as a separate json file

use std::collections::BTreeMap;
use tracing_subscriber::Layer;
use tracing::metadata::Metadata;
use tracing_subscriber::layer::Context;

pub struct SolcCompilerIOLayer;

impl<S> Layer<S> for SolcCompilerIOLayer
    where
        S: tracing::Subscriber,
        S: for<'lookup> tracing_subscriber::registry::LookupSpan<'lookup>,
{

    // TODO env var solc io

    fn enabled(&self, metadata: &Metadata<'_>, ctx: Context<'_, S>) -> bool {
        // TODO check solc
        true
    }

    fn on_new_span(
        &self,
        attrs: &tracing::span::Attributes<'_>,
        id: &tracing::span::Id,
        ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let span = ctx.span(id).unwrap();
        println!("Got on_new_span!");
        println!("  level={:?}", span.metadata().level());
        println!("  target={:?}", span.metadata().target());
        println!("  name={:?}", span.metadata().name());

        // Our old friend, `println!` exploration.
        let mut visitor = PrintlnVisitor;
        attrs.record(&mut visitor);
    }

}


struct PrintlnVisitor;

impl tracing::field::Visit for PrintlnVisitor {
    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        println!("  field={} value={}", field.name(), value)
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        println!("  field={} value={}", field.name(), value)
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        println!("  field={} value={}", field.name(), value)
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        println!("  field={} value={}", field.name(), value)
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        println!("  field={} value={}", field.name(), value)
    }

    fn record_error(
        &mut self,
        field: &tracing::field::Field,
        value: &(dyn std::error::Error + 'static),
    ) {
        println!("  field={} value={}", field.name(), value)
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        println!("  field={} value={:?}", field.name(), value)
    }
}
