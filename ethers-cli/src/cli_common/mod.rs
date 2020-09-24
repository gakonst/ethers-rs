use gumdrop::Options;

pub mod abigen;
pub use abigen::{generate, AbigenOpts};

#[derive(Debug, Options)]
pub struct EthersCliOpts {
    help: bool,
    #[options(command)]
    pub command: Option<Command>,
}

#[derive(Debug, Options)]
pub enum Command {
    #[options(help = "generate type-safe Rust bindings from a contract's ABI")]
    Abigen(AbigenOpts),
}
