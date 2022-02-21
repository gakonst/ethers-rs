pub mod generic;
pub mod solc;
pub mod vyper;

use std::path::PathBuf;

pub use generic::*;
pub use semver::Version;
pub use solc::*;
pub use vyper::*;

use crate::{error::Result, CompilerInput, CompilerOutput};
use async_trait::async_trait;

#[async_trait]
pub trait CompilerTrait {
    fn path(&self) -> PathBuf;

    fn arg(&mut self, arg: String);
    fn args(&mut self, args: Vec<String>);
    fn get_args(&self) -> Vec<String>;

    fn version(&self) -> Version;
    fn language(&self) -> String;

    fn compile_exact(&self, input: &CompilerInput) -> Result<CompilerOutput>;
    fn compile(&self, input: &CompilerInput) -> Result<CompilerOutput>;
}
