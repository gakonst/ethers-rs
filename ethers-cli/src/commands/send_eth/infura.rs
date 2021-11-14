mod ropsten;

use abscissa_core::{Clap, Command, Runnable};

#[derive(Command, Debug, Clap, Runnable)]
pub enum InfuraCmd {
    Ropsten(ropsten::RopstenCmd),
}
