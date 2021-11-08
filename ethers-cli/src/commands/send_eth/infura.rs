mod ropsten;

use abscissa_core::{Command, Clap, Runnable};

#[derive(Command, Debug, Clap, Runnable)]
pub enum InfuraCmd {
    Ropsten(ropsten::RopstenCmd),
}