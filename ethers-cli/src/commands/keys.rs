mod add;
mod import;
mod show;

use abscissa_core::{Command, Clap, Runnable};

/// Key management commands for cli 
#[derive(Command, Debug, Clap, Runnable)]
pub enum KeysCmd {
    Add(add::AddKeyCmd),
    Import(import::ImportKeyCmd),
    Show(show::ShowKeyCmd)
}