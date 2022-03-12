/// A Compiler-specific Result Type
pub type Result<T, E> = std::result::Result<T, E>;

/// # Compiler
///
/// A Generalized Compiler Trait
/// Must be implemented to support specific compilation.
///
/// ### Example Implementation
///
/// ```no_run
/// use ethers_compile::{Compiler};
///
/// pub struct Sanskrit {};
///
/// pub enum SError {
///   IO,
///   Compile,
///   Unknown
/// };
///
/// impl Compiler<u64, u64, SError> for Sanskrit {
///   fn compile(&self, input: &Self::Input) -> Result<Self::Output, Self::Error> {
///     return Ok(1);
///   }
/// };
///
/// let scompiler = Sanskrit {};
/// let result = scompiler.compile(&input).unwrap();
/// ```
pub trait Compiler {
  type Input;
  type Output;
  type Error;

  pub fn compile(&self, input: &Self::Input) -> Result<Self::Output, Self::Error>;

  /// Exposes an api to set the underlying compiler version
  pub fn configure_version(&self) -> Result<bool, Self::Error>;
}