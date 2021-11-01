//! EthersCli Subcommands
//!
//! This is where you specify the subcommands of your application.
//!
//! The default application comes with two subcommands:
//!
//! - `start`: launches the application
//! - `version`: print application version
//!
//! See the `impl Configurable` below for how to specify the path to the
//! application's configuration file.

mod abigen;
mod start;
mod version;

use self::{start::StartCmd, version::VersionCmd, abigen::AbigenCmd};
use crate::config::EthersCliConfig;
use abscissa_core::{
    config::Override, Command, Configurable, FrameworkError, Help, Options, Runnable,
};
use std::path::PathBuf;

/// EthersCli Configuration Filename
pub const CONFIG_FILE: &str = "ethers_cli.toml";

/// EthersCli Subcommands
#[derive(Command, Debug, Options, Runnable)]
pub enum EthersCliCmd {
    /// The `help` subcommand
    #[options(help = "get usage information")]
    Help(Help<Self>),

    /// The `start` subcommand
    #[options(help = "start the application")]
    Start(StartCmd),

    /// The `version` subcommand
    #[options(help = "display version information")]
    Version(VersionCmd),

    /// The `abigen` subcommand
    #[options(help = "Abi generator for  contracts")]
    Abigen(AbigenCmd),
}

/// This trait allows you to define how application configuration is loaded.
impl Configurable<EthersCliConfig> for EthersCliCmd {
    /// Location of the configuration file
    fn config_path(&self) -> Option<PathBuf> {
        // Check if the config file exists, and if it does not, ignore it.
        // If you'd like for a missing configuration file to be a hard error
        // instead, always return `Some(CONFIG_FILE)` here.
        let filename = PathBuf::from(CONFIG_FILE);

        if filename.exists() {
            Some(filename)
        } else {
            None
        }
    }

    /// Apply changes to the config after it's been loaded, e.g. overriding
    /// values in a config file using command-line options.
    ///
    /// This can be safely deleted if you don't want to override config
    /// settings from command-line options.
    fn process_config(&self, config: EthersCliConfig) -> Result<EthersCliConfig, FrameworkError> {
        match self {
            EthersCliCmd::Start(cmd) => cmd.override_config(config),
            _ => Ok(config),
        }
    }
}
