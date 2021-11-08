//! `start` subcommand - example of how to write a subcommand

/// App-local prelude includes `app_reader()`/`app_writer()`/`app_config()`
/// accessors along with logging macros. Customize as you see fit.
use crate::prelude::*;

use crate::config::EthersCliConfig;
use abscissa_core::{config, Clap, Command, FrameworkError, Runnable};

/// `start` subcommand
///
/// The `Clap` proc macro generates an option parser based on the struct
/// definition, and is defined in the `gumdrop` crate. See their documentation
/// for a more comprehensive example:
///
/// <https://docs.rs/clap/>
#[derive(Command, Debug, Clap)]
pub struct StartCmd {
    /// To whom are we saying hello?
    recipient: Vec<String>,
}

impl Runnable for StartCmd {
    /// Start the application.
    fn run(&self) {
        let config = APP.config();
        println!("Hello, {}!", &config.hello.recipient);
    }
}

impl config::Override<EthersCliConfig> for StartCmd {
    // Process the given command line options, overriding settings from
    // a configuration file using explicit flags taken from command-line
    // arguments.
    fn override_config(
        &self,
        mut config: EthersCliConfig,
    ) -> Result<EthersCliConfig, FrameworkError> {
        if !self.recipient.is_empty() {
            config.hello.recipient = self.recipient.join(" ");
        }

        Ok(config)
    }
}
